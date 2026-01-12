use axum::extract::{Json, State};
use axum::routing;
use rand::Rng;
use rand::distributions::{Alphanumeric, DistString};
mod types;

#[derive(Clone)]
struct AppState {
    code_map: std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<String, String>>>,
    token_map: std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<String, String>>>,
}

#[tokio::main]
async fn main() {
    let state = AppState {
        code_map: std::sync::Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        token_map: std::sync::Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
    };
    // our router
    let app = axum::Router::new()
        .route("/health", routing::get(health))
        .route("/request_code", routing::post(request_code))
        .route("/verify_code", routing::post(verify_code))
        .route("/verify_token", routing::post(verify_token))
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

async fn request_code(
    State(state): State<AppState>,
    Json(req): Json<types::RequestCodeRequest>,
) -> axum::http::StatusCode {
    let email_sha = sha256::digest(req.email.clone());
    // Generate a random 6-digit token in a short scope so RNG doesn't live across `.await`
    let code = {
        let mut rng = rand::thread_rng();
        let random_6_digit_number = rng.gen_range(100000..1000000);
        random_6_digit_number.to_string()
    };
    {
        let mut code_map = state.code_map.write().await;
        code_map.insert(email_sha.clone(), code.clone());
    }
    println!("Generated code {code} for email {email_sha}");
    // TODO: Send email with token
    axum::http::StatusCode::OK
}

async fn verify_code(
    State(state): State<AppState>,
    Json(req): Json<types::VerifyCodeRequest>,
) -> (axum::http::StatusCode, axum::Json<serde_json::Value>) {
    println!("Login request: {:?}", req);
    let code_map = state.code_map.read().await;
    let code = code_map.get(&sha256::digest(req.email.clone()));
    match code {
        Some(stored_code) => {
            if *stored_code == req.code {
                let token = Alphanumeric.sample_string(&mut rand::thread_rng(), 64);
                let res = types::VerifyCodeResponse { token };
                {
                    let mut token_map = state.token_map.write().await;
                    token_map.insert(sha256::digest(req.email.clone()), res.token.clone());
                }
                return (
                    axum::http::StatusCode::OK,
                    axum::Json(serde_json::to_value(res).expect("json conversion should work")),
                );
            } else {
                return (
                    axum::http::StatusCode::UNAUTHORIZED,
                    axum::Json(serde_json::from_str("{}").unwrap_or_default()),
                );
            }
        }
        _ => (
            axum::http::StatusCode::UNAUTHORIZED,
            axum::Json(serde_json::from_str("{}").unwrap_or_default()),
        ),
    }
}

async fn verify_token(
    State(state): State<AppState>,
    Json(req): Json<types::VerifyTokenRequest>,
) -> axum::http::StatusCode {
    println!("Verify token request: {:?}", req);
    let token_map = state.token_map.read().await;
    let stored_token = token_map.get(&sha256::digest(req.email.clone()));
    match stored_token {
        Some(t) => {
            if *t == req.token {
                axum::http::StatusCode::OK
            } else {
                axum::http::StatusCode::UNAUTHORIZED
            }
        }
        _ => axum::http::StatusCode::UNAUTHORIZED,
    }
}
