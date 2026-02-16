use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MemoryScope {
    Global,
    User(String),
    Session(String),
    Table(String),
}

impl MemoryScope {
    pub fn global() -> Self {
        MemoryScope::Global
    }

    pub fn user(user_id: &str) -> Self {
        MemoryScope::User(user_id.to_string())
    }

    pub fn session(session_id: &str) -> Self {
        MemoryScope::Session(session_id.to_string())
    }

    pub fn table(table_name: &str) -> Self {
        MemoryScope::Table(table_name.to_string())
    }

    pub fn key(&self) -> String {
        match self {
            MemoryScope::Global => "global".to_string(),
            MemoryScope::User(id) => format!("user:{}", id),
            MemoryScope::Session(id) => format!("session:{}", id),
            MemoryScope::Table(name) => format!("table:{}", name),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: Option<i64>,
    pub scope: MemoryScope,
    pub content: String,
    pub memory_type: MemoryType,
    pub relevance_score: Option<f32>,
    pub created_at: Option<String>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MemoryType {
    Fact,
    Correction,
    Query,
    Schema,
    Preference,
    Conversation,
}

impl Memory {
    pub fn new(scope: MemoryScope, content: String, memory_type: MemoryType) -> Self {
        Self {
            id: None,
            scope,
            content,
            memory_type,
            relevance_score: None,
            created_at: None,
            metadata: serde_json::json!({}),
        }
    }

    pub fn with_metadata(mut self, key: &str, value: serde_json::Value) -> Self {
        self.metadata[key] = value;
        self
    }

    pub fn with_relevance(mut self, score: f32) -> Self {
        self.relevance_score = Some(score);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Correction {
    pub original_query: String,
    pub corrected_query: String,
    pub error_message: String,
    pub explanation: String,
    pub table_involved: Vec<String>,
}

impl Correction {
    pub fn new(
        original_query: String,
        corrected_query: String,
        error_message: String,
        explanation: String,
    ) -> Self {
        Self {
            original_query,
            corrected_query,
            error_message,
            explanation,
            table_involved: Vec::new(),
        }
    }

    pub fn with_tables(mut self, tables: Vec<String>) -> Self {
        self.table_involved = tables;
        self
    }

    pub fn to_memory(&self, scope: MemoryScope) -> Memory {
        let content = format!(
            "Query Correction:\nOriginal: {}\nCorrected: {}\nError: {}\nExplanation: {}",
            self.original_query, self.corrected_query, self.error_message, self.explanation
        );
        Memory::new(scope, content, MemoryType::Correction)
    }
}
