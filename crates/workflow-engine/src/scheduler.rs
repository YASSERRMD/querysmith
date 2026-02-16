use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tracing::info;

use crate::engine::WorkflowEngine;
use crate::models::Schedule;

pub struct WorkflowScheduler {
    engine: Arc<WorkflowEngine>,
    tasks: Arc<RwLock<Vec<ScheduledTask>>>,
}

struct ScheduledTask {
    workflow_name: String,
    schedule: Schedule,
    running: bool,
}

impl WorkflowScheduler {
    pub fn new(engine: Arc<WorkflowEngine>) -> Self {
        Self {
            engine,
            tasks: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn schedule(&self, workflow_name: String, schedule: Schedule) {
        info!("Scheduling workflow '{}' with cron: {}", workflow_name, schedule.cron);
        
        let task = ScheduledTask {
            workflow_name: workflow_name.clone(),
            schedule,
            running: false,
        };
        
        self.tasks.write().await.push(task);
    }

    pub async fn unschedule(&self, workflow_name: &str) {
        let mut tasks = self.tasks.write().await;
        tasks.retain(|t| t.workflow_name != workflow_name);
    }

    pub async fn list_scheduled(&self) -> Vec<String> {
        let tasks = self.tasks.read().await;
        tasks.iter().map(|t| t.workflow_name.clone()).collect()
    }

    pub async fn trigger(&self, workflow_name: &str) -> Result<String, String> {
        info!("Manually triggering workflow: {}", workflow_name);
        
        self.engine
            .execute(workflow_name)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn start(&self) {
        info!("Starting workflow scheduler");
    }
}
