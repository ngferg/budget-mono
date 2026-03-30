use axum::extract::{Json, State};
use axum::routing;
use envconfig::Envconfig;
use rand::Rng;
use rand::distributions::{Alphanumeric, DistString};
use std::collections::HashSet;
use tower_http::cors::CorsLayer;
mod types;

#[derive(Envconfig)]
struct Config {
    smtp_user: String,
    smtp_pass: String,
    smtp_host: String,
}

const CODE_EXPIRATION_MINUTES: i64 = 10;
const CODE_CLEANUP_INTERVAL_SECONDS: u64 = 60;
const TOKEN_EXPIRATION_DAYS: i64 = 7;
const TOKEN_CLEANUP_INTERVAL_SECONDS: u64 = 3600;

#[derive(Clone)]
struct AppState {
    config: std::sync::Arc<Config>,
    code_map: std::sync::Arc<
        tokio::sync::RwLock<
            std::collections::HashMap<String, (String, chrono::DateTime<chrono::Utc>)>,
        >,
    >,
    token_map: std::sync::Arc<
        tokio::sync::RwLock<
            std::collections::HashMap<String, (HashSet<String>, chrono::DateTime<chrono::Utc>)>,
        >,
    >,
}

#[tokio::main]
async fn main() {
    let _ = rustls::crypto::ring::default_provider().install_default();

    let config = Config::init_from_env().expect("Failed to load config from environment");
    let state = AppState {
        config: std::sync::Arc::new(config),
        code_map: std::sync::Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        token_map: std::sync::Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
    };

    let token_cleanup_monitor = tokio::spawn({
        println!(
            "Starting token cleanup task with an interval of {TOKEN_CLEANUP_INTERVAL_SECONDS} seconds"
        );
        let state = state.clone();
        async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(
                    TOKEN_CLEANUP_INTERVAL_SECONDS,
                ))
                .await;
                let mut token_map = state.token_map.write().await;
                let now = chrono::Utc::now();
                token_map.retain(|_, v| {
                    now.signed_duration_since(v.1) < chrono::Duration::days(TOKEN_EXPIRATION_DAYS)
                });
            }
        }
    });

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
        .route("/login_count", routing::post(login_count))
        .route("/logout", routing::post(logout))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    println!("Auth service starting on port 3001");
    axum::serve(listener, app).await.unwrap();
    cache_cleanup_monitor.abort();
    token_cleanup_monitor.abort();
}

// which calls one of these handlers
async fn health() -> (axum::http::StatusCode, axum::Json<serde_json::Value>) {
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
    let from_email = state.config.smtp_user.as_str();
    let smtp_pass = state.config.smtp_pass.as_str();

    let message = mail_send::mail_builder::MessageBuilder::new()
        .from(("FeBudget", from_email))
        .to(to_addr.as_str())
        .subject("Here's your login code")
        .text_body(format!("Your login code is: {code}"))
        .html_body(format!("<p>Your login code is: {code}</p>"));

    // Connect to the SMTP submissions port, upgrade to TLS and
    // authenticate using the provided credentials.
    let smtp_client = mail_send::SmtpClientBuilder::new(state.config.smtp_host.as_str(), 587)
        .implicit_tls(false)
        .credentials((from_email, smtp_pass))
        .connect()
        .await;
    match smtp_client {
        Ok(mut smtp_client) => {
            let res = smtp_client
                .send(message)
                .await
                .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR);
            match res {
                Ok(_) => axum::http::StatusCode::OK,
                Err(e) => e,
            }
        }
        Err(_) => return axum::http::StatusCode::INTERNAL_SERVER_ERROR,
    }
}

async fn verify_code(
    State(state): State<AppState>,
    Json(req): Json<types::VerifyCodeRequest>,
) -> (axum::http::StatusCode, axum::Json<serde_json::Value>) {
    let code_entry = {
        let mut code_map = state.code_map.write().await;
        code_map.remove(&req.hashed_email)
    };
    match code_entry {
        Some((code, _)) => {
            if code == req.code {
                let token = Alphanumeric.sample_string(&mut rand::thread_rng(), 64);
                let res = types::VerifyCodeResponse { token };
                {
                    let mut token_map = state.token_map.write().await;
                    token_map
                        .entry(req.hashed_email)
                        .and_modify(|v| {
                            v.0.insert(res.token.clone());
                            v.1 = chrono::Utc::now();
                        })
                        .or_insert_with(|| {
                            let mut set = HashSet::new();
                            set.insert(res.token.clone());
                            (set, chrono::Utc::now())
                        });
                }
                (
                    axum::http::StatusCode::OK,
                    axum::Json(serde_json::to_value(res).unwrap_or_default()),
                )
            } else {
                (
                    axum::http::StatusCode::UNAUTHORIZED,
                    axum::Json(serde_json::from_str("{}").unwrap_or_default()),
                )
            }
        }
        None => (
            axum::http::StatusCode::UNAUTHORIZED,
            axum::Json(serde_json::from_str("{}").unwrap_or_default()),
        ),
    }
}

async fn verify_token(
    State(state): State<AppState>,
    Json(req): Json<types::VerifyTokenRequest>,
) -> axum::http::StatusCode {
    let mut token_map = state.token_map.write().await;
    match token_map.entry(req.hashed_email.clone()) {
        std::collections::hash_map::Entry::Occupied(mut entry) => {
            if entry.get().0.contains(&req.token) {
                entry.get_mut().1 = chrono::Utc::now();
                axum::http::StatusCode::OK
            } else {
                axum::http::StatusCode::UNAUTHORIZED
            }
        }
        std::collections::hash_map::Entry::Vacant(_) => axum::http::StatusCode::UNAUTHORIZED,
    }
}

async fn logout(
    State(state): State<AppState>,
    Json(req): Json<types::LogoutRequest>,
) -> axum::http::StatusCode {
    let stored_token = {
        let token_map = state.token_map.read().await;
        token_map.get(&req.hashed_email).cloned()
    };
    match stored_token {
        Some(t) => {
            if t.0.contains(&req.token) {
                let mut token_map = state.token_map.write().await;
                if let Some(entry) = token_map.get_mut(&req.hashed_email) {
                    if req.logout_all {
                        entry.0.clear();
                    } else {
                        entry.0.remove(&req.token);
                    }
                }
                token_map.retain(|_, v| !v.0.is_empty());
                axum::http::StatusCode::OK
            } else {
                axum::http::StatusCode::UNAUTHORIZED
            }
        }
        _ => axum::http::StatusCode::UNAUTHORIZED,
    }
}

async fn login_count(
    State(state): State<AppState>,
    Json(req): Json<types::LoginCountRequest>,
) -> (
    axum::http::StatusCode,
    axum::Json<types::LoginCountResponse>,
) {
    let stored_tokens = {
        let token_map = state.token_map.read().await;
        token_map.get(&req.hashed_email).map(|v| v.0.clone())
    };
    match stored_tokens {
        Some(tokens) => {
            if tokens.contains(&req.token) {
                (
                    axum::http::StatusCode::OK,
                    axum::Json(types::LoginCountResponse {
                        count: tokens.len(),
                    }),
                )
            } else {
                (
                    axum::http::StatusCode::UNAUTHORIZED,
                    axum::Json(types::LoginCountResponse { count: 0 }),
                )
            }
        }
        None => (
            axum::http::StatusCode::UNAUTHORIZED,
            axum::Json(types::LoginCountResponse { count: 0 }),
        ),
    }
}
