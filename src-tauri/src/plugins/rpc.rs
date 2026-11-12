pub mod json_rpc {
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

        pub fn new(method: String, params: Map<String, Value>, id: Some(RpcId)) -> RpcRequest {
            RpcRequest {
                jsonrpc: String::from("2.0"),
                method,
                params,
                id,
            }
        }
    }

    // TODO Implement Traits for SERDE_JSON and Vec<u8> for easier manipulation
}
