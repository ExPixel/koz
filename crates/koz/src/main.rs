mod config;

use std::{net::SocketAddr, process::ExitCode};

use anyhow::Context as _;
use koz_ingest::{Ingest, IngestConfig};
use koz_storage::Storage;
use swain::Swain;
use tokio::{runtime::Runtime, task::JoinSet};

pub fn main() -> ExitCode {
    if let Err(err) = main_internal() {
        eprintln!("exited with error: {err:?}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn main_internal() -> anyhow::Result<()> {
    let loaded_config_paths = config::load_dotenv().context("error while loading dotenv")?;
    init_tracing().context("error while initializing tracing")?;

    if loaded_config_paths.is_empty() {
        tracing::warn!("no config files loaded");
    }

    for path in loaded_config_paths {
        tracing::info!("loaded config file: {path:?}");
    }

    let runtime = init_runtime().context("error while initializing tokio runtime")?;
    runtime.block_on(run()).context("error while running koz")
}

async fn run() -> anyhow::Result<()> {
    tracing::info!("running koz");

    let storage = init_storage()
        .await
        .context("error while initializing storage")?;
    tracing::info!("initialized storage");

    let riot_api_key: String = config::parse_opt_required("KOZ_RIOT_API_KEY")?;
    let swain = Swain::new("koz/0.1.0".to_owned(), riot_api_key);

    let mut tasks = JoinSet::<anyhow::Result<()>>::new();

    {
        let storage = storage.clone();
        tasks.spawn(async move {
            let ingest_config = init_ingest_config()
                .await
                .context("error while initializing ingest config")?;
            let ingest = Ingest::new(ingest_config, storage, swain)
                .context("error while initializing ingest")?;
            ingest.run().await.context("error while running ingest")?;
            Ok(())
        });
    }

    {
        let storage = storage.clone();
        let web_config = init_web_config()
            .await
            .context("error while initializing web config")?;
        tasks.spawn(async move {
            koz_web::run(web_config, storage)
                .await
                .context("error while running web")?;
            Ok(())
        });
    }

    while let Some(join_result) = tasks.join_next().await {
        let task_result = join_result?;

        if let Err(err) = task_result {
            tracing::error!(?err, "error while running task");
        }
    }

    Ok(())
}

async fn init_ingest_config() -> anyhow::Result<IngestConfig> {
    let regions_to_ingest_str: Option<String> = config::parse_opt("KOZ_REGION_INGEST")?;
    let regions_to_ingest = regions_to_ingest_str
        .unwrap_or_default()
        .split(',')
        .filter(|s| !s.is_empty())
        .map(|s| s.trim().to_lowercase())
        .collect();
    let ingest_config = IngestConfig { regions_to_ingest };
    Ok(ingest_config)
}

async fn init_web_config() -> anyhow::Result<koz_web::WebConfig> {
    let address: SocketAddr = config::parse_opt_required("KOZ_WEB_ADDRESS")?;
    let web_config = koz_web::WebConfig { address };
    Ok(web_config)
}

fn init_runtime() -> anyhow::Result<Runtime> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("error while building tokio runtime")
}

async fn init_storage() -> anyhow::Result<Storage> {
    let database_url: String = config::parse_opt_required("KOZ_DATABASE_URL")?;
    let database_max_connections: Option<u32> = config::parse_opt("KOZ_DATABASE_MAX_CONNECTIONS")?;

    let mut storage_builder = Storage::builder();
    storage_builder = storage_builder.database_url(database_url);
    if let Some(database_max_connections) = database_max_connections {
        storage_builder = storage_builder.database_max_connections(database_max_connections);
    }

    let storage = storage_builder
        .build()
        .await
        .context("error while building storage")?;

    Ok(storage)
}

fn init_tracing() -> anyhow::Result<()> {
    use tracing_subscriber::layer::SubscriberExt as _;

    let enable_log_ansi: bool = config::parse_opt("KOZ_LOG_ANSI")?.unwrap_or(true);
    let filter: String = config::parse_opt("KOZ_LOG_FILTER")?.unwrap_or_else(|| "info".to_owned());
    let filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(tracing_subscriber::filter::LevelFilter::ERROR.into())
        .parse(filter)
        .context("error while parsing tracing env filter")?;
    let stderr_layer = tracing_subscriber::fmt::layer()
        .with_ansi(enable_log_ansi)
        .compact()
        .with_writer(std::io::stderr);
    let layers = tracing_subscriber::registry()
        .with(stderr_layer)
        .with(filter);
    tracing::subscriber::set_global_default(layers)
        .context("error while installing global tracing subscriber")?;
    Ok(())
}
