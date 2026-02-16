use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Warehouse error: {0}")]
    Warehouse(String),
    #[error("Connection error: {0}")]
    Connection(String),
    #[error("Query error: {0}")]
    Query(String),
}
