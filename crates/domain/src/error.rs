use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Validation(#[from] ValidationError),

    #[error(transparent)]
    Observation(#[from] ObservationError),
}

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Content cannot be empty")]
    EmptyContent,

    #[error("content too large: {size} bytes (max: {max}")]
    ContentTooLarge { size: usize, max: usize },

    #[error("missing required field: {field}")]
    MissingField { field: &'static str },

    #[error("missing required field: {field}")]
    EmptyField { field: &'static str },
}

#[derive(Debug, Error)]
pub enum ObservationError {
    #[error("duplicate observation with content hash {hash}")]
    Duplicate { hash: String },
}

impl ValidationError {
    #[must_use]
    pub const fn missing_field(field: &'static str) -> Self {
        Self::MissingField { field }
    }
}
