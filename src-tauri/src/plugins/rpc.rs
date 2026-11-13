use crate::JsonRpcError::{self, NotValidRPCParams};
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
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

    pub fn new(
        method: String,
        params: serde_json::Map<String, serde_json::Value>,
        id: Option<RpcId>,
    ) -> RpcRequest {
        let my_params = Some(RpcParams::ByName(params));
        RpcRequest {
            jsonrpc: String::from("2.0"),
            method,
            params: my_params,
            id,
        }
    }
}
