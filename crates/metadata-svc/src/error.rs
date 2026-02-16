use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Metadata error: {0}")]
    Metadata(String),
    #[error("Database error: {0}")]
    Database(String),
    #[error("Not found: {0}")]
    NotFound(String),
}
