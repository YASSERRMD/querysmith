use std::collections::HashMap;
use std::sync::Arc;

use crate::llm::{ChatMessage, MessageRole};
use crate::registry::ToolRegistry;

pub struct AgentRuntime {
    pub model: String,
    pub tools: Arc<ToolRegistry>,
    pub max_retries: usize,
    pub system_prompt: String,
}

impl AgentRuntime {
    pub fn new(model: String, tools: ToolRegistry) -> Self {
        Self {
            model,
            tools: Arc::new(tools),
            max_retries: 3,
            system_prompt: Self::default_system_prompt(),
        }
    }

    pub fn with_system_prompt(mut self, prompt: String) -> Self {
        self.system_prompt = prompt;
        self
    }

    pub fn with_max_retries(mut self, retries: usize) -> Self {
        self.max_retries = retries;
        self
    }

    fn default_system_prompt() -> String {
        r#"You are QuerySmith, an AI data agent that helps users query databases using natural language.

Your capabilities:
1. Search for relevant tables using the search_tables tool
2. Run SQL queries using the run_sql tool
3. Debug and fix failed queries using the debug_query tool

Guidelines:
- Always explore available tables before writing complex queries
- Provide clear explanations of your SQL
- If a query fails, use the debug_query tool to analyze the error
- Return results in a user-friendly format

When you need to use a tool, respond with a JSON object containing tool_calls.
"#.to_string()
    }

    pub fn build_system_message(&self) -> ChatMessage {
        ChatMessage {
            role: MessageRole::System,
            content: self.system_prompt.clone(),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    pub fn get_tool_schemas(&self) -> Vec<serde_json::Value> {
        self.tools.to_json_schemas()
    }

    pub async fn execute_tool(&self, tool_name: &str, arguments: serde_json::Value) -> Result<String, String> {
        let tool = self.tools.get(tool_name).ok_or_else(|| format!("Tool not found: {}", tool_name))?;
        
        let params: HashMap<String, serde_json::Value> = serde_json::from_value(arguments)
            .map_err(|e| format!("Invalid arguments: {}", e))?;

        let result = tool.execute(params).await;
        
        match result {
            Ok(tool_result) => {
                if tool_result.success {
                    Ok(serde_json::to_string(&tool_result.data).unwrap_or_else(|_| "{}".to_string()))
                } else {
                    Err(tool_result.error.unwrap_or_else(|| "Unknown error".to_string()))
                }
            }
            Err(e) => Err(e),
        }
    }

    pub fn get_tools(&self) -> Arc<ToolRegistry> {
        self.tools.clone()
    }

    pub fn max_retries(&self) -> usize {
        self.max_retries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_runtime_creation() {
        let registry = ToolRegistry::new();
        let runtime = AgentRuntime::new("minimax-m2.5".to_string(), registry);
        
        assert_eq!(runtime.model, "minimax-m2.5");
        assert_eq!(runtime.max_retries(), 3);
    }
}
