#[derive(Debug, serde::Deserialize)]
pub(crate) struct RequestTokenRequest {
    pub email: String,
}
