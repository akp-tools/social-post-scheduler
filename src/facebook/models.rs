use crate::responders::location::LocationResponder;

use rocket::response::Responder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TempResponse {
    pub access_token: FacebookAccessToken,
    pub debug_graph: FacebookDebugTokenGraphContainer,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FacebookAccessToken {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u32,
}

#[derive(Responder)]
pub enum RedirectResponse<T> {
    #[response(status = 500)]
    InternalServerError(&'static str),
    #[response(status = 400)]
    Unauthorized(&'static str),
    #[response(status = 307)]
    Redirect(LocationResponder),
    #[response(status = 200)]
    Ok(T),
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
