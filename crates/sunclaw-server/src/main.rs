use axum::{
    extract::{State, Json},
    http::{StatusCode, HeaderMap},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use sunclaw_app::build_runtime;
use sunclaw_core::{AgentContext, Role};
use sunclaw_runtime::Runtime;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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
        .route("/api/v1/health", get(health_check))
        .route("/api/v1/chat", post(chat_handler))
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    tracing::info!("Sunclaw API Server running on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
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
        Ok(outcome) => {
            Json(ChatResponse {
                output: outcome.output,
                trace_id,
            }).into_response()
        }
        Err(e) => {
            tracing::error!("Runtime error: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}
