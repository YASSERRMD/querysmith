use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub name: String,
    pub description: Option<String>,
    pub version: String,
    pub trigger: Trigger,
    pub steps: Vec<Step>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trigger {
    #[serde(rename = "type")]
    pub trigger_type: String,
    pub schedule: Option<String>,
    pub event: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    pub name: String,
    pub action: Action,
    pub on_error: Option<String>,
    pub retry: Option<RetryConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Action {
    #[serde(rename = "query")]
    Query {
        sql: String,
        database: Option<String>,
    },
    #[serde(rename = "transform")]
    Transform { input: String, script: String },
    #[serde(rename = "notify")]
    Notify { channel: String, message: String },
    #[serde(rename = "sleep")]
    Sleep { duration: u64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub delay_seconds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: Option<i64>,
    pub definition: WorkflowDefinition,
    pub enabled: bool,
    pub last_run: Option<String>,
    pub next_run: Option<String>,
}

impl WorkflowDefinition {
    pub fn from_yaml(yaml: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(yaml)
    }
}

impl Workflow {
    pub fn new(definition: WorkflowDefinition) -> Self {
        Self {
            id: None,
            definition,
            enabled: true,
            last_run: None,
            next_run: None,
        }
    }
}
