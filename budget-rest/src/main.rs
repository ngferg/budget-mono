mod dao;
mod types;

use axum::{Router, http, routing::get};

#[tokio::main]
async fn main() {
    // our router
    let app = Router::new()
        .route("/", get(root))
        .route("/users", get(get_users).post(create_user));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// which calls one of these handlers
async fn root() {
    println!("hello");
}
async fn get_users() {}
async fn create_user(
    axum::extract::Json(req): axum::extract::Json<types::dao::CreateUserRequest>,
) -> http::StatusCode {
    use dao::Dao as dao_trait;

    let dao = dao::sqlite_dao::SqliteDao::new();
    let res = dao.create_user(&req);
    match res {
        Ok(()) => http::StatusCode::CREATED,
        Err(e) => match e {
            types::dao::CreateUserError::EmailImproperlyFormatted() => {
                http::StatusCode::UNPROCESSABLE_ENTITY
            }
            types::dao::CreateUserError::UserAlreadyExists() => {
                http::StatusCode::CONFLICT
            }
            types::dao::CreateUserError::Internal(_) => {
                http::StatusCode::INTERNAL_SERVER_ERROR
            }
        },
    }
}
