use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct FacebookRedirect {
    pub code: String,
    pub state: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FacebookAccessToken {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FacebookDebugTokenGraphContainer {
    pub data: FacebookDebugTokenGraph,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FacebookDebugTokenGraph {
    pub app_id: String,
    #[serde(rename(deserialize = "type", serialize = "type"))]
    pub type_: String,
    pub application: String,
    pub data_access_expires_at: i32,
    pub expires_at: i32,
    pub is_valid: bool,
    pub issued_at: i32,
    pub scopes: Vec<String>,
    pub granular_scopes: Vec<FacebookGranularScope>,
    pub user_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FacebookGranularScope {
    pub scope: String,
    pub target_ids: Vec<String>,
}
