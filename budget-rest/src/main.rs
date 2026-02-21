use axum::{
    Router, http,
    routing::{delete, get, post},
};
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    // CORS configuration
    let cors = CorsLayer::permissive();
    // our router
    let app = Router::new()
        .route("/health", get(health))
        .route("/users", post(create_user).delete(delete_user))
        .route("/users/budget", post(find_budget))
        .route(
            "/users/budget/line_item",
            delete(delete_line_item)
                .post(add_line_item)
                .put(edit_line_item),
        )
        .route("/users/budget/clone_month", post(clone_month))
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Budget service started on port 3000");
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

async fn find_budget(
    headers: axum::http::HeaderMap,
    axum::extract::Json(req): axum::extract::Json<budget_lib::types::GetBudgetRequest>,
) -> (http::StatusCode, axum::Json<serde_json::Value>) {
    if let Err(e) = verify_auth(headers, &req.email.as_str()).await {
        return (
            e,
            axum::Json(serde_json::from_str("{}").unwrap_or_default()),
        );
    }
    let res = budget_lib::get_budget(req).await;
    match res {
        Ok(res) => (
            http::StatusCode::OK,
            axum::Json(serde_json::to_value(res).expect("json conversion should work")),
        ),
        Err(e) => match e {
            budget_lib::types::GetBudgetError::Internal(e) => (
                http::StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::to_value(e).expect("json conversion should work")),
            ),
            budget_lib::types::GetBudgetError::UserDoesntExists() => (
                http::StatusCode::NOT_FOUND,
                axum::Json(serde_json::from_str("{}").unwrap_or_default()),
            ),
            budget_lib::types::GetBudgetError::BudgetDoesntExists() => (
                http::StatusCode::NOT_FOUND,
                axum::Json(serde_json::from_str("{}").unwrap_or_default()),
            ),
            budget_lib::types::GetBudgetError::DateError(e) => (
                http::StatusCode::BAD_REQUEST,
                axum::Json(
                    serde_json::from_str(format!("{{\"error\": \"{e}\"}}").as_str())
                        .unwrap_or_default(),
                ),
            ),
        },
    }
}

async fn create_user(
    headers: axum::http::HeaderMap,
    axum::extract::Json(req): axum::extract::Json<budget_lib::types::CreateUserRequest>,
) -> http::StatusCode {
    if let Err(e) = verify_auth(headers, &req.email.as_str()).await {
        return e;
    }
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

async fn delete_user(
    headers: axum::http::HeaderMap,
    axum::extract::Json(req): axum::extract::Json<budget_lib::types::DeleteUserRequest>,
) -> http::StatusCode {
    if let Err(e) = verify_auth(headers, &req.email.as_str()).await {
        return e;
    }
    let res = budget_lib::delete_user(req).await;
    match res {
        Ok(()) => http::StatusCode::NO_CONTENT,
        Err(_) => http::StatusCode::INTERNAL_SERVER_ERROR,
    }
}

async fn delete_line_item(
    headers: axum::http::HeaderMap,
    axum::extract::Json(req): axum::extract::Json<budget_lib::types::DeleteLineItemRequest>,
) -> http::StatusCode {
    if let Err(e) = verify_auth(headers, &req.email.as_str()).await {
        return e;
    }
    let res = budget_lib::delete_line_item(req).await;
    match res {
        Ok(()) => http::StatusCode::NO_CONTENT,
        Err(e) => match e {
            budget_lib::types::DeleteLineItemError::UserDoesntExists() => {
                http::StatusCode::NOT_FOUND
            }
            budget_lib::types::DeleteLineItemError::LineItemDoesntExist() => {
                http::StatusCode::NOT_FOUND
            }
            budget_lib::types::DeleteLineItemError::Internal(_) => {
                http::StatusCode::INTERNAL_SERVER_ERROR
            }
        },
    }
}

async fn add_line_item(
    headers: axum::http::HeaderMap,
    axum::extract::Json(req): axum::extract::Json<budget_lib::types::AddLineItemRequest>,
) -> http::StatusCode {
    if let Err(e) = verify_auth(headers, &req.email.as_str()).await {
        return e;
    }
    let res = budget_lib::add_line_item(req).await;
    match res {
        Ok(()) => http::StatusCode::CREATED,
        Err(e) => match e {
            budget_lib::types::AddLineItemError::UserDoesntExists() => http::StatusCode::NOT_FOUND,
            budget_lib::types::AddLineItemError::Internal(_) => {
                http::StatusCode::INTERNAL_SERVER_ERROR
            }
        },
    }
}

async fn edit_line_item(
    headers: axum::http::HeaderMap,
    axum::extract::Json(req): axum::extract::Json<budget_lib::types::EditLineItemRequest>,
) -> http::StatusCode {
    if let Err(e) = verify_auth(headers, &req.email.as_str()).await {
        return e;
    }
    let res = budget_lib::edit_line_item(req).await;
    match res {
        Ok(()) => http::StatusCode::OK,
        Err(e) => match e {
            budget_lib::types::EditLineItemError::UserDoesntExists() => http::StatusCode::NOT_FOUND,
            budget_lib::types::EditLineItemError::LineItemDoesntExist() => {
                http::StatusCode::NOT_FOUND
            }
            budget_lib::types::EditLineItemError::Internal(_) => {
                http::StatusCode::INTERNAL_SERVER_ERROR
            }
        },
    }
}

async fn clone_month(
    headers: axum::http::HeaderMap,
    axum::extract::Json(req): axum::extract::Json<budget_lib::types::CloneMonthRequest>,
) -> http::StatusCode {
    if let Err(e) = verify_auth(headers, &req.email.as_str()).await {
        return e;
    }
    let res = budget_lib::clone_last_month(req).await;
    match res {
        Ok(()) => http::StatusCode::CREATED,
        Err(e) => match e {
            budget_lib::types::CloneMonthError::UserDoesntExists() => http::StatusCode::NOT_FOUND,
            budget_lib::types::CloneMonthError::SourceMonthDoesntExist() => {
                http::StatusCode::NOT_FOUND
            }
            budget_lib::types::CloneMonthError::Internal(_) => {
                http::StatusCode::INTERNAL_SERVER_ERROR
            }
        },
    }
}

async fn verify_auth(headers: axum::http::HeaderMap, email: &str) -> Result<(), http::StatusCode> {
    let auth = headers.get("Authorization");
    match auth {
        None => {
            return Err(http::StatusCode::UNAUTHORIZED);
        }
        Some(token) => {
            let token_str = token.to_str().unwrap_or("");
            let res = budget_lib::check_token(email, token_str).await;
            match res {
                Err(_) => {
                    return Err(http::StatusCode::UNAUTHORIZED);
                }
                Ok(()) => Ok(()),
            }
        }
    }
}
