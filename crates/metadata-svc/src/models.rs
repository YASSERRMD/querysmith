use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    pub id: Option<i64>,
    pub name: String,
    pub source: String,
    pub tables: Vec<TableMetadata>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableMetadata {
    pub name: String,
    pub schema_name: Option<String>,
    pub columns: Vec<ColumnMetadata>,
    pub primary_key: Option<Vec<String>>,
    pub annotations: Vec<Annotation>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnMetadata {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    pub key: String,
    pub value: String,
    pub source: Option<String>,
}
