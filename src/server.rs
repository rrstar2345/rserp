use crate::engines::Engine;
use crate::models::{Envelope, Query};
use axum::{
    extract::{Query as AxumQuery},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    // Can add shared state like caches, clients, etc.
}

#[derive(Deserialize)]
pub struct SearchParams {
    pub text: String,
    pub engines: Option<String>,
    pub limit: Option<usize>,
    pub lang: Option<String>,
    pub region: Option<String>,
}

pub async fn start_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let app_state = Arc::new(AppState {});

    let app = Router::new()
        .route("/search", get(search_handler))
        .route("/duckduckgo/search", get(duckduckgo_search_handler))
        .route("/bing/search", get(bing_search_handler))
        .route("/health", get(health_handler))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
    tracing::info!("Server listening on http://127.0.0.1:{}", port);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn search_handler(
    AxumQuery(params): AxumQuery<SearchParams>,
) -> impl IntoResponse {
    let engines = params
        .engines
        .as_deref()
        .unwrap_or("duckduckgo,bing")
        .split(',')
        .map(|e| e.trim().to_lowercase())
        .collect::<Vec<_>>();

    let limit = params.limit.unwrap_or(10);

    let query = Query {
        text: params.text,
        lang: params.lang,
        region: params.region,
        limit,
    };

    let request_id = Uuid::new_v4().to_string();
    let engine_names: Vec<String> = engines.iter().map(|e| e.clone()).collect();
    let mut envelope = Envelope::new(&query, request_id, engine_names);

    let start = std::time::Instant::now();

    for engine_name in engines {
        if let Some(engine) = Engine::from_string(&engine_name) {
            match engine.search(&query.text, limit).await {
                Ok(results) => {
                    envelope.meta.engines_responded.push(engine_name.clone());
                    envelope.results.extend(results);
                }
                Err(e) => {
                    envelope.meta.engines_failed.push(engine_name);
                    tracing::warn!("Engine failed: {}", e);
                }
            }
        } else {
            envelope
                .meta
                .engines_failed
                .push(format!("unknown: {}", engine_name));
        }
    }

    envelope.finalize(start);

    if envelope.results.is_empty() {
        (StatusCode::NOT_FOUND, Json(envelope))
    } else {
        (StatusCode::OK, Json(envelope))
    }
}

async fn duckduckgo_search_handler(
    AxumQuery(params): AxumQuery<SearchParams>,
) -> impl IntoResponse {
    let limit = params.limit.unwrap_or(10);
    let query = Query {
        text: params.text,
        lang: params.lang,
        region: params.region,
        limit,
    };

    let request_id = Uuid::new_v4().to_string();
    let mut envelope = Envelope::new(&query, request_id, vec!["duckduckgo".to_string()]);

    let start = std::time::Instant::now();

    match Engine::DuckDuckGo.search(&query.text, limit).await {
        Ok(results) => {
            envelope.meta.engines_responded.push("duckduckgo".to_string());
            envelope.results = results;
        }
        Err(e) => {
            envelope.meta.engines_failed.push("duckduckgo".to_string());
            tracing::warn!("DuckDuckGo search failed: {}", e);
        }
    }

    envelope.finalize(start);

    if envelope.results.is_empty() {
        (StatusCode::NOT_FOUND, Json(envelope))
    } else {
        (StatusCode::OK, Json(envelope))
    }
}

async fn bing_search_handler(
    AxumQuery(params): AxumQuery<SearchParams>,
) -> impl IntoResponse {
    let limit = params.limit.unwrap_or(10);
    let query = Query {
        text: params.text,
        lang: params.lang,
        region: params.region,
        limit,
    };

    let request_id = Uuid::new_v4().to_string();
    let mut envelope = Envelope::new(&query, request_id, vec!["bing".to_string()]);

    let start = std::time::Instant::now();

    match Engine::Bing.search(&query.text, limit).await {
        Ok(results) => {
            envelope.meta.engines_responded.push("bing".to_string());
            envelope.results = results;
        }
        Err(e) => {
            envelope.meta.engines_failed.push("bing".to_string());
            tracing::warn!("Bing search failed: {}", e);
        }
    }

    envelope.finalize(start);

    if envelope.results.is_empty() {
        (StatusCode::NOT_FOUND, Json(envelope))
    } else {
        (StatusCode::OK, Json(envelope))
    }
}

async fn health_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "version": "0.1.0"
    }))
}
