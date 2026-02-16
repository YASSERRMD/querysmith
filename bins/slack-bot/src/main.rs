use axum::{
    extract::Json,
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Router,
};
use memory_svc::{Memory, MemoryScope, MemoryType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

#[derive(Clone)]
struct SlackBotState {
    agent: Arc<agent_core::AgentRuntime>,
    memory: Arc<memory_svc::MemoryService>,
    conversations: Arc<RwLock<HashMap<String, ConversationState>>>,
}

#[derive(Clone)]
struct ConversationState {
    user_id: String,
    thread_ts: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SlackEvent {
    #[serde(rename = "type")]
    event_type: String,
    channel: Option<String>,
    user: Option<String>,
    text: Option<String>,
    ts: Option<String>,
    thread_ts: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SlackRequest {
    #[serde(rename = "type")]
    request_type: String,
    challenge: Option<String>,
    event: Option<SlackEvent>,
    command: Option<String>,
    text: Option<String>,
    user_id: Option<String>,
    channel_id: Option<String>,
    response_url: Option<String>,
    thread_ts: Option<String>,
}

#[derive(Debug, Serialize)]
struct SlackResponse {
    response_type: String,
    text: String,
}

async fn handle_url_verification(Json(payload): Json<SlackRequest>) -> impl IntoResponse {
    if let Some(challenge) = payload.challenge {
        (StatusCode::OK, challenge)
    } else {
        (StatusCode::OK, "".to_string())
    }
}

async fn handle_event_callback(
    state: axum::extract::State<SlackBotState>,
    Json(payload): Json<SlackRequest>,
) -> impl IntoResponse {
    if payload.request_type != "event_callback" {
        return (StatusCode::OK, "".to_string());
    }

    if let Some(event) = payload.event {
        if event.event_type == "message" {
            if let (Some(text), Some(user), Some(channel), Some(thread_ts)) = 
                (event.text, event.user, event.channel, event.thread_ts.or(event.ts)) 
            {
                info!("Received message from user {} in channel {}", user, channel);
                
                let conversation_key = format!("{}:{}", channel, thread_ts);
                
                let _ = state.conversations.write().await.insert(
                    conversation_key.clone(),
                    ConversationState {
                        user_id: user.clone(),
                        thread_ts: Some(thread_ts),
                    },
                );
                
                let user_memory_scope = MemoryScope::user(&user);
                let _ = state.memory.retrieve(&text, Some(user_memory_scope), 5).await;
            }
        }
    }

    (StatusCode::OK, "".to_string())
}

async fn handle_slash_command(
    state: axum::extract::State<SlackBotState>,
    Json(payload): Json<SlackRequest>,
) -> impl IntoResponse {
    let command = payload.command.unwrap_or_default();
    let text = payload.text.unwrap_or_default();
    let user_id = payload.user_id.unwrap_or_default();
    let channel_id = payload.channel_id.unwrap_or_default();

    info!("Slash command: {} with text: {}", command, text);

    match command.as_str() {
        "/query" | "/querysmith" => {
            let user_memory_scope = MemoryScope::user(&user_id);
            let context = state.memory.inject_into_prompt(&text, Some(user_memory_scope)).await.unwrap_or_default();
            
            let full_prompt = if context.is_empty() {
                text.clone()
            } else {
                format!("{}\n\nRelevant context:\n{}", text, context)
            };
            
            let response_text = format!("Processing query: {}\n\n{}", text, "This is a placeholder response. Connect to LLM to get actual results.");
            
            let _ = state.memory.save(
                Memory::new(
                    MemoryScope::user(&user_id),
                    format!("Q: {}\nA: {}", text, response_text),
                    MemoryType::Conversation,
                )
            ).await;

            Json(SlackResponse {
                response_type: "in_channel".to_string(),
                text: response_text,
            })
        }
        _ => {
            Json(SlackResponse {
                response_type: "ephemeral".to_string(),
                text: "Unknown command. Try /query <your question>".to_string(),
            })
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    info!("Starting QuerySmith Slack Bot");

    let agent = Arc::new(agent_core::AgentRuntime::new(
        "minimax-m2.5".to_string(),
        agent_core::ToolRegistry::new(),
    ));

    let memory = Arc::new(memory_svc::MemoryService::new());
    let conversations = Arc::new(RwLock::new(HashMap::new()));

    let state = SlackBotState {
        agent,
        memory,
        conversations,
    };

    let app = Router::new()
        .route("/slack/events", post(handle_url_verification))
        .route("/slack/events", post(handle_event_callback))
        .route("/slack/commands", post(handle_slash_command))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    info!("Slack bot listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}
