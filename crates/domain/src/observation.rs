use chrono::{DateTime, Utc};

use crate::{
    error::{Result, ValidationError},
    ids::ObservationId,
};

#[derive(Debug, Default)]
pub enum SourceKind {
    Rss,
    Pdf,
    Web,
    Text,
    Manual,
    #[default]
    Unknown,
}

impl SourceKind {
    pub const fn as_str(&self) -> &str {
        match self {
            SourceKind::Rss => "rss",
            SourceKind::Pdf => "pdf",
            SourceKind::Web => "web",
            SourceKind::Text => "text",
            SourceKind::Manual => "manual",
            SourceKind::Unknown => "unknown",
        }
    }
}

pub struct Observation {
    id: ObservationId,
    content_hash: String,
    content: String,
    title: Option<String>,
    source_url: Option<String>,
    source_kind: SourceKind,
    created_at: DateTime<Utc>,
    published_at: Option<DateTime<Utc>>,
}

impl Observation {
    #[must_use]
    pub fn builder() -> ObservationBuilder {
        ObservationBuilder::default()
    }
}

#[derive(Debug, Default)]
pub struct ObservationBuilder {
    content: Option<String>,
    title: Option<String>,
    source_url: Option<String>,
    source_kind: SourceKind,
    published_at: Option<DateTime<Utc>>,
    // Allow pre-setting ID for testing or reconstruction
    id: Option<ObservationId>,
    created_at: Option<DateTime<Utc>>,
}

impl ObservationBuilder {
    #[must_use]
    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = Some(content.into());
        self
    }

    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    #[must_use]
    pub fn source_url(mut self, url: impl Into<String>) -> Self {
        self.source_url = Some(url.into());
        self
    }

    #[must_use]
    pub const fn source_kind(mut self, kind: SourceKind) -> Self {
        self.source_kind = kind;
        self
    }

    #[must_use]
    pub const fn published_at(mut self, published: DateTime<Utc>) -> Self {
        self.published_at = Some(published);
        self
    }

    #[must_use]
    pub const fn with_id(mut self, id: ObservationId) -> Self {
        self.id = Some(id);
        self
    }

    #[must_use]
    pub const fn with_created_at(mut self, created_at: DateTime<Utc>) -> Self {
        self.created_at = Some(created_at);
        self
    }

    pub fn build(self) -> Result<Observation> {
        let contet = self
            .content
            .ok_or_else(|| ValidationError::missing_field("content"))?;

        Ok(Observation {
            id: self.id.unwrap_or_default(),
            content_hash: "".to_string(),
            content: contet,
            title: self.title,
            source_url: self.source_url,
            source_kind: self.source_kind,
            created_at: self.created_at.unwrap_or_else(Utc::now),
            published_at: self.published_at,
        })
    }
}
