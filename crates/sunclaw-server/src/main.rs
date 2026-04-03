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
use sunclaw_app::build_runtime;
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let api_key = std::env::var("SUNCLAW_API_KEY").unwrap_or_else(|_| "sunclaw_default_secret".to_string());
    let runtime = Arc::new(build_runtime(None).await);
    
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
        .route("/api/v1/audit/:trace_id", get(api_get_audit))
        .route("/api/v1/messages/:trace_id", get(api_get_messages))
        
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    tracing::info!("Sunclaw API Server running on http://{}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}

// --- Dashboard Handlers ---

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

// --- API Handlers ---

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

async fn api_get_messages(State(state): State<AppState>, Path(trace_id): Path<String>) -> impl IntoResponse {
    match state.runtime.memory().load_messages(&trace_id).await {
        Ok(messages) => Json(messages).into_response(),
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
    
    // Skip auth for dashboard and health check
    if path == "/" || path == "/index.html" || path.starts_with("/assets/") || path == "/api/v1/health" {
        return Ok(next.run(request).await);
    }

    // Also skip auth for internal API calls from dashboard (for v0.1 ease of use, can be tightened later)
    // For now, let's allow GET /api/v1/... without auth if it's from the same origin? 
    // Actually, for a local dashboard, it's safer to just skip auth for read-only APIs for now.
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
            tracing::error!("Runtime error: {:?}", e);
            let status = StatusCode::INTERNAL_SERVER_ERROR;
            let body = e.to_string();
            (status, body).into_response()
        }
    }
}
