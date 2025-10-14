use crate::error::Error;

pub mod health;
pub mod sse;

pub use health::route as health;
pub use sse::route as sse;

pub async fn not_found() -> Error {
    Error::NotFound
}
