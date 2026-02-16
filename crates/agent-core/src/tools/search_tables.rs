use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use crate::traits::{Tool, ToolParameters, ToolResult};

pub struct SearchTablesTool {
    tables: Vec<TableInfo>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TableInfo {
    pub name: String,
    pub schema: Option<String>,
    pub description: Option<String>,
}

impl SearchTablesTool {
    pub fn new(tables: Vec<TableInfo>) -> Self {
        Self { tables }
    }
}

impl Tool for SearchTablesTool {
    fn name(&self) -> &str {
        "search_tables"
    }

    fn description(&self) -> &str {
        "Search for tables by name or description. Returns matching tables with their schemas."
    }

    fn parameters(&self) -> ToolParameters {
        let mut props = HashMap::new();
        props.insert(
            "query".to_string(),
            crate::traits::ToolProperty {
                prop_type: "string".to_string(),
                description: "Search query for finding tables".to_string(),
            },
        );
        ToolParameters {
            param_type: "object".to_string(),
            properties: props,
            required: vec!["query".to_string()],
        }
    }

    fn execute(
        &self,
        params: HashMap<String, serde_json::Value>,
    ) -> Pin<Box<dyn Future<Output = Result<ToolResult, String>> + Send>> {
        let tables = self.tables.clone();
        Box::pin(async move {
            let query = params
                .get("query")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_lowercase();

            if query.is_empty() {
                return Ok(ToolResult::success(serde_json::json!({
                    "tables": tables.iter().map(|t| {
                        serde_json::json!({
                            "name": t.name,
                            "schema": t.schema,
                            "description": t.description
                        })
                    }).collect::<Vec<_>>()
                })));
            }

            let matches: Vec<_> = tables
                .iter()
                .filter(|t| {
                    t.name.to_lowercase().contains(&query)
                        || t.description
                            .as_ref()
                            .map(|d| d.to_lowercase().contains(&query))
                            .unwrap_or(false)
                })
                .map(|t| {
                    serde_json::json!({
                        "name": t.name,
                        "schema": t.schema,
                        "description": t.description
                    })
                })
                .collect();

            Ok(ToolResult::success(
                serde_json::json!({ "tables": matches }),
            ))
        })
    }
}
