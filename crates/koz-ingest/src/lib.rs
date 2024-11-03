mod lol;
mod task_runner;

use std::{collections::HashSet, future::Future, str::FromStr as _, sync::Arc};

use anyhow::Context;
use koz_storage::{scheduled_task::ScheduledTask, Storage};
use koz_types::lol::{LolRank, LolRegion};
use lol::region::PeriodicallyIngestLeague;
use swain::Swain;
use task_runner::{RunScheduledTasks, ScheduledTaskRunner};
use tracing::instrument;

#[derive(Clone)]
pub struct Ingest {
    inner: Arc<IngestInner>,
}

impl Ingest {
    pub fn new(config: IngestConfig, storage: Storage, swain: Swain) -> anyhow::Result<Self> {
        let mut regions_to_ingest = HashSet::new();
        for region_str in config.regions_to_ingest {
            if region_str.eq_ignore_ascii_case("all") {
                regions_to_ingest.extend(LolRegion::VARIANTS);
            } else if region_str.eq_ignore_ascii_case("americas") {
                regions_to_ingest.extend(LolRegion::AMERICAS);
            } else if region_str.eq_ignore_ascii_case("asia") {
                regions_to_ingest.extend(LolRegion::ASIA);
            } else if region_str.eq_ignore_ascii_case("europe") {
                regions_to_ingest.extend(LolRegion::EUROPE);
            } else {
                let region = LolRegion::from_str(&region_str)
                    .with_context(|| format!("invalid region: `{region_str}`"))?;
                regions_to_ingest.insert(region);
            }
        }
        let regions_to_ingest = regions_to_ingest.into_iter().collect::<Vec<_>>();
        Ok(Self {
            inner: Arc::new(IngestInner {
                regions_to_ingest: regions_to_ingest.into_boxed_slice(),
                task_runner: ScheduledTaskRunner::new(storage.clone()),
                storage,
                swain,
            }),
        })
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        tracing::info!("running ingest...");
        self.init_region_ingest_tasks()
            .await
            .context("error while initializing region ingest tasks")?;
        self.ask(RunScheduledTasks)
            .await
            .context("error while running scheduled tasks")
    }

    #[instrument(skip(self, task), fields(task_name = task.name))]
    async fn run_task(&self, task: ScheduledTask) -> anyhow::Result<()> {
        Ok(())
    }

    pub async fn ask<R>(&self, request: R) -> anyhow::Result<R::Output>
    where
        R: IngestRequest,
    {
        let request_name = std::any::type_name::<R>();
        let ingest = self.clone();
        let join_handle = tokio::task::spawn(async move { request.run(ingest).await });
        let join_result = join_handle
            .await
            .with_context(|| format!("error while joining task for request: {request_name}"))?;
        join_result.with_context(|| format!("error while running request: {request_name}"))
    }

    pub fn tell<R>(&self, request: R)
    where
        R: IngestRequest,
    {
        let request_name = std::any::type_name::<R>();
        let ingest = self.clone();
        tokio::task::spawn(async move {
            if let Err(err) = request.run(ingest).await {
                tracing::error!(%request_name, ?err, "error while running request");
            }
        });
    }

    #[tracing::instrument(skip(self))]
    async fn init_region_ingest_tasks(&self) -> anyhow::Result<()> {
        for &region in self.inner.regions_to_ingest.iter() {
            for rank in LolRank::ALL {
                let (tier, division) = rank.parts();
                let task_name =
                    format!("periodically-ingest-leagues/{region:#}/{tier:#}/{division}");
                self.task_runner
                    .init_task(&task_name, "@every: 1h")
                    .await
                    .context("error while initializing task")?;
                let make_request = move || PeriodicallyIngestLeague { region, rank };
                self.task_runner.register(task_name, self, make_request);
            }
        }
        Ok(())
    }
}

pub struct IngestInner {
    pub(crate) storage: Storage,
    pub(crate) swain: Swain,
    task_runner: ScheduledTaskRunner,
    regions_to_ingest: Box<[LolRegion]>,
}

impl std::ops::Deref for Ingest {
    type Target = IngestInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct IngestConfig {
    pub regions_to_ingest: Vec<String>,
}

pub trait IngestRequest: 'static + Send {
    type Output: Send;

    fn run(self, ingest: Ingest) -> impl Future<Output = anyhow::Result<Self::Output>> + Send;
}
