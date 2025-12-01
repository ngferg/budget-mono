use axum::{http, routing::get, Router};

#[tokio::main]
async fn main() {
    // our router
    let app = Router::new()
        .route("/health", get(health))
        .route("/users", get(get_users).post(create_user));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// which calls one of these handlers
async fn health() -> (http::StatusCode, axum::Json<serde_json::Value>) {
    println!("Performing health check...");
    let healthy = r#"
    {
      "status": "healthy"
    }
    "#;
    let json = serde_json::from_str(healthy).unwrap_or_default();
    (http::StatusCode::OK, axum::Json(json))
}

async fn get_users() {}

async fn create_user(
    axum::extract::Json(req): axum::extract::Json<budget_lib::types::CreateUserRequest>,
) -> http::StatusCode {
    let res = budget_lib::create_user(req).await;
    match res {
        Ok(()) => http::StatusCode::CREATED,
        Err(e) => match e {
            budget_lib::types::CreateUserError::EmailImproperlyFormatted() => {
                http::StatusCode::UNPROCESSABLE_ENTITY
            }
            budget_lib::types::CreateUserError::UserAlreadyExists() => http::StatusCode::CONFLICT,
            budget_lib::types::CreateUserError::Internal(_) => {
                http::StatusCode::INTERNAL_SERVER_ERROR
            }
        },
    }
}
