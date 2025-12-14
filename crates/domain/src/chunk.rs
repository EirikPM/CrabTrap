use crate::{
    ids::{ChunkId, ObservationId},
    observation::Observation,
};

// mod private {
//     pub trait Sealed {}
// }

// pub trait ChunkState: private::Sealed + Send + Sync + 'static {
//     const STATE_NAME: &'static str;
// }

// #[derive(Debug, Clone, Copy, Default)]
// pub struct Raw;

// impl private::Sealed for Raw {}
// impl ChunkState for Raw {
//     const STATE_NAME: &'static str = "raw";
// }

// #[derive(Debug, Clone, Copy, Default)]
// pub struct Embedded;

// impl private::Sealed for Embedded {}
// impl ChunkState for Embedded {
//     const STATE_NAME: &'static str = "embedded";
// }

// pub struct Chunk<S: ChunkState> {
//     id: ChunkId,
//     observation_id: ObservationId,
//     index: usize,
//     total_chunks: usize,
//     text: String,
//     start_offset: usize,
//     end_offset: usize, // embedding_data: Option<Embedding>
// }

pub struct Chunk {
    pub(crate) id: ChunkId,
    pub(crate) observation_id: ObservationId,
    pub(crate) index: i32,
    pub(crate) text: String,
    pub(crate) start_offset: usize,
    pub(crate) end_offset: usize,
    pub(crate) token_estimate: u32,
}

impl Chunk {
    #[must_use]
    pub const fn reconstruct(
        id: ChunkId,
        observation_id: ObservationId,
        index: i32,
        text: String,
        start_offset: usize,
        end_offset: usize,
        token_estimate: u32,
    ) -> Self {
        Self {
            id,
            observation_id,
            index,
            text,
            start_offset,
            end_offset,
            token_estimate,
        }
    }

    pub fn new(
        observation: &Observation,
        index: i32,
        text: impl Into<String>,
        start: usize,
        end: usize,
    ) -> Self {
        Self {
            id: ChunkId::new(),
            observation_id: observation.id(),
            index,
            text: text.into(),
            start_offset: start,
            end_offset: end,
            token_estimate: 1,
        }
    }

    #[must_use]
    pub const fn id(&self) -> ChunkId {
        self.id
    }

    #[must_use]
    pub const fn observation_id(&self) -> ObservationId {
        self.observation_id
    }

    #[must_use]
    pub const fn index(&self) -> i32 {
        self.index
    }

    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    #[must_use]
    pub const fn start_offset(&self) -> usize {
        self.start_offset
    }

    #[must_use]
    pub const fn end_offset(&self) -> usize {
        self.end_offset
    }

    #[must_use]
    pub const fn token_estimate(&self) -> u32 {
        self.token_estimate
    }
}
