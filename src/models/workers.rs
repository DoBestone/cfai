use serde::{Deserialize, Serialize};

/// Workers 脚本
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WorkerScript {
    pub id: Option<String>,
    pub tag: Option<String>,
    pub etag: Option<String>,
    pub handlers: Option<Vec<String>>,
    pub modified_on: Option<String>,
    pub created_on: Option<String>,
    pub usage_model: Option<String>,
    pub logpush: Option<bool>,
}

/// Workers 路由
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WorkerRoute {
    pub id: Option<String>,
    pub pattern: Option<String>,
    pub script: Option<String>,
}

/// 创建 Workers 路由请求
#[derive(Debug, Serialize)]
pub struct CreateWorkerRouteRequest {
    pub pattern: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script: Option<String>,
}

/// Workers KV 命名空间
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct KvNamespace {
    pub id: Option<String>,
    pub title: Option<String>,
    pub supports_url_encoding: Option<bool>,
}

/// Workers 域名绑定
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WorkerDomain {
    pub id: Option<String>,
    pub zone_id: Option<String>,
    pub zone_name: Option<String>,
    pub hostname: Option<String>,
    pub service: Option<String>,
    pub environment: Option<String>,
}
