pub(crate) mod dao;

#[derive(thiserror::Error, Debug)]
pub enum CreateUserError {
    #[error("Email improperly formatted")]
    EmailImproperlyFormatted(),
    #[error("User already exists")]
    UserAlreadyExists(),
    #[error("Internal Error: {0}")]
    Internal(String),
}

#[derive(Debug, serde::Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
}
