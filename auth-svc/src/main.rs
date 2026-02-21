use axum::extract::{Json, State};
use axum::routing;
use rand::Rng;
use rand::distributions::{Alphanumeric, DistString};
use tower_http::cors::CorsLayer;
mod types;

static FROM_ADDR: std::sync::OnceLock<String> = std::sync::OnceLock::new();
static SMTP_PASSWORD: std::sync::OnceLock<String> = std::sync::OnceLock::new();
static SMTP_HOST: std::sync::OnceLock<String> = std::sync::OnceLock::new();

const CODE_EXPIRATION_MINUTES: i64 = 10;
const CODE_CLEANUP_INTERVAL_SECONDS: u64 = 60;

#[derive(Clone)]
struct AppState {
    code_map: std::sync::Arc<
        tokio::sync::RwLock<
            std::collections::HashMap<String, (String, chrono::DateTime<chrono::Utc>)>,
        >,
    >,
    token_map: std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<String, String>>>,
}

#[tokio::main]
async fn main() {
    let _ = rustls::crypto::ring::default_provider().install_default();

    FROM_ADDR
        .set(std::env::var("SMTP_USER").expect("SMTP_USER env var must be set"))
        .unwrap();
    SMTP_PASSWORD
        .set(std::env::var("SMTP_PASS").expect("SMTP_PASS env var must be set"))
        .unwrap();
    SMTP_HOST
        .set(std::env::var("SMTP_HOST").expect("SMTP_HOST env var must be set"))
        .unwrap();
    let state = AppState {
        code_map: std::sync::Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        token_map: std::sync::Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
    };

    let cache_cleanup_monitor = tokio::spawn({
        println!(
            "Starting code cleanup task with an interval of {CODE_CLEANUP_INTERVAL_SECONDS} seconds"
        );
        let state = state.clone();
        async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(
                    CODE_CLEANUP_INTERVAL_SECONDS,
                ))
                .await;
                let mut code_map = state.code_map.write().await;
                let now = chrono::Utc::now();
                code_map.retain(|_, v| {
                    now.signed_duration_since(v.1)
                        < chrono::Duration::minutes(CODE_EXPIRATION_MINUTES)
                });
            }
        }
    });

    // CORS configuration
    let cors = CorsLayer::permissive();
    // our router
    let app = axum::Router::new()
        .route("/health", routing::get(health))
        .route("/request_code", routing::post(request_code))
        .route("/verify_code", routing::post(verify_code))
        .route("/verify_token", routing::post(verify_token))
        .route("/logout", routing::post(logout))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    println!("Auth service starting on port 3001");
    axum::serve(listener, app).await.unwrap();
    cache_cleanup_monitor.abort();
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
        code_map.insert(email_sha.clone(), (code.clone(), chrono::Utc::now()));
    }
    // Build a simple multipart message
    let to_addr = req.email.clone();
    let from_email = FROM_ADDR.get().map(|s| s.as_str()).unwrap_or("");
    let smtp_pass = SMTP_PASSWORD.get().map(|s| s.as_str()).unwrap_or("");

    let message = mail_send::mail_builder::MessageBuilder::new()
        .from(("FeBudget", from_email))
        .to(to_addr.as_str())
        .subject("Here's your login code")
        .text_body(format!("Your login code is: {code}"))
        .html_body(format!("<p>Your login code is: {code}</p>"));

    // Connect to the SMTP submissions port, upgrade to TLS and
    // authenticate using the provided credentials.
    let res =
        mail_send::SmtpClientBuilder::new(SMTP_HOST.get().map(|s| s.as_str()).unwrap_or(""), 587)
            .implicit_tls(false)
            .credentials((from_email, smtp_pass))
            .connect()
            .await
            .expect("Failed to connect to SMTP server")
            .send(message)
            .await
            .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR);
    match res {
        Ok(_) => axum::http::StatusCode::OK,
        Err(e) => e,
    }
}

async fn verify_code(
    State(state): State<AppState>,
    Json(req): Json<types::VerifyCodeRequest>,
) -> (axum::http::StatusCode, axum::Json<serde_json::Value>) {
    let code_map = state.code_map.read().await;
    let hashed_email = sha256::digest(req.email.clone());
    let code = code_map.get(&hashed_email);
    match code {
        Some(stored_code) => {
            if *stored_code.0 == req.code {
                drop(code_map);
                state.code_map.write().await.remove(&hashed_email);
                let token = Alphanumeric.sample_string(&mut rand::thread_rng(), 64);
                let res = types::VerifyCodeResponse { token };
                {
                    let mut token_map = state.token_map.write().await;
                    token_map.insert(hashed_email, res.token.clone());
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

async fn logout(
    State(state): State<AppState>,
    Json(req): Json<types::VerifyTokenRequest>,
) -> axum::http::StatusCode {
    println!("Logout request {req:?}");
    let token_map = state.token_map.read().await;
    let stored_token = token_map.get(&sha256::digest(req.email.clone()));
    match stored_token {
        Some(t) => {
            if *t == req.token {
                drop(token_map);
                state
                    .token_map
                    .write()
                    .await
                    .remove(&sha256::digest(req.email.clone()));
                axum::http::StatusCode::OK
            } else {
                axum::http::StatusCode::UNAUTHORIZED
            }
        }
        _ => axum::http::StatusCode::UNAUTHORIZED,
    }
}
