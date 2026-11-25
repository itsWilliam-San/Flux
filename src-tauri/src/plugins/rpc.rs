use crate::error::TransportError;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum RpcId {
    Number(i64),
    String(String),
    Null,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum RpcResponse {
    Error {
        id: Option<RpcId>,
        error: RpcError,
    },
    Success {
        id: Option<RpcId>,
        result: serde_json::Value,
    },
    // A response with no id and no result/error field — treated as a no-op notification.
    Notification,
}

impl RpcResponse {
    pub fn from_bytes(payload: &[u8]) -> Result<RpcResponse, TransportError> {
        let rpc_response: RpcResponse = serde_json::from_slice(payload)?;
        Ok(rpc_response)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcRequest {
    pub method: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Map<String, serde_json::Value>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RpcId>,
}

impl RpcRequest {
    pub fn is_notification(&self) -> bool {
        self.id.is_none()
    }

    pub fn new(method: String, params: Option<serde_json::Map<String, serde_json::Value>>, id: Option<RpcId>) -> Self {
        RpcRequest {
            method,
            params,
            id,
        }
    }
}
