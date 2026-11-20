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
#[serde(untagged)]
pub enum RpcParams {
    ByName(serde_json::Map<String, serde_json::Value>),
    ByPosition(Vec<serde_json::Value>),
}

impl From<serde_json::Map<String, serde_json::Value>> for RpcParams {
    fn from(value: serde_json::Map<String, serde_json::Value>) -> Self {
        Self::ByName(value)
    }
}

impl From<Vec<serde_json::Value>> for RpcParams {
    fn from(value: Vec<serde_json::Value>) -> Self {
        Self::ByPosition(value)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl RpcResponse {
    pub fn new(payload: &Vec<u8>) -> Result<RpcResponse, TransportError> {
        let rpc_response: RpcResponse = serde_json::from_slice(payload)?;

        Ok(rpc_response)
    }
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum RpcResponse {
    Success {
        jsonrpc: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<RpcId>,
        result: serde_json::Value,
    },
    Error {
        jsonrpc: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<RpcId>,
        error: RpcError,
    },
    Notification,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub method: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<RpcParams>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RpcId>,
}

impl RpcRequest {
    pub fn is_notification(&self) -> bool {
        self.id.is_none()
    }

    pub fn new(method: String, params: Option<RpcParams>, id: Option<RpcId>) -> Self {
        RpcRequest {
            jsonrpc: String::from("2.0"),
            method,
            params,
            id,
        }
    }
}
