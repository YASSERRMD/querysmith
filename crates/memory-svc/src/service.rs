use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::Error;
use crate::models::{Correction, Memory, MemoryScope, MemoryType};

pub struct MemoryService {
    memories: Arc<RwLock<HashMap<String, Vec<Memory>>>>,
}

impl MemoryService {
    pub fn new() -> Self {
        Self {
            memories: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn save(&self, memory: Memory) -> Result<Memory, Error> {
        let scope_key = memory.scope.key();
        let mut memories = self.memories.write().await;
        
        let entry = memories.entry(scope_key).or_insert_with(Vec::new);
        entry.push(memory.clone());
        
        Ok(memory)
    }

    pub async fn get(&self, scope: &MemoryScope) -> Result<Vec<Memory>, Error> {
        let scope_key = scope.key();
        let memories = self.memories.read().await;
        
        Ok(memories.get(&scope_key).cloned().unwrap_or_default())
    }

    pub async fn get_all(&self) -> Result<Vec<Memory>, Error> {
        let memories = self.memories.read().await;
        let mut all: Vec<Memory> = memories.values().flatten().cloned().collect();
        all.sort_by(|a, b| {
            let empty = String::new();
            let a_time = a.created_at.as_ref().unwrap_or(&empty);
            let b_time = b.created_at.as_ref().unwrap_or(&empty);
            b_time.cmp(a_time)
        });
        Ok(all)
    }

    pub async fn retrieve(&self, query: &str, scope: Option<MemoryScope>, limit: usize) -> Result<Vec<Memory>, Error> {
        let query_lower = query.to_lowercase();
        let memories = self.memories.read().await;
        
        let mut results: Vec<Memory> = Vec::new();
        
        let scopes_to_search = if let Some(ref s) = scope {
            vec![s.key()]
        } else {
            memories.keys().cloned().collect()
        };
        
        for scope_key in scopes_to_search {
            if let Some(scope_memories) = memories.get(&scope_key) {
                for memory in scope_memories {
                    if self.is_relevant(&memory.content, &query_lower) {
                        results.push(memory.clone());
                    }
                }
            }
        }
        
        results.sort_by(|a, b| {
            let a_score = a.relevance_score.unwrap_or(0.0);
            let b_score = b.relevance_score.unwrap_or(0.0);
            b_score.partial_cmp(&a_score).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        results.truncate(limit);
        Ok(results)
    }

    fn is_relevant(&self, content: &str, query: &str) -> bool {
        let content_lower = content.to_lowercase();
        let query_terms: Vec<&str> = query.split_whitespace().collect();
        
        query_terms.iter().any(|term| content_lower.contains(term))
    }

    pub async fn save_correction(&self, correction: Correction, scope: MemoryScope) -> Result<Memory, Error> {
        let memory = correction.to_memory(scope);
        self.save(memory).await
    }

    pub async fn get_corrections(&self, scope: &MemoryScope) -> Result<Vec<Correction>, Error> {
        let memories = self.get(scope).await?;
        
        let corrections: Vec<Correction> = memories
            .iter()
            .filter(|m| m.memory_type == MemoryType::Correction)
            .filter_map(|m| self.parse_correction(&m.content))
            .collect();
        
        Ok(corrections)
    }

    fn parse_correction(&self, content: &str) -> Option<Correction> {
        let mut original = String::new();
        let mut corrected = String::new();
        let mut error_msg = String::new();
        let mut explanation = String::new();
        
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("Original:") {
                original = line.replace("Original:", "").trim().to_string();
            } else if line.starts_with("Corrected:") {
                corrected = line.replace("Corrected:", "").trim().to_string();
            } else if line.starts_with("Error:") {
                error_msg = line.replace("Error:", "").trim().to_string();
            } else if line.starts_with("Explanation:") {
                explanation = line.replace("Explanation:", "").trim().to_string();
            }
        }
        
        if original.is_empty() || corrected.is_empty() {
            return None;
        }
        
        Some(Correction::new(original, corrected, error_msg, explanation))
    }

    pub async fn inject_into_prompt(&self, query: &str, scope: Option<MemoryScope>) -> Result<String, Error> {
        let memories = self.retrieve(query, scope, 5).await?;
        
        if memories.is_empty() {
            return Ok(String::new());
        }
        
        let mut context = String::from("\n\nRelevant memories:\n");
        
        for memory in &memories {
            context.push_str(&format!("- {}\n", memory.content));
        }
        
        Ok(context)
    }

    pub async fn delete(&self, scope: &MemoryScope, memory_id: i64) -> Result<(), Error> {
        let scope_key = scope.key();
        let mut memories = self.memories.write().await;
        
        if let Some(scope_memories) = memories.get_mut(&scope_key) {
            scope_memories.retain(|m| m.id != Some(memory_id));
        }
        
        Ok(())
    }

    pub async fn clear(&self, scope: &MemoryScope) -> Result<(), Error> {
        let scope_key = scope.key();
        let mut memories = self.memories.write().await;
        memories.remove(&scope_key);
        Ok(())
    }

    pub async fn count(&self) -> usize {
        let memories = self.memories.read().await;
        memories.values().map(|v| v.len()).sum()
    }
}

impl Default for MemoryService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_save_and_retrieve() {
        let service = MemoryService::new();
        
        let memory = Memory::new(
            MemoryScope::global(),
            "Users table has id, name, email".to_string(),
            MemoryType::Fact,
        );
        
        service.save(memory).await.unwrap();
        
        let retrieved = service.retrieve("users", None, 10).await.unwrap();
        assert!(!retrieved.is_empty());
    }

    #[tokio::test]
    async fn test_save_correction() {
        let service = MemoryService::new();
        
        let correction = Correction::new(
            "SELECT * FORM users".to_string(),
            "SELECT * FROM users".to_string(),
            "syntax error".to_string(),
            "Fixed typo in FROM clause".to_string(),
        );
        
        service.save_correction(correction, MemoryScope::table("users")).await.unwrap();
        
        let corrections = service.get_corrections(&MemoryScope::table("users")).await.unwrap();
        assert!(!corrections.is_empty());
    }
}
