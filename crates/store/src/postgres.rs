use chrono::{DateTime, Utc};
use domain::{
    chunk::Chunk,
    ids::{ChunkId, ObservationId},
    observation::{Observation, SourceKind},
};
use sqlx::{PgPool, Row, postgres::PgPoolOptions};
use uuid::Uuid;

use crate::error::{Result, StoreError};

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

pub struct PgStore {
    pool: PgPool,
}

impl PgStore {
    pub async fn connect(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> Result<()> {
        MIGRATOR.run(&self.pool).await?;
        Ok(())
    }

    #[must_use]
    pub const fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn upsert_observation(
        &self,
        observation: &Observation,
    ) -> Result<(ObservationId, bool)> {
        let content_hash = observation.content_hash().to_hex();

        let inserted_id: Option<Uuid> = sqlx::query_scalar(
            r#"
INSERT INTO observations (
    id,
    content_hash,
    content,
    title,
    source_url,
    source_kind,
    created_at,
    published_at
)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
ON CONFLICT (content_hash) DO NOTHING
RETURNING id
            "#,
        )
        .bind(observation.id().into_inner())
        .bind(&content_hash)
        .bind(observation.content())
        .bind(observation.title())
        .bind(observation.source_url())
        .bind(observation.source_kind().as_str())
        .bind(observation.created_at())
        .bind(observation.published_at())
        .fetch_optional(&self.pool)
        .await?;

        if let Some(id) = inserted_id {
            return Ok((ObservationId::from_raw(id), true));
        }

        let existing_id: Uuid =
            sqlx::query_scalar("SELECT id FROM observations WHERE content_hash = $1")
                .bind(&content_hash)
                .fetch_one(&self.pool)
                .await?;

        Ok((ObservationId::from_raw(existing_id), false))
    }

    pub async fn get_observation(&self, id: ObservationId) -> Result<Option<Observation>> {
        let row = sqlx::query(
            r#"
SELECT
    id,
    content,
    title,
    source_url,
    source_kind,
    created_at,
    published_at
FROM observations
WHERE id = $1
            "#,
        )
        .bind(id.into_inner())
        .fetch_optional(&self.pool)
        .await?;

        let Some(row) = row else {
            return Ok(None);
        };

        let id: Uuid = row.try_get("id")?;
        let content: String = row.try_get("content")?;
        let title: Option<String> = row.try_get("title")?;
        let source_url: Option<String> = row.try_get("source_url")?;
        let source_kind: String = row.try_get("source_kind")?;
        let created_at: DateTime<Utc> = row.try_get("created_at")?;
        let published_at: Option<DateTime<Utc>> = row.try_get("published_at")?;

        let mut builder = Observation::builder()
            .with_id(ObservationId::from_raw(id))
            .with_created_at(created_at)
            .source_kind(SourceKind::parse(&source_kind))
            .content(content);

        if let Some(title) = title {
            builder = builder.title(title);
        }

        if let Some(source_url) = source_url {
            builder = builder.source_url(source_url);
        }

        if let Some(published_at) = published_at {
            builder = builder.published_at(published_at);
        }

        Ok(Some(builder.build()?))
    }

    pub async fn upsert_chunks(&self, chunks: &[Chunk]) -> Result<u64> {
        let mut tx = self.pool.begin().await?;
        let mut affected_total = 0u64;

        for chunk in chunks {
            let start_offset = i64::try_from(chunk.start_offset())
                .map_err(|_| StoreError::OutOfRange("start_offset"))?;
            let end_offset = i64::try_from(chunk.end_offset())
                .map_err(|_| StoreError::OutOfRange("end_offset"))?;
            let token_estimate = i32::try_from(chunk.token_estimate())
                .map_err(|_| StoreError::OutOfRange("token_estimate"))?;

            let result = sqlx::query(
                r#"
INSERT INTO chunks (
    id,
    observation_id,
    chunk_index,
    text,
    start_offset,
    end_offset,
    token_estimate
)
VALUES ($1, $2, $3, $4, $5, $6, $7)
ON CONFLICT (observation_id, chunk_index) DO UPDATE SET
    text = EXCLUDED.text,
    start_offset = EXCLUDED.start_offset,
    end_offset = EXCLUDED.end_offset,
    token_estimate = EXCLUDED.token_estimate
                "#,
            )
            .bind(chunk.id().into_inner())
            .bind(chunk.observation_id().into_inner())
            .bind(chunk.index())
            .bind(chunk.text())
            .bind(start_offset)
            .bind(end_offset)
            .bind(token_estimate)
            .execute(&mut *tx)
            .await?;

            affected_total += result.rows_affected();
        }

        tx.commit().await?;
        Ok(affected_total)
    }

    pub async fn list_chunks(&self, observation_id: ObservationId) -> Result<Vec<Chunk>> {
        let rows = sqlx::query(
            r#"
SELECT
    id,
    observation_id,
    chunk_index,
    text,
    start_offset,
    end_offset,
    token_estimate
FROM chunks
WHERE observation_id = $1
ORDER BY chunk_index ASC
            "#,
        )
        .bind(observation_id.into_inner())
        .fetch_all(&self.pool)
        .await?;

        let mut chunks = Vec::with_capacity(rows.len());

        for row in rows {
            let id: Uuid = row.try_get("id")?;
            let observation_id: Uuid = row.try_get("observation_id")?;
            let chunk_index: i32 = row.try_get("chunk_index")?;
            let text: String = row.try_get("text")?;
            let start_offset: i64 = row.try_get("start_offset")?;
            let end_offset: i64 = row.try_get("end_offset")?;
            let token_estimate: i32 = row.try_get("token_estimate")?;

            let start_offset = usize::try_from(start_offset)
                .map_err(|_| StoreError::OutOfRange("start_offset"))?;
            let end_offset =
                usize::try_from(end_offset).map_err(|_| StoreError::OutOfRange("end_offset"))?;
            let token_estimate = u32::try_from(token_estimate)
                .map_err(|_| StoreError::OutOfRange("token_estimate"))?;

            chunks.push(Chunk::reconstruct(
                ChunkId::from_raw(id),
                ObservationId::from_raw(observation_id),
                chunk_index,
                text,
                start_offset,
                end_offset,
                token_estimate,
            ));
        }

        Ok(chunks)
    }
}
