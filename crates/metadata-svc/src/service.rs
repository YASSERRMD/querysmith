use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::Error;
use crate::lineage::LineageGraph;
use crate::models::{Annotation, Schema, TableMetadata};

pub struct MetadataService {
    schemas: Arc<RwLock<HashMap<String, Schema>>>,
    lineage: Arc<RwLock<Option<LineageGraph>>>,
}

impl MetadataService {
    pub fn new() -> Self {
        Self {
            schemas: Arc::new(RwLock::new(HashMap::new())),
            lineage: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn save_schema(&self, schema: Schema) -> Result<Schema, Error> {
        let mut schemas = self.schemas.write().await;
        let name = schema.name.clone();
        let mut saved = schema.clone();
        saved.id = Some(0);
        schemas.insert(name, saved.clone());
        Ok(saved)
    }

    pub async fn get_schema(&self, name: &str) -> Result<Schema, Error> {
        let schemas = self.schemas.read().await;
        schemas
            .get(name)
            .cloned()
            .ok_or_else(|| Error::NotFound(format!("Schema '{}' not found", name)))
    }

    pub async fn list_schemas(&self) -> Result<Vec<Schema>, Error> {
        let schemas = self.schemas.read().await;
        Ok(schemas.values().cloned().collect())
    }

    pub async fn delete_schema(&self, name: &str) -> Result<(), Error> {
        let mut schemas = self.schemas.write().await;
        schemas.remove(name).ok_or_else(|| Error::NotFound(format!("Schema '{}' not found", name)))?;
        Ok(())
    }

    pub async fn add_table(&self, schema_name: &str, table: TableMetadata) -> Result<(), Error> {
        let mut schemas = self.schemas.write().await;
        let schema = schemas
            .get_mut(schema_name)
            .ok_or_else(|| Error::NotFound(format!("Schema '{}' not found", schema_name)))?;
        schema.tables.push(table);
        Ok(())
    }

    pub async fn get_table(&self, schema_name: &str, table_name: &str) -> Result<TableMetadata, Error> {
        let schemas = self.schemas.read().await;
        let schema = schemas
            .get(schema_name)
            .ok_or_else(|| Error::NotFound(format!("Schema '{}' not found", schema_name)))?;
        schema
            .tables
            .iter()
            .find(|t| t.name == table_name)
            .cloned()
            .ok_or_else(|| Error::NotFound(format!("Table '{}' not found", table_name)))
    }

    pub async fn add_annotation(
        &self,
        schema_name: &str,
        table_name: &str,
        annotation: Annotation,
    ) -> Result<(), Error> {
        let mut schemas = self.schemas.write().await;
        let schema = schemas
            .get_mut(schema_name)
            .ok_or_else(|| Error::NotFound(format!("Schema '{}' not found", schema_name)))?;
        let table = schema
            .tables
            .iter_mut()
            .find(|t| t.name == table_name)
            .ok_or_else(|| Error::NotFound(format!("Table '{}' not found", table_name)))?;
        table.annotations.push(annotation);
        Ok(())
    }

    pub async fn get_annotations(&self, schema_name: &str, table_name: &str) -> Result<Vec<Annotation>, Error> {
        let table = self.get_table(schema_name, table_name).await?;
        Ok(table.annotations)
    }

    pub async fn set_lineage(&self, graph: LineageGraph) -> Result<(), Error> {
        let mut lineage = self.lineage.write().await;
        *lineage = Some(graph);
        Ok(())
    }

    pub async fn get_lineage(&self) -> Result<LineageGraph, Error> {
        let lineage = self.lineage.read().await;
        lineage
            .clone()
            .ok_or_else(|| Error::NotFound("No lineage graph found".to_string()))
    }

    pub async fn get_table_dependencies(&self, table_id: &str) -> Result<Vec<String>, Error> {
        let lineage = self.lineage.read().await;
        match lineage.as_ref() {
            Some(graph) => Ok(graph.get_table_dependencies(table_id)),
            None => Ok(vec![]),
        }
    }
}

impl Default for MetadataService {
    fn default() -> Self {
        Self::new()
    }
}

use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_save_and_get_schema() {
        let service = MetadataService::new();
        let schema = Schema {
            id: None,
            name: "test".to_string(),
            source: "postgres".to_string(),
            tables: vec![],
            created_at: None,
            updated_at: None,
        };
        
        let saved = service.save_schema(schema.clone()).await.unwrap();
        assert_eq!(saved.name, "test");
        
        let retrieved = service.get_schema("test").await.unwrap();
        assert_eq!(retrieved.name, "test");
    }

    #[tokio::test]
    async fn test_add_annotation() {
        let service = MetadataService::new();
        let schema = Schema {
            id: None,
            name: "test".to_string(),
            source: "postgres".to_string(),
            tables: vec![TableMetadata {
                name: "users".to_string(),
                schema_name: None,
                columns: vec![],
                primary_key: None,
                annotations: vec![],
                description: None,
            }],
            created_at: None,
            updated_at: None,
        };
        
        service.save_schema(schema).await.unwrap();
        
        let annotation = Annotation {
            key: "description".to_string(),
            value: "User data".to_string(),
            source: Some("manual".to_string()),
        };
        
        service.add_annotation("test", "users", annotation).await.unwrap();
        
        let annotations = service.get_annotations("test", "users").await.unwrap();
        assert_eq!(annotations.len(), 1);
        assert_eq!(annotations[0].key, "description");
    }
}
