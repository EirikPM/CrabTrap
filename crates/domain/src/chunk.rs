use crate::ids::{ChunkId, ObservationId};

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

pub struct Chunk {
    id: ChunkId,
    observation_id: ObservationId,
    index: usize,
    text: String,
    start_offset: usize,
    end_offset: usize,
}

// pub struct Chunk<S: ChunkState> {
//     id: ChunkId,
//     observation_id: ObservationId,
//     index: usize,
//     total_chunks: usize,
//     text: String,
//     start_offset: usize,
//     end_offset: usize, // embedding_data: Option<Embedding>
// }
