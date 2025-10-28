#[derive(Debug, serde::Deserialize)]
pub(crate) struct CreateUserRequest {
    pub email: String,
}

#[derive(thiserror::Error, Debug)]
pub(crate) enum CreateUserError {
    #[error("Email improperly formatted")]
    EmailImproperlyFormatted(),
}
