use std::sync::Arc;

use crate::llm::{ChatCompletionResponse, MessageRole, ToolCall};
use crate::runtime::AgentRuntime;

pub struct AgentOrchestrator {
    runtime: Arc<AgentRuntime>,
    messages: Vec<crate::llm::ChatMessage>,
}

impl AgentOrchestrator {
    pub fn new(runtime: AgentRuntime) -> Self {
        Self {
            runtime: Arc::new(runtime),
            messages: Vec::new(),
        }
    }

    pub fn with_initial_message(mut self, content: String) -> Self {
        self.add_user_message(content);
        self
    }

    pub fn add_message(&mut self, role: MessageRole, content: String) {
        self.messages.push(crate::llm::ChatMessage {
            role,
            content,
            tool_calls: None,
            tool_call_id: None,
        });
    }

    pub fn add_user_message(&mut self, content: String) {
        self.add_message(MessageRole::User, content);
    }

    pub fn messages(&self) -> &[crate::llm::ChatMessage] {
        &self.messages
    }

    pub fn clear(&mut self) {
        self.messages.clear();
    }

    pub async fn process_response(&mut self, response: ChatCompletionResponse) -> Result<AgentAction, String> {
        let choice = response.choices.first()
            .ok_or("No choices in response")?;

        let message = &choice.message;
        
        if let Some(tool_calls) = &message.tool_calls {
            let tool_call = tool_calls.first()
                .ok_or("No tool calls in message")?;
            
            self.messages.push(crate::llm::ChatMessage {
                role: MessageRole::Assistant,
                content: message.content.clone(),
                tool_calls: Some(tool_calls.clone()),
                tool_call_id: None,
            });

            Ok(AgentAction::ToolCall(tool_call.clone()))
        } else if !message.content.is_empty() {
            self.messages.push(crate::llm::ChatMessage {
                role: MessageRole::Assistant,
                content: message.content.clone(),
                tool_calls: None,
                tool_call_id: None,
            });

            Ok(AgentAction::Response(message.content.clone()))
        } else {
            Ok(AgentAction::Response("No response content".to_string()))
        }
    }

    pub fn add_tool_result(&mut self, tool_call_id: &str, result: String) {
        self.messages.push(crate::llm::ChatMessage {
            role: MessageRole::Tool,
            content: result,
            tool_calls: None,
            tool_call_id: Some(tool_call_id.to_string()),
        });
    }

    pub fn get_messages_for_llm(&self) -> Vec<crate::llm::ChatMessage> {
        let mut msgs = vec![crate::llm::ChatMessage {
            role: MessageRole::System,
            content: self.runtime.system_prompt.clone(),
            tool_calls: None,
            tool_call_id: None,
        }];
        msgs.extend(self.messages.clone());
        msgs
    }

    pub fn max_retries(&self) -> usize {
        self.runtime.max_retries()
    }
}

#[derive(Debug, Clone)]
pub enum AgentAction {
    ToolCall(ToolCall),
    Response(String),
    Error(String),
}

pub struct SelfCorrectingAgent {
    orchestrator: AgentOrchestrator,
}

impl SelfCorrectingAgent {
    pub fn new(orchestrator: AgentOrchestrator) -> Self {
        Self { orchestrator }
    }

    pub async fn execute_with_retry<F, Fut>(&mut self, llm_call: F) -> Result<String, String>
    where
        F: Fn(Vec<crate::llm::ChatMessage>) -> Fut,
        Fut: std::future::Future<Output = Result<ChatCompletionResponse, String>>,
    {
        let max_retries = self.orchestrator.max_retries();
        
        for attempt in 0..max_retries {
            let messages = self.orchestrator.get_messages_for_llm();
            
            let response = llm_call(messages).await?;
            
            let action = self.orchestrator.process_response(response).await?;
            
            match action {
                AgentAction::Response(content) => {
                    return Ok(content);
                }
                AgentAction::ToolCall(tool_call) => {
                    let tool_name = &tool_call.name;
                    let arguments = tool_call.arguments;
                    let tool_call_id = &tool_call.id;
                    
                    let result = self.orchestrator.runtime.execute_tool(tool_name, arguments).await;
                    
                    match result {
                        Ok(result_str) => {
                            self.orchestrator.add_tool_result(tool_call_id, result_str);
                        }
                        Err(error) => {
                            self.orchestrator.add_tool_result(tool_call_id, format!("Error: {}", error));
                        }
                    }
                }
                AgentAction::Error(e) => {
                    if attempt == max_retries - 1 {
                        return Err(e);
                    }
                }
            }
        }
        
        Err("Max retries exceeded".to_string())
    }
}
