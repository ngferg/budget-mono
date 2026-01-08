use axum::{
    Router, http,
    routing::{delete, get, post},
};

#[tokio::main]
async fn main() {
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
        );

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

async fn find_budget(
    axum::extract::Json(req): axum::extract::Json<budget_lib::types::GetBudgetRequest>,
) -> (http::StatusCode, axum::Json<serde_json::Value>) {
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
        },
    }
}

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

async fn delete_user(
    axum::extract::Json(req): axum::extract::Json<budget_lib::types::DeleteUserRequest>,
) -> http::StatusCode {
    let res = budget_lib::delete_user(req).await;
    match res {
        Ok(()) => http::StatusCode::NO_CONTENT,
        Err(_) => http::StatusCode::INTERNAL_SERVER_ERROR,
    }
}

async fn delete_line_item(
    axum::extract::Json(req): axum::extract::Json<budget_lib::types::DeleteLineItemRequest>,
) -> http::StatusCode {
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
    axum::extract::Json(req): axum::extract::Json<budget_lib::types::AddLineItemRequest>,
) -> http::StatusCode {
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
    axum::extract::Json(req): axum::extract::Json<budget_lib::types::EditLineItemRequest>,
) -> http::StatusCode {
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
