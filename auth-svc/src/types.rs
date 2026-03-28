#[derive(Debug, serde::Deserialize)]
pub(crate) struct RequestCodeRequest {
    pub email: String,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct VerifyCodeRequest {
    pub hashed_email: String,
    pub code: String,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct VerifyCodeResponse {
    pub token: String,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct VerifyTokenRequest {
    pub hashed_email: String,
    pub token: String,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct LogoutRequest {
    pub hashed_email: String,
    pub token: String,
    pub logout_all: bool,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct LoginCountRequest {
    pub hashed_email: String,
    pub token: String,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct LoginCountResponse {
    pub count: usize,
}
