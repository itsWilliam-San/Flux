pub mod loader;
pub mod python_runtime;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum PluginCategory {
    Extraction,
    Exporter,
    #[serde(untagged)]
    Custom(String),
}

impl std::fmt::Display for PluginCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginCategory::Extraction => write!(f, "extraction"),
            PluginCategory::Exporter => write!(f, "exporter"),
            PluginCategory::Custom(c) => write!(f, "{}", c),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PluginOrigin {
    Local,
    Bridge,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PluginDescriptor {
    pub origin: PluginOrigin,
    pub manifest: PluginManifest,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PluginManifest {
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    #[serde(alias = "downloadUrl", skip_serializing_if = "Option::is_none")]
    pub download_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    pub id: String,
    pub category: PluginCategory,
    pub exports: Vec<PluginExport>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PluginExport {
    pub id: String,
    pub label: String,
    pub description: String,
    pub entrypoint: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Vec<PluginParameter>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PluginParameter {
    pub name: String,
    #[serde(alias = "paramType", alias = "type", rename = "param_type")]
    pub param_type: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<serde_json::Value>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IpcPayload {
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Vec<PluginParameter>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ActionExecutionPayload {
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
}

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
    }
}
