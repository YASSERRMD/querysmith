use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageGraph {
    pub nodes: Vec<LineageNode>,
    pub relationships: Vec<LineageRelationship>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageNode {
    pub id: String,
    pub name: String,
    pub node_type: NodeType,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    Table,
    Column,
    View,
    Source,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageRelationship {
    pub from_node: String,
    pub to_node: String,
    pub relationship_type: RelationshipType,
    pub transform: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelationshipType {
    DependsOn,
    DerivedFrom,
    JoinedWith,
    AggregatedFrom,
}

impl LineageGraph {
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn add_node(&mut self, node: LineageNode) {
        self.nodes.push(node);
    }

    pub fn add_relationship(&mut self, rel: LineageRelationship) {
        self.relationships.push(rel);
    }

    pub fn get_table_dependencies(&self, table_id: &str) -> Vec<String> {
        self.relationships
            .iter()
            .filter(|r| r.from_node == table_id)
            .filter(|r| {
                matches!(
                    r.relationship_type,
                    RelationshipType::DependsOn | RelationshipType::DerivedFrom
                )
            })
            .map(|r| r.to_node.clone())
            .collect()
    }
}
