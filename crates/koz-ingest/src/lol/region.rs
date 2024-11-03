use std::time::Duration;

use anyhow::Context as _;
use koz_types::lol::{LolRank, LolRegion};

use crate::{Ingest, IngestRequest};

pub struct PeriodicallyIngestLeague {
    pub(crate) region: LolRegion,
    pub(crate) rank: LolRank,
}

impl IngestRequest for PeriodicallyIngestLeague {
    type Output = ();

    #[tracing::instrument(skip(self, ingest), fields(region = %self.region, rank = %self.rank))]
    async fn run(self, ingest: Ingest) -> anyhow::Result<()> {
        let Self { region, rank } = self;

        let delay = Duration::from_secs(3600);
        loop {
            tracing::debug!("starting ingest of league: {region}/{rank}");
            ingest
                .ask(IngestLeagueByRank { region, rank })
                .await
                .with_context(|| format!("error ingesting league: {region}/{rank}"))?;
            tokio::time::sleep(delay).await;
        }
    }
}

pub struct IngestLeagueByRank {
    region: LolRegion,
    rank: LolRank,
}

impl IngestRequest for IngestLeagueByRank {
    type Output = ();

    #[tracing::instrument(skip(self, ingest), fields(region = %self.region, rank = %self.rank))]
    async fn run(self, ingest: Ingest) -> anyhow::Result<()> {
        // let Self { region, rank } = self;

        let _storage = ingest.storage.clone();
        let _swain = ingest.swain.clone();

        Ok(())
    }
}
