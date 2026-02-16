use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> ToolParameters;
    fn execute(
        &self,
        params: HashMap<String, serde_json::Value>,
    ) -> Pin<Box<dyn Future<Output = Result<ToolResult, String>> + Send>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameters {
    #[serde(rename = "type")]
    pub param_type: String,
    pub properties: HashMap<String, ToolProperty>,
    pub required: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolProperty {
    #[serde(rename = "type")]
    pub prop_type: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
}

impl ToolResult {
    pub fn success(data: serde_json::Value) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

pub fn search_tables_params() -> ToolParameters {
    let mut props = HashMap::new();
    props.insert(
        "query".to_string(),
        ToolProperty {
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

pub fn run_sql_params() -> ToolParameters {
    let mut props = HashMap::new();
    props.insert(
        "sql".to_string(),
        ToolProperty {
            prop_type: "string".to_string(),
            description: "SQL query to execute".to_string(),
        },
    );
    ToolParameters {
        param_type: "object".to_string(),
        properties: props,
        required: vec!["sql".to_string()],
    }
}

pub fn debug_query_params() -> ToolParameters {
    let mut props = HashMap::new();
    props.insert(
        "sql".to_string(),
        ToolProperty {
            prop_type: "string".to_string(),
            description: "SQL query to debug".to_string(),
        },
    );
    props.insert(
        "error".to_string(),
        ToolProperty {
            prop_type: "string".to_string(),
            description: "Error message from failed query".to_string(),
        },
    );
    ToolParameters {
        param_type: "object".to_string(),
        properties: props,
        required: vec!["sql".to_string(), "error".to_string()],
    }
}
