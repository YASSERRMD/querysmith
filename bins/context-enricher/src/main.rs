use anyhow::Result;
use metadata_svc::{MetadataService, Schema, TableMetadata, Annotation};
use metadata_svc::models::ColumnMetadata;
use rag_engine::VectorIndex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableContext {
    pub table_name: String,
    pub schema_name: Option<String>,
    pub description: String,
    pub columns: Vec<ColumnInfo>,
    pub primary_key: Option<Vec<String>>,
    pub annotations: Vec<Annotation>,
    pub lineage_dependencies: Vec<String>,
    pub sample_values: Vec<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub comment: Option<String>,
    pub sample_values: Vec<serde_json::Value>,
}

pub struct ContextEnricher {
    metadata: MetadataService,
    #[allow(dead_code)]
    vector_index: Option<VectorIndex>,
}

impl ContextEnricher {
    pub fn new() -> Self {
        Self {
            metadata: MetadataService::new(),
            vector_index: None,
        }
    }

    pub async fn load_from_warehouse(
        &self,
        warehouse: &dyn warehouse_conn::Warehouse,
    ) -> Result<Schema> {
        info!("Loading schema from warehouse");
        
        let table_names = warehouse.list_tables().await?;
        info!("Found {} tables", table_names.len());

        let mut tables = Vec::new();
        
        for table_name in table_names {
            let schema = warehouse.get_schema(&table_name).await?;
            
            let columns: Vec<ColumnMetadata> = schema
                .columns
                .iter()
                .map(|c| ColumnMetadata {
                    name: c.name.clone(),
                    data_type: c.data_type.clone(),
                    nullable: c.nullable,
                    comment: c.comment.clone(),
                })
                .collect();

            let sample_result = warehouse.preview_table(&table_name, 5).await?;
            let _sample_values: Vec<HashMap<String, serde_json::Value>> = sample_result
                .rows
                .iter()
                .map(|row| {
                    sample_result
                        .columns
                        .iter()
                        .enumerate()
                        .map(|(i, col)| (col.clone(), row.get(i).cloned().unwrap_or(serde_json::Value::Null)))
                        .collect()
                })
                .collect();

            let table_meta = TableMetadata {
                name: table_name.clone(),
                schema_name: Some("public".to_string()),
                columns,
                primary_key: schema.primary_key,
                annotations: vec![],
                description: None,
            };

            let _ = self.metadata.add_table("main", table_meta).await;
            tables.push(TableMetadata {
                name: table_name,
                schema_name: Some("public".to_string()),
                columns: vec![],
                primary_key: None,
                annotations: vec![],
                description: None,
            });
        }

        let full_schema = Schema {
            id: Some(1),
            name: "main".to_string(),
            source: "warehouse".to_string(),
            tables,
            created_at: None,
            updated_at: None,
        };

        Ok(full_schema)
    }

    pub async fn generate_table_context(
        &self,
        table_name: &str,
        schema_name: Option<&str>,
    ) -> Result<TableContext> {
        let schema = schema_name.unwrap_or("main");
        let table = self.metadata.get_table(schema, table_name).await?;
        
        let lineage_deps = self.metadata.get_table_dependencies(table_name).await.unwrap_or_default();
        
        let description = self.generate_table_description(&table);

        let columns: Vec<ColumnInfo> = table
            .columns
            .iter()
            .map(|c| ColumnInfo {
                name: c.name.clone(),
                data_type: c.data_type.clone(),
                nullable: c.nullable,
                comment: c.comment.clone(),
                sample_values: vec![],
            })
            .collect();

        Ok(TableContext {
            table_name: table.name.clone(),
            schema_name: table.schema_name.clone(),
            description,
            columns,
            primary_key: table.primary_key,
            annotations: table.annotations,
            lineage_dependencies: lineage_deps,
            sample_values: vec![],
        })
    }

    fn generate_table_description(&self, table: &TableMetadata) -> String {
        let mut desc = format!("Table: {}", table.name);
        
        if let Some(ref desc_text) = table.description {
            desc.push_str(&format!("\nDescription: {}", desc_text));
        }

        if !table.columns.is_empty() {
            desc.push_str("\nColumns:");
            for col in &table.columns {
                let _nullable = if col.nullable { "NULL" } else { "NOT NULL" };
                let comment = col.comment.as_ref().map(|c| format!(" - {}", c)).unwrap_or_default();
                desc.push_str(&format!("\n  - {} ({}) {}", col.name, col.data_type, comment));
            }
        }

        if let Some(ref pk) = table.primary_key {
            desc.push_str(&format!("\nPrimary Key: {}", pk.join(", ")));
        }

        if !table.annotations.is_empty() {
            desc.push_str("\nAnnotations:");
            for ann in &table.annotations {
                desc.push_str(&format!("\n  {}: {}", ann.key, ann.value));
            }
        }

        desc
    }

    pub async fn generate_all_contexts(&self) -> Result<Vec<TableContext>> {
        let schema = self.metadata.get_schema("main").await?;
        let mut contexts = Vec::new();

        for table in &schema.tables {
            match self.generate_table_context(&table.name, table.schema_name.as_deref()).await {
                Ok(ctx) => contexts.push(ctx),
                Err(e) => error!("Failed to generate context for {}: {}", table.name, e),
            }
        }

        Ok(contexts)
    }

    pub fn to_context_blob(&self, contexts: &[TableContext]) -> String {
        contexts
            .iter()
            .map(|ctx| {
                let mut blob = format!("# Table: {}\n", ctx.table_name);
                if let Some(ref schema) = ctx.schema_name {
                    blob.push_str(&format!("Schema: {}\n", schema));
                }
                blob.push_str(&format!("{}\n", ctx.description));
                blob
            })
            .collect::<Vec<_>>()
            .join("\n---\n\n")
    }
}

impl Default for ContextEnricher {
    fn default() -> Self {
        Self::new()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    info!("Context Enricher starting...");

    let _enricher = ContextEnricher::new();
    
    println!("Context enricher initialized");
    println!("Usage:");
    println!("  - Load schema from warehouse and generate contexts");
    println!("  - Output context blob for RAG indexing");

    Ok(())
}
