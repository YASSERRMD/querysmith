use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    http::{header, Method},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};

#[derive(Clone)]
#[allow(dead_code)]
struct AppState {
    agent: Arc<agent_core::AgentRuntime>,
    memory: Arc<memory_svc::MemoryService>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatRequest {
    message: String,
    user_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatResponse {
    response: String,
    tool_calls: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

async fn chat_handler(
    State(_state): State<AppState>,
    Json(payload): Json<ChatRequest>,
) -> impl IntoResponse {
    let _user_id = payload.user_id.unwrap_or_else(|| "anonymous".to_string());

    Json(ApiResponse {
        success: true,
        data: Some(ChatResponse {
            response: "Response from agent".to_string(),
            tool_calls: None,
        }),
        error: None,
    })
}

async fn ws_handler(ws: WebSocketUpgrade, State(_state): State<AppState>) -> Response {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(socket: WebSocket) {
    let (mut sender, mut receiver) = socket.split();

    while let Some(msg) = receiver.next().await {
        if let Ok(msg) = msg {
            if let Message::Text(text) = msg {
                let _ = sender.send(Message::Text(format!("Echo: {}", text))).await;
            }
        } else {
            break;
        }
    }
}

async fn health_handler() -> impl IntoResponse {
    Json(ApiResponse {
        success: true,
        data: Some(serde_json::json!({"status": "healthy"})),
        error: None,
    })
}

fn cors() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::CONTENT_TYPE])
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let agent = Arc::new(agent_core::AgentRuntime::new(
        "minimax-m2.5".to_string(),
        agent_core::ToolRegistry::new(),
    ));

    let memory = Arc::new(memory_svc::MemoryService::new());

    let state = AppState { agent, memory };

    let app = Router::new()
        .route("/", get(|| async { "QuerySmith API" }))
        .route("/health", get(health_handler))
        .route("/chat", post(chat_handler))
        .route("/ws", get(ws_handler))
        .layer(ServiceBuilder::new().layer(cors()))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    tracing::info!("Starting QuerySmith API on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
