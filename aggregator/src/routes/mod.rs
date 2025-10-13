use crate::error::Error;

pub mod health;
pub mod post;

pub use health::route as health;
pub use post::route as post;

pub async fn not_found() -> Error {
    Error::NotFound
}
