mod misc;
pub mod scheduled_task;

use std::sync::Arc;

use anyhow::Context as _;
use scheduled_task::ScheduledTaskStorage;

#[derive(Clone)]
pub struct Storage {
    inner: Arc<StorageInner>,
}

impl Storage {
    pub fn builder() -> StorageBuilder {
        StorageBuilder::new()
    }
}

pub struct StorageInner {
    pub scheduled_task: ScheduledTaskStorage,
}

impl std::ops::Deref for Storage {
    type Target = StorageInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(Default)]
pub struct StorageBuilder {
    database_url: Option<String>,
    database_max_connections: Option<u32>,
}

impl StorageBuilder {
    const DEFAULT_DATABASE_MAX_CONNECTIONS: u32 = 4;

    pub fn new() -> Self {
        Self::default()
    }

    pub fn database_url(mut self, database_url: impl Into<String>) -> Self {
        self.database_url = Some(database_url.into());
        self
    }

    pub fn database_max_connections(mut self, database_max_connections: u32) -> Self {
        self.database_max_connections = Some(database_max_connections);
        self
    }

    pub async fn build(self) -> anyhow::Result<Storage> {
        let database_url = self
            .database_url
            .ok_or_else(|| anyhow::anyhow!("database url not set"))?;
        let database_max_connections = self
            .database_max_connections
            .unwrap_or(Self::DEFAULT_DATABASE_MAX_CONNECTIONS);

        let opts = sqlx::postgres::PgPoolOptions::new().max_connections(database_max_connections);
        let pool = opts
            .connect(&database_url)
            .await
            .context("error while connecting to sql")?;

        let storage_inner = StorageInner {
            scheduled_task: ScheduledTaskStorage::new(pool.clone()),
        };

        let storage = Storage {
            inner: Arc::new(storage_inner),
        };

        Ok(storage)
    }
}
