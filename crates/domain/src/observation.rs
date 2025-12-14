use chrono::{DateTime, Utc};

use crate::{
    chunk::Chunk,
    error::{Result, ValidationError},
    ids::{ContentHash, ObservationId},
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
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

    #[must_use]
    pub fn parse(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "rss" => Self::Rss,
            "pdf" => Self::Pdf,
            "web" => Self::Web,
            "text" => Self::Text,
            "manual" => Self::Manual,
            "unknown" => Self::Unknown,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Observation {
    id: ObservationId,
    content_hash: ContentHash,
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

    pub fn from_content(content: impl Into<String>) -> Result<Self> {
        Self::builder().content(content).build()
    }

    #[must_use]
    pub const fn id(&self) -> ObservationId {
        self.id
    }

    #[must_use]
    pub const fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    #[must_use]
    pub const fn published_at(&self) -> Option<DateTime<Utc>> {
        self.published_at
    }

    #[must_use]
    pub const fn source_kind(&self) -> SourceKind {
        self.source_kind
    }

    #[must_use]
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    #[must_use]
    pub fn source_url(&self) -> Option<&str> {
        self.source_url.as_deref()
    }

    #[must_use]
    pub const fn content_hash(&self) -> &ContentHash {
        &self.content_hash
    }

    #[must_use]
    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn chunk(&self, chunk_size: usize) -> Result<Vec<Chunk>> {
        if chunk_size == 0 {
            return Err(ValidationError::InvalidChunkSize.into());
        }

        let mut chunks = Vec::new();
        let text = self.content.as_str();

        let mut start = 0usize;
        let mut index = 0i32;

        while start < text.len() {
            let mut end = (start + chunk_size).min(text.len());
            while end > start && !text.is_char_boundary(end) {
                end -= 1;
            }

            if end == start {
                end = (start + chunk_size).min(text.len());
                while end < text.len() && !text.is_char_boundary(end) {
                    end += 1;
                }
            }

            let chunk_text = &text[start..end];
            chunks.push(Chunk::new(self, index, chunk_text, start, end));

            start = end;
            index += 1;
        }

        Ok(chunks)
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
        let content = self
            .content
            .ok_or_else(|| ValidationError::missing_field("content"))?;

        if content.trim().is_empty() {
            return Err(ValidationError::EmptyContent.into());
        }

        Ok(Observation {
            id: self.id.unwrap_or_default(),
            content_hash: ContentHash::from_content(&content),
            content,
            title: self.title,
            source_url: self.source_url,
            source_kind: self.source_kind,
            created_at: self.created_at.unwrap_or_else(Utc::now),
            published_at: self.published_at,
        })
    }
}
