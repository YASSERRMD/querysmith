use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};

use crate::error::Error;
use crate::models::{Action, Workflow};

pub struct WorkflowEngine {
    workflows: Arc<RwLock<HashMap<String, Workflow>>>,
}

#[async_trait]
pub trait QueryHandler: Send + Sync {
    async fn execute(&self, sql: &str, database: Option<&str>) -> Result<String, String>;
}

#[async_trait]
pub trait NotifyHandler: Send + Sync {
    async fn send(&self, channel: &str, message: &str) -> Result<(), String>;
}

impl WorkflowEngine {
    pub fn new() -> Self {
        Self {
            workflows: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register(&self, workflow: Workflow) -> Result<(), Error> {
        let name = workflow.definition.name.clone();
        let mut workflows = self.workflows.write().await;
        workflows.insert(name, workflow);
        Ok(())
    }

    pub async fn get(&self, name: &str) -> Result<Workflow, Error> {
        let workflows = self.workflows.read().await;
        workflows
            .get(name)
            .cloned()
            .ok_or_else(|| Error::NotFound(format!("Workflow '{}' not found", name)))
    }

    pub async fn list(&self) -> Vec<Workflow> {
        let workflows = self.workflows.read().await;
        workflows.values().cloned().collect()
    }

    pub async fn execute(&self, name: &str) -> Result<String, Error> {
        let workflow = self.get(name).await?;
        self.execute_workflow(&workflow).await
    }

    pub async fn execute_workflow(&self, workflow: &Workflow) -> Result<String, Error> {
        info!("Executing workflow: {}", workflow.definition.name);

        let mut results: Vec<String> = Vec::new();

        for step in &workflow.definition.steps {
            info!("Executing step: {}", step.name);

            let result = self.execute_action(&step.action).await;

            match result {
                Ok(output) => {
                    results.push(format!("{}: {}", step.name, output));
                }
                Err(e) => {
                    error!("Step {} failed: {}", step.name, e);
                    if let Some(on_error) = &step.on_error {
                        results.push(format!("{}: Error handled by '{}'", step.name, on_error));
                    } else {
                        return Err(Error::Execution(format!(
                            "Step {} failed: {}",
                            step.name, e
                        )));
                    }
                }
            }
        }

        Ok(results.join("\n"))
    }

    async fn execute_action(&self, action: &Action) -> Result<String, String> {
        match action {
            Action::Query { sql, database: _ } => Ok(format!(
                "Query handler not configured. Would execute: {}",
                sql
            )),
            Action::Transform { input, script } => {
                Ok(format!("Transform: {} with {}", input, script))
            }
            Action::Notify { channel, message } => Ok(format!(
                "Notify handler not configured. Would send to {}: {}",
                channel, message
            )),
            Action::Sleep { duration } => {
                tokio::time::sleep(tokio::time::Duration::from_secs(*duration)).await;
                Ok(format!("Slept for {} seconds", duration))
            }
        }
    }
}

impl Default for WorkflowEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Schedule {
    pub cron: String,
    pub timezone: Option<String>,
}

impl Schedule {
    pub fn new(cron: &str) -> Self {
        Self {
            cron: cron.to_string(),
            timezone: None,
        }
    }
}

pub struct WorkflowScheduler {
    engine: Arc<WorkflowEngine>,
}

impl WorkflowScheduler {
    pub fn new(engine: Arc<WorkflowEngine>) -> Self {
        Self { engine }
    }

    pub async fn trigger(&self, workflow_name: &str) -> Result<String, String> {
        info!("Manually triggering workflow: {}", workflow_name);
        self.engine
            .execute(workflow_name)
            .await
            .map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_workflow_engine() {
        let engine = WorkflowEngine::new();

        let yaml = r#"
name: test-workflow
version: "1.0"
trigger:
  type: manual
steps:
  - name: step1
    action:
      type: sleep
      duration: 1
"#;

        let definition = crate::models::WorkflowDefinition::from_yaml(yaml).unwrap();
        let workflow = Workflow::new(definition);

        engine.register(workflow).await.unwrap();

        let result = engine.execute("test-workflow").await;
        assert!(result.is_ok());
    }
}
