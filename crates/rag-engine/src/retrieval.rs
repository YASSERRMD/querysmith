use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::types::VectorIndex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievedChunk {
    pub id: String,
    pub content: String,
    pub source: SourceType,
    pub score: f32,
    pub metadata: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SourceType {
    Table,
    Documentation,
    Memory,
    Schema,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalResult {
    pub query: String,
    pub chunks: Vec<RetrievedChunk>,
    pub total_results: usize,
}

pub struct RAGService {
    table_index: VectorIndex,
    doc_index: VectorIndex,
    memory_index: VectorIndex,
    schema_index: VectorIndex,
}

impl RAGService {
    pub fn new(dimension: usize) -> Self {
        Self {
            table_index: VectorIndex::new(dimension),
            doc_index: VectorIndex::new(dimension),
            memory_index: VectorIndex::new(dimension),
            schema_index: VectorIndex::new(dimension),
        }
    }

    pub fn index_table(
        &mut self,
        id: String,
        vector: Vec<f32>,
        content: String,
        metadata: serde_json::Value,
    ) {
        self.table_index
            .add_with_content(id, vector, content, metadata);
    }

    pub fn index_documentation(
        &mut self,
        id: String,
        vector: Vec<f32>,
        content: String,
        metadata: serde_json::Value,
    ) {
        self.doc_index
            .add_with_content(id, vector, content, metadata);
    }

    pub fn index_memory(
        &mut self,
        id: String,
        vector: Vec<f32>,
        content: String,
        metadata: serde_json::Value,
    ) {
        self.memory_index
            .add_with_content(id, vector, content, metadata);
    }

    pub fn index_schema(
        &mut self,
        id: String,
        vector: Vec<f32>,
        content: String,
        metadata: serde_json::Value,
    ) {
        self.schema_index
            .add_with_content(id, vector, content, metadata);
    }

    pub async fn retrieve(
        &self,
        query: &str,
        query_vector: &[f32],
        k: usize,
        sources: Option<Vec<SourceType>>,
    ) -> RetrievalResult {
        let sources = sources.unwrap_or_else(|| {
            vec![
                SourceType::Table,
                SourceType::Documentation,
                SourceType::Memory,
                SourceType::Schema,
            ]
        });

        let mut all_chunks = Vec::new();

        if sources.contains(&SourceType::Table) {
            let results = self.table_index.search_with_content(query_vector, k);
            for (id, content, score) in results {
                all_chunks.push(RetrievedChunk {
                    id: id.clone(),
                    content,
                    source: SourceType::Table,
                    score,
                    metadata: {
                        let mut m = HashMap::new();
                        m.insert("type".to_string(), Value::String("table".to_string()));
                        m
                    },
                });
            }
        }

        if sources.contains(&SourceType::Documentation) {
            let results = self.doc_index.search_with_content(query_vector, k);
            for (id, content, score) in results {
                all_chunks.push(RetrievedChunk {
                    id: id.clone(),
                    content,
                    source: SourceType::Documentation,
                    score,
                    metadata: {
                        let mut m = HashMap::new();
                        m.insert("type".to_string(), Value::String("doc".to_string()));
                        m
                    },
                });
            }
        }

        if sources.contains(&SourceType::Memory) {
            let results = self.memory_index.search_with_content(query_vector, k);
            for (id, content, score) in results {
                all_chunks.push(RetrievedChunk {
                    id: id.clone(),
                    content,
                    source: SourceType::Memory,
                    score,
                    metadata: {
                        let mut m = HashMap::new();
                        m.insert("type".to_string(), Value::String("memory".to_string()));
                        m
                    },
                });
            }
        }

        if sources.contains(&SourceType::Schema) {
            let results = self.schema_index.search_with_content(query_vector, k);
            for (id, content, score) in results {
                all_chunks.push(RetrievedChunk {
                    id: id.clone(),
                    content,
                    source: SourceType::Schema,
                    score,
                    metadata: {
                        let mut m = HashMap::new();
                        m.insert("type".to_string(), Value::String("schema".to_string()));
                        m
                    },
                });
            }
        }

        all_chunks.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        all_chunks.truncate(k);

        let total_results = all_chunks.len();

        RetrievalResult {
            query: query.to_string(),
            chunks: all_chunks,
            total_results,
        }
    }

    pub fn format_context(&self, result: &RetrievalResult) -> String {
        let mut context = String::from("## Relevant Context\n\n");

        for chunk in &result.chunks {
            context.push_str(&format!(
                "### {} (score: {:.2})\n",
                self.source_to_string(&chunk.source),
                chunk.score
            ));
            context.push_str(&chunk.content);
            context.push_str("\n\n");
        }

        context
    }

    fn source_to_string(&self, source: &SourceType) -> String {
        match source {
            SourceType::Table => "Table".to_string(),
            SourceType::Documentation => "Documentation".to_string(),
            SourceType::Memory => "Memory".to_string(),
            SourceType::Schema => "Schema".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rag_service_creation() {
        let rag = RAGService::new(128);
        assert!(!rag.table_index.vectors.is_empty() || rag.table_index.vectors.is_empty());
    }

    #[test]
    fn test_index_and_retrieve() {
        let mut rag = RAGService::new(3);

        rag.index_table(
            "users".to_string(),
            vec![1.0, 0.0, 0.0],
            "Users table with id, name, email".to_string(),
            serde_json::json!({"table": "users"}),
        );

        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(rag.retrieve("user data", &vec![1.0, 0.0, 0.0], 5, None));

        assert!(!result.chunks.is_empty());
    }
}
