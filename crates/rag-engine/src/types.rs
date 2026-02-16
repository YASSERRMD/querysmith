use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorIndex {
    pub dimension: usize,
    pub vectors: HashMap<String, Vec<f32>>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl VectorIndex {
    pub fn new(dimension: usize) -> Self {
        Self {
            dimension,
            vectors: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn add(&mut self, id: String, vector: Vec<f32>, metadata: serde_json::Value) {
        if vector.len() != self.dimension {
            return;
        }
        self.vectors.insert(id.clone(), vector);
        self.metadata.insert(id, metadata);
    }

    pub fn search(&self, query: &[f32], k: usize) -> Vec<(String, f32)> {
        if query.len() != self.dimension || self.vectors.is_empty() {
            return vec![];
        }

        let mut scores: Vec<(String, f32)> = self
            .vectors
            .iter()
            .map(|(id, vec)| (id.clone(), self.cosine_similarity(query, vec)))
            .collect();

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scores.truncate(k);
        scores
    }

    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let mag_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let mag_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if mag_a == 0.0 || mag_b == 0.0 {
            0.0
        } else {
            dot / (mag_a * mag_b)
        }
    }

    pub fn get(&self, id: &str) -> Option<(&Vec<f32>, &serde_json::Value)> {
        let vector = self.vectors.get(id)?;
        let meta = self.metadata.get(id)?;
        Some((vector, meta))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_index() {
        let mut index = VectorIndex::new(3);

        index.add(
            "doc1".to_string(),
            vec![1.0, 0.0, 0.0],
            serde_json::json!({"text": "hello"}),
        );
        index.add(
            "doc2".to_string(),
            vec![0.0, 1.0, 0.0],
            serde_json::json!({"text": "world"}),
        );

        let results = index.search(&vec![1.0, 0.0, 0.0], 1);
        assert_eq!(results[0].0, "doc1");
    }
}
