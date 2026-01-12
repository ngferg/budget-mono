#[derive(Debug, serde::Deserialize)]
pub(crate) struct RequestCodeRequest {
    pub email: String,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct VerifyCodeRequest {
    pub email: String,
    pub code: String,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct VerifyCodeResponse {
    pub token: String,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct VerifyTokenRequest {
    pub email: String,
    pub token: String,
}
