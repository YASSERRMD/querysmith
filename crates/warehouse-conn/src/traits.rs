use async_trait::async_trait;
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableSchema {
    pub name: String,
    pub columns: Vec<Column>,
    pub primary_key: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub row_count: usize,
}

#[async_trait]
pub trait Warehouse: Send + Sync {
    async fn connect(&self) -> Result<(), Error>;
    async fn disconnect(&self) -> Result<(), Error>;
    async fn execute(&self, sql: &str) -> Result<QueryResult, Error>;
    async fn get_schema(&self, table_name: &str) -> Result<TableSchema, Error>;
    async fn list_tables(&self) -> Result<Vec<String>, Error>;
    async fn preview_table(&self, table_name: &str, limit: usize) -> Result<QueryResult, Error>;
}
