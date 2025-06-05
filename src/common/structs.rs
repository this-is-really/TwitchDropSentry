use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug)]
pub struct ClientInfo {
    pub url: String,
    pub id: String,
    pub user_agent: String,
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
pub struct GQLoperation {
    pub operationName: String,
    pub extensions: Extensions,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<Value>,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize)]
pub struct Extensions {
    pub persistedQuery: PersistedQuery,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize)]
pub struct PersistedQuery {
    pub version: i32,
    pub sha256Hash: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Token {
    pub userid: String,
    pub oauth: String,
}