use axum::{
    extract::{Path, State, Json},
    http::{header, StatusCode, HeaderMap},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use sunclaw_app::{build_runtime, RuntimeConfig};
use sunclaw_core::AgentContext;
use sunclaw_runtime::Runtime;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(RustEmbed)]
#[folder = "ui/"]
struct Assets;

#[derive(Clone)]
struct AppState {
    runtime: Arc<Runtime>,
    api_key: String,
}

#[derive(Deserialize)]
struct ChatRequest {
    message: String,
    trace_id: Option<String>,
    model_profile: Option<String>,
}

#[derive(Serialize)]
struct ChatResponse {
    output: String,
    trace_id: String,
}

pub async fn start_server(api_key: String, auto_open: bool) -> anyhow::Result<()> {
    // Initialize logging - ignore error if already initialized
    let _ = tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .try_init();

    let rc = RuntimeConfig {
        provider: "openrouter".to_string(),
        model_id: "deepseek/deepseek-chat".to_string(),
        api_key: api_key.clone(),
        tavily_key: None,
    };
    let runtime = Arc::new(build_runtime(Some(rc)).await);
    
    let state = AppState {
        runtime,
        api_key,
    };

    let app = Router::new()
        // Dashboard routes
        .route("/", get(index_handler))
        .route("/index.html", get(index_handler))
        .route("/assets/*file", get(static_handler))
        
        // API routes
        .route("/api/v1/health", get(health_check))
        .route("/api/v1/chat", post(chat_handler))
        .route("/api/v1/traces", get(api_list_traces))
        .route("/api/v1/missions", get(api_list_missions))
        .route("/api/v1/mission", post(api_create_mission))
        .route("/api/v1/audit/:trace_id", get(api_get_audit))
        .route("/api/v1/trace/:trace_id", get(api_get_trace))
        .route("/api/v1/trace_graph/:trace_id", get(api_get_trace_graph))
        .route("/api/v1/messages/:trace_id", get(api_get_messages))
        
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = "0.0.0.0:18789";
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("🚀 Sunclaw Dashboard running on http://localhost:18789");
    
    // Auto-open browser
    if auto_open {
        let _ = open::that("http://localhost:18789");
    }

    axum::serve(listener, app).await?;

    Ok(())
}

async fn index_handler() -> impl IntoResponse {
    static_handler(Path("index.html".to_string())).await
}

async fn static_handler(Path(path): Path<String>) -> impl IntoResponse {
    match Assets::get(&path) {
        Some(content) => {
            let mime = mime_guess::from_path(&path).first_or_octet_stream();
            (
                [(header::CONTENT_TYPE, mime.as_ref())],
                content.data.into_owned(),
            ).into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn api_list_traces(State(state): State<AppState>) -> impl IntoResponse {
    match state.runtime.memory().list_traces().await {
        Ok(traces) => Json(traces).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn api_get_audit(State(state): State<AppState>, Path(trace_id): Path<String>) -> impl IntoResponse {
    match state.runtime.audit().load_events(&trace_id).await {
        Ok(events) => Json(events).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn api_get_trace(State(state): State<AppState>, Path(trace_id): Path<String>) -> impl IntoResponse {
    match state.runtime.trace().load_traces(&trace_id).await {
        Ok(traces) => Json(traces).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn api_get_trace_graph(State(state): State<AppState>, Path(trace_id): Path<String>) -> impl IntoResponse {
    match state.runtime.trace().load_traces(&trace_id).await {
        Ok(traces) => {
            // Chuyển đổi Traces thành định dạng Mermaid Flowchart đơn giản
            let mut graph = "graph TD\n".to_string();
            graph.push_str("  Start((Start)) --> Thought1\n");
            
            for (i, t) in traces.iter().enumerate() {
                let id = format!("Node{}", i);
                let label = t.content.replace("\"", "'");
                let color = match t.event_type.as_str() {
                    "tool_call" => "fill:#fef3c7,stroke:#fbbf24",
                    "tool_result" => "fill:#f0fdf4,stroke:#4ade80",
                    "error" => "fill:#fef2f2,stroke:#ef4444",
                    _ => "fill:#ede9fe,stroke:#8b5cf6",
                };
                
                graph.push_str(&format!("  {}[\"{}\"]\n", id, label));
                graph.push_str(&format!("  style {} {}\n", id, color));
                
                if i > 0 {
                    graph.push_str(&format!("  Node{} --> {}\n", i-1, id));
                } else {
                    graph.push_str(&format!("  Thought1[\"Bắt đầu suy nghĩ\"] --> {}\n", id));
                }
            }
            
            Json(serde_json::json!({ "mermaid": graph })).into_response()
        },
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn api_get_messages(State(state): State<AppState>, Path(trace_id): Path<String>) -> impl IntoResponse {
    match state.runtime.memory().load_messages(&trace_id).await {
        Ok(messages) => Json(messages).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn api_list_missions(State(state): State<AppState>) -> impl IntoResponse {
    match state.runtime.mission().list_missions().await {
        Ok(missions) => Json(missions).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn api_create_mission(State(state): State<AppState>, Json(mission): Json<sunclaw_core::Mission>) -> impl IntoResponse {
    match state.runtime.mission().create_mission(mission).await {
        Ok(_) => StatusCode::CREATED.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}

async fn auth_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    request: axum::extract::Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let path = request.uri().path();
    
    if path == "/" || path == "/index.html" || path.starts_with("/assets/") || path == "/api/v1/health" {
        return Ok(next.run(request).await);
    }

    if request.method() == axum::http::Method::GET && path.starts_with("/api/v1/") {
         return Ok(next.run(request).await);
    }

    let auth_header = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "));

    if let Some(token) = auth_header {
        if token == state.api_key {
            return Ok(next.run(request).await);
        }
    }

    Err(StatusCode::UNAUTHORIZED)
}

async fn chat_handler(
    State(state): State<AppState>,
    Json(payload): Json<ChatRequest>,
) -> impl IntoResponse {
    let trace_id = payload.trace_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    
    let ctx = AgentContext {
        trace_id: trace_id.clone(),
        skill: None,
        model_profile: payload.model_profile,
        role: None,
        max_tokens: None,
    };

    match state.runtime.run_once(&ctx, &payload.message).await {
        Ok(outcome) => Json(ChatResponse {
            output: outcome.output,
            trace_id,
        }).into_response(),
        Err(e) => {
            let status = StatusCode::INTERNAL_SERVER_ERROR;
            let body = e.to_string();
            (status, body).into_response()
        }
    }
}
