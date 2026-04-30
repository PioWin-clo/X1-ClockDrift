use crate::config::Config;
use crate::db::{self, Pool};
use anyhow::Result;
use axum::{
    extract::{Path as AxPath, State},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

#[derive(Clone)]
struct AppState {
    pool: Pool,
    config: Arc<Config>,
}

pub async fn run(pool: Pool, config: Config, shutdown: CancellationToken) -> Result<()> {
    let listen = config.api_listen.clone();
    let data_dir = PathBuf::from(&config.git_repo_path).join("data");
    let frontend_dir = config
        .frontend_dir
        .clone()
        .unwrap_or_else(|| "/home/x1pio/strontium-meter/frontend".to_string());

    let state = AppState {
        pool,
        config: Arc::new(config),
    };

    let app = Router::new()
        .route("/api/summary", get(handle_summary))
        .route("/api/validators", get(handle_validators))
        .route("/api/history", get(handle_history))
        .route("/api/validator/:pubkey", get(handle_validator))
        .route("/healthz", get(handle_health))
        .nest_service("/data", ServeDir::new(&data_dir))
        .fallback_service(ServeDir::new(&frontend_dir))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&listen).await?;
    tracing::info!(listen = %listen, "api server listening");
    axum::serve(listener, app)
        .with_graceful_shutdown(async move { shutdown.cancelled().await })
        .await?;
    tracing::info!("api server shut down");
    Ok(())
}

async fn read_static_json(state: &AppState, name: &str) -> Result<Value, StatusCode> {
    let path = PathBuf::from(&state.config.git_repo_path)
        .join("data")
        .join(name);
    let bytes = tokio::fs::read(&path).await.map_err(|_| StatusCode::NOT_FOUND)?;
    serde_json::from_slice(&bytes).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn handle_summary(State(s): State<AppState>) -> Result<Json<Value>, StatusCode> {
    Ok(Json(read_static_json(&s, "summary.json").await?))
}

async fn handle_validators(State(s): State<AppState>) -> Result<Json<Value>, StatusCode> {
    Ok(Json(read_static_json(&s, "validators.json").await?))
}

async fn handle_history(State(s): State<AppState>) -> Result<Json<Value>, StatusCode> {
    Ok(Json(read_static_json(&s, "history.json").await?))
}

async fn handle_validator(
    State(s): State<AppState>,
    AxPath(pubkey): AxPath<String>,
) -> Result<Json<Value>, StatusCode> {
    if pubkey.is_empty() || pubkey.len() > 64 {
        return Err(StatusCode::BAD_REQUEST);
    }
    let rows = db::fetch_validator_history(&s.pool, &pubkey, 1000)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let history: Vec<Value> = rows
        .into_iter()
        .map(|(slot, ts_local_us, drift_ms)| {
            json!({
                "slot": slot,
                "ts_local_us": ts_local_us,
                "drift_ms": drift_ms,
            })
        })
        .collect();
    Ok(Json(json!({
        "pubkey": pubkey,
        "n": history.len(),
        "samples": history,
    })))
}

async fn handle_health(State(s): State<AppState>) -> Result<Json<Value>, StatusCode> {
    let slot_obs = db::slot_obs_count(&s.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let latest = db::latest_slot(&s.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let errors = db::errors_in_last_hour(&s.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(json!({
        "status": "ok",
        "slot_obs_count": slot_obs,
        "latest_slot": latest,
        "errors_last_hour": errors,
        "version": env!("CARGO_PKG_VERSION"),
    })))
}
