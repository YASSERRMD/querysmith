use std::collections::HashMap;
use std::sync::Arc;

use crate::traits::Tool;

pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register<T: Tool + 'static>(&mut self, tool: T) {
        let name = tool.name().to_string();
        self.tools.insert(name, Arc::new(tool));
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(name).cloned()
    }

    pub fn list(&self) -> Vec<ToolDefinition> {
        self.tools
            .iter()
            .map(|(name, tool)| ToolDefinition {
                name: name.clone(),
                description: tool.description().to_string(),
                parameters: tool.parameters().clone(),
            })
            .collect()
    }

    pub fn names(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: crate::traits::ToolParameters,
}

impl ToolRegistry {
    pub fn to_json_schemas(&self) -> Vec<serde_json::Value> {
        self.list()
            .into_iter()
            .map(|tool| {
                serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": tool.name,
                        "description": tool.description,
                        "parameters": {
                            "type": tool.parameters.param_type,
                            "properties": tool.parameters.properties.iter().map(|(k, v)| {
                                (k.clone(), serde_json::json!({
                                    "type": v.prop_type,
                                    "description": v.description
                                }))
                            }).collect::<serde_json::Map<String, serde_json::Value>>(),
                            "required": tool.parameters.required
                        }
                    }
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry() {
        let registry = ToolRegistry::new();
        assert!(registry.names().is_empty());
    }
}
