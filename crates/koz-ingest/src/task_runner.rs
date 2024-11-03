use std::sync::Arc;

use ahash::{AHashMap, AHashSet};
use anyhow::Context as _;
use chrono::{DateTime, Utc};
use futures::future::BoxFuture;
use koz_storage::{
    scheduled_task::{NewScheduledTask, ScheduledTask},
    Storage,
};
use parking_lot::{Mutex, RwLock};
use tracing::Instrument;

use crate::{Ingest, IngestRequest};

type ScheduledTaskFn = Box<dyn Send + Sync + Fn() -> BoxFuture<'static, anyhow::Result<()>>>;

pub struct ScheduledTaskRunner {
    tasks: RwLock<AHashMap<String, ScheduledTaskFn>>,
    running_tasks: Arc<Mutex<AHashSet<String>>>,
    storage: Storage,
}

impl ScheduledTaskRunner {
    pub fn new(storage: Storage) -> Self {
        Self {
            tasks: RwLock::default(),
            running_tasks: Arc::default(),
            storage,
        }
    }

    pub fn register<F, R>(&self, name: String, ingest: &Ingest, make_request: F)
    where
        F: Send + Sync + 'static + Fn() -> R,
        R: IngestRequest,
    {
        let ingest = ingest.clone();
        self.register_fn(name, move || {
            let ingest = ingest.clone();
            let request = (make_request)();
            Box::pin(async move {
                ingest.ask(request).await?;
                Ok(())
            })
        });
    }

    pub fn register_fn<F>(&self, name: String, task_fn: F)
    where
        F: Send + Sync + 'static + Fn() -> BoxFuture<'static, anyhow::Result<()>>,
    {
        self.tasks.write().insert(name, Box::new(task_fn));
    }

    #[tracing::instrument(skip(self))]
    pub async fn init_task(&self, task_name: &str, default_cron: &str) -> anyhow::Result<()> {
        let storage = self.storage.clone();

        if storage
            .scheduled_task
            .exists(task_name)
            .await
            .context("error while checking if task exists")?
        {
            tracing::trace!("task {task_name} already exists, skipping...");
            return Ok(());
        }

        let new_task = NewScheduledTask {
            name: task_name.into(),
            schedule: default_cron.to_owned(),
            enabled: true,
            next_run_at: Utc::now(),
        };

        let created_task = storage
            .scheduled_task
            .create(new_task)
            .await
            .context("error while creating task")?;

        tracing::info!("initialized scheduled task {}", created_task.name);

        Ok(())
    }

    #[tracing::instrument(skip(self, ingest))]
    async fn run(&self, ingest: Ingest) -> anyhow::Result<()> {
        let storage = self.storage.clone();

        loop {
            let maybe_task = storage
                .scheduled_task
                .next_task()
                .await
                .context("error while finding next task")?;

            let Some(task) = maybe_task else {
                tracing::debug!("no tasks found, sleeping...");
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                continue;
            };

            self.start_task(&storage, &task, &ingest)
                .await
                .with_context(|| format!("error while starting task: {}", task.name))?;
        }
    }

    #[tracing::instrument(skip(self, storage, task, ingest), fields(task_name = task.name))]
    async fn start_task(
        &self,
        storage: &Storage,
        task: &ScheduledTask,
        ingest: &Ingest,
    ) -> anyhow::Result<()> {
        if let Some(next_run_at) = task.next_run_at {
            if next_run_at > Utc::now() {
                tracing::debug!(task_name = %&task.name, next_run_at = ?task.next_run_at, "task scheduled for the future, sleeping...");
                return Ok(());
            }
        }

        let last_run_at = task.last_run_at.unwrap_or_else(Utc::now);
        let schedule = &task.schedule;
        let next_run_at = calculate_next_run_at(last_run_at, schedule)
            .context("error while calculating next run at")?;
        storage
            .scheduled_task
            .update_next_run_at(&task.name, next_run_at)
            .await
            .context("error while updating task next run at")?;

        self.run_task(task, ingest).await?;

        Ok(())
    }

    async fn run_task(&self, task: &ScheduledTask, ingest: &Ingest) -> anyhow::Result<()> {
        let task_name = &task.name;
        if self.running_tasks.lock().contains(task_name) {
            tracing::debug!("task {task_name} already running, skipping...");
            return Ok(());
        }
        self.running_tasks.lock().insert(task_name.into());

        let task_fut = {
            let tasks = self.tasks.read();
            let Some(task_fn) = tasks.get(task_name) else {
                anyhow::bail!("task {task_name} does not have a task function, skipping...");
            };
            (task_fn)()
        };

        let ingest = ingest.clone();
        let task_name = task_name.to_owned();
        let task_span = tracing::info_span!("scheduled_task", %task_name);
        tokio::task::spawn(
            async move {
                tracing::trace!("running task {task_name}...");
                if let Err(err) = task_fut.await {
                    tracing::error!(%task_name, ?err, "error while running task");
                }
                ingest.task_runner.running_tasks.lock().remove(&task_name);
                tracing::trace!("task {task_name} finished");
            }
            .instrument(task_span),
        );
        Ok(())
    }
}

pub struct RunScheduledTasks;

impl IngestRequest for RunScheduledTasks {
    type Output = ();

    async fn run(self, ingest: Ingest) -> anyhow::Result<()> {
        ingest
            .task_runner
            .run(ingest.clone())
            .await
            .context("error while running scheduled tasks")
    }
}

fn calculate_next_run_at(
    last_run_at: DateTime<Utc>,
    schedule: &str,
) -> anyhow::Result<DateTime<Utc>> {
    if let Some(every_str) = schedule.strip_prefix("@every:") {
        let every = humantime::parse_duration(every_str.trim())
            .with_context(|| format!("invalid @every schedule: {schedule}"))?;
        Ok(last_run_at + every)
    } else {
        anyhow::bail!("invalid schedule: {schedule}");
    }
}
