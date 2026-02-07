use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use polarisdb_core::{
    AsyncCollection, CollectionConfig, DistanceMetric, Payload, VectorId,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::net::SocketAddr;

#[derive(Clone)]
struct AppState {
    collections: Arc<RwLock<HashMap<String, AsyncCollection>>>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let state = AppState {
        collections: Arc::new(RwLock::new(HashMap::new())),
    };

    let app = Router::new()
        .route("/collections/:name", post(create_collection))
        .route("/collections/:name/insert", post(insert_vector))
        .route("/collections/:name/search", post(search_vector))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Deserialize)]
struct CreateCollectionRequest {
    dimension: usize,
    metric: String,
}

async fn create_collection(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(payload): Json<CreateCollectionRequest>,
) -> impl IntoResponse {
    let metric = match payload.metric.as_str() {
        "cosine" => DistanceMetric::Cosine,
        "euclidean" => DistanceMetric::Euclidean,
        "dot" => DistanceMetric::DotProduct,
        _ => return (StatusCode::BAD_REQUEST, "Invalid metric").into_response(),
    };

    let config = CollectionConfig::new(payload.dimension, metric);
    let path = format!("./data/{}", name);
    
    // In a real app, handle error properly
    let collection = AsyncCollection::open_or_create(path, config).await.unwrap();

    state.collections.write().unwrap().insert(name.clone(), collection);

    (StatusCode::CREATED, format!("Collection {} created", name)).into_response()
}

#[derive(Deserialize)]
struct InsertRequest {
    id: VectorId,
    vector: Vec<f32>,
    payload: Option<serde_json::Value>,
}

async fn insert_vector(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(req): Json<InsertRequest>,
) -> impl IntoResponse {
    let collection = {
        let collections = state.collections.read().unwrap();
        match collections.get(&name) {
            Some(c) => c.clone(),
            None => return (StatusCode::NOT_FOUND, "Collection not found").into_response(),
        }
    };

    let payload = if let Some(p) = req.payload {
        if let serde_json::Value::Object(map) = p {
            let map: HashMap<String, serde_json::Value> = map.into_iter().collect();
            Payload::from_map(map)
        } else {
            return (StatusCode::BAD_REQUEST, "Payload must be a JSON object").into_response();
        }
    } else {
        Payload::new()
    };

    if let Err(e) = collection.insert(req.id, req.vector, payload).await {
         return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
    }

    StatusCode::OK.into_response()
}

#[derive(Deserialize)]
struct SearchRequest {
    vector: Vec<f32>,
    k: usize,
}

#[derive(Serialize)]
struct SearchResponse {
    id: VectorId,
    distance: f32,
}

async fn search_vector(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(req): Json<SearchRequest>,
) -> impl IntoResponse {
    let collection = {
        let collections = state.collections.read().unwrap();
        match collections.get(&name) {
            Some(c) => c.clone(),
            None => return (StatusCode::NOT_FOUND, "Collection not found").into_response(),
        }
    };

    let results = collection.search(&req.vector, req.k, None).await;
    
    let response: Vec<SearchResponse> = results.into_iter().map(|r| SearchResponse {
        id: r.id,
        distance: r.distance,
    }).collect();

    Json(response).into_response()
}
