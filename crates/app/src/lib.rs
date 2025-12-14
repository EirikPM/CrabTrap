use domain::{
    chunk::Chunk,
    ids::ObservationId,
    observation::{Observation, SourceKind},
};
use store::{PgStore, Result};

pub struct App {
    store: PgStore,
}

impl App {
    pub async fn connect(database_url: &str) -> Result<Self> {
        let store = PgStore::connect(database_url).await?;
        Ok(Self { store })
    }

    pub async fn migrate(&self) -> Result<()> {
        self.store.migrate().await
    }

    pub async fn ingest_text(
        &self,
        content: String,
        title: Option<String>,
        source_url: Option<String>,
    ) -> Result<(ObservationId, bool)> {
        let mut builder = Observation::builder()
            .content(content)
            .source_kind(SourceKind::Text);

        if let Some(title) = title {
            builder = builder.title(title);
        }

        if let Some(source_url) = source_url {
            builder = builder.source_url(source_url);
        }

        let observation = builder.build()?;
        self.store.upsert_observation(&observation).await
    }

    pub async fn get_observation(&self, id: ObservationId) -> Result<Option<Observation>> {
        self.store.get_observation(id).await
    }

    pub async fn chunk_observation(
        &self,
        id: ObservationId,
        chunk_size: usize,
    ) -> Result<Option<usize>> {
        let Some(observation) = self.store.get_observation(id).await? else {
            return Ok(None);
        };

        let chunks = observation.chunk(chunk_size)?;
        self.store.upsert_chunks(&chunks).await?;
        Ok(Some(chunks.len()))
    }

    pub async fn list_chunks(&self, observation_id: ObservationId) -> Result<Vec<Chunk>> {
        self.store.list_chunks(observation_id).await
    }
}
