use anyhow::Context as _;
use chrono::{DateTime, Utc};

use crate::misc::is_unique_constraint_violation;

pub struct ScheduledTaskStorage {
    pg_pool: sqlx::Pool<sqlx::Postgres>,
}

impl ScheduledTaskStorage {
    pub fn new(pg_pool: sqlx::Pool<sqlx::Postgres>) -> Self {
        Self { pg_pool }
    }

    pub async fn create(
        &self,
        new: NewScheduledTask,
    ) -> Result<ScheduledTask, CreateScheduledTaskError> {
        sqlx::query_as!(
            ScheduledTask,
            r#"
                INSERT INTO scheduled_task (name, schedule, enabled)
                VALUES ($1, $2, $3)
                RETURNING name, schedule, enabled, last_run_at, next_run_at;
            "#,
            new.name,
            new.schedule,
            new.enabled
        )
        .fetch_one(&self.pg_pool)
        .await
        .map_err(|err| match err {
            err if is_unique_constraint_violation(&err) => {
                CreateScheduledTaskError::Duplicate { name: new.name }
            }
            err => CreateScheduledTaskError::Unknown(err.into()),
        })
    }

    pub async fn exists(&self, name: &str) -> Result<bool, FindScheduledTaskError> {
        sqlx::query_scalar!(
            "SELECT EXISTS (SELECT 1 FROM scheduled_task WHERE name = $1)",
            name
        )
        .fetch_one(&self.pg_pool)
        .await
        .context("error while checking if scheduled task exists")
        .map_err(Into::into)
        .map(|exists| exists.unwrap_or(false))
    }

    pub async fn find_by_name(
        &self,
        name: &str,
    ) -> Result<Option<ScheduledTask>, FindScheduledTaskError> {
        sqlx::query_as!(
            ScheduledTask,
            r#"
                SELECT
                    name, schedule, enabled, last_run_at, next_run_at
                FROM scheduled_task
                WHERE name = $1
            "#,
            name
        )
        .fetch_optional(&self.pg_pool)
        .await
        .context("error while finding scheduled task by name")
        .map_err(Into::into)
    }

    pub async fn next_task(&self) -> Result<Option<ScheduledTask>, FindScheduledTaskError> {
        sqlx::query_as!(
            ScheduledTask,
            r#"
                SELECT
                    name, schedule, enabled, last_run_at, next_run_at
                FROM scheduled_task
                WHERE next_run_at IS NULL OR next_run_at <= NOW()
                ORDER BY next_run_at ASC NULLS FIRST
                LIMIT 1
            "#,
        )
        .fetch_optional(&self.pg_pool)
        .await
        .context("error while finding next scheduled task")
        .map_err(Into::into)
    }

    pub async fn update_next_run_at(
        &self,
        name: &str,
        next_run_at: DateTime<Utc>,
    ) -> Result<(), UpdateScheduledTaskError> {
        sqlx::query!(
            r#"
                UPDATE scheduled_task
                SET next_run_at = $1
                WHERE name = $2
            "#,
            next_run_at,
            name
        )
        .execute(&self.pg_pool)
        .await
        .context("error while updating next run at")
        .map_err(Into::into)
        .and_then(|result| {
            if result.rows_affected() == 0 {
                Err(UpdateScheduledTaskError::TaskNotFound { name: name.into() })
            } else {
                Ok(())
            }
        })
    }
}

pub struct NewScheduledTask {
    pub name: String,
    pub schedule: String,
    pub enabled: bool,
    pub next_run_at: DateTime<Utc>,
}

pub struct ScheduledTask {
    pub name: String,
    pub schedule: String,
    pub enabled: bool,
    pub last_run_at: Option<DateTime<Utc>>,
    pub next_run_at: Option<DateTime<Utc>>,
}

#[derive(Debug, thiserror::Error)]
pub enum CreateScheduledTaskError {
    #[error("scheduled task with name {name} already exists")]
    Duplicate { name: String },
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum FindScheduledTaskError {
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateScheduledTaskError {
    #[error("scheduled task with name {name} does not exist")]
    TaskNotFound { name: String },
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
