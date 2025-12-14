pub mod error;
pub mod postgres;

pub use crate::{
    error::{Result, StoreError},
    postgres::PgStore,
};
