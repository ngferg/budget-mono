use axum::extract::{Json, State};
use axum::routing;
use rand::Rng;
mod types;

#[derive(Clone)]
struct AppState {
    token_map: std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<String, String>>>,
    login_map: std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<String, String>>>,
}

#[tokio::main]
async fn main() {
    let state = AppState {
        token_map: std::sync::Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        login_map: std::sync::Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
    };
    // our router
    let app = axum::Router::new()
        .route("/health", routing::get(health))
        .route("/request_token", routing::post(request_token))
        .route("/login", routing::post(login))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// which calls one of these handlers
async fn health() -> (axum::http::StatusCode, axum::Json<serde_json::Value>) {
    println!("Performing health check...");
    let healthy = r#"
    {
      "status": "healthy"
    }
    "#;
    let json = serde_json::from_str(healthy).unwrap_or_default();
    (axum::http::StatusCode::OK, axum::Json(json))
}

async fn request_token(
    State(state): State<AppState>,
    Json(req): Json<types::RequestTokenRequest>,
) -> axum::http::StatusCode {
    let email_sha = sha256::digest(req.email.clone());
    // Generate a random 6-digit token in a short scope so RNG doesn't live across `.await`
    let token = {
        let mut rng = rand::thread_rng();
        let random_6_digit_number = rng.gen_range(100000..1000000);
        random_6_digit_number.to_string()
    };
    {
        let mut token_map = state.token_map.write().await;
        token_map.insert(email_sha.clone(), token.clone());
    }
    println!("Generated token {token} for email {email_sha}");
    // TODO: Send email with token
    axum::http::StatusCode::OK
}

async fn login(
    State(_state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> axum::http::StatusCode {
    println!("Login request: {:?}", req);
    axum::http::StatusCode::OK
}
