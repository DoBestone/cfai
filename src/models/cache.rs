use serde::{Deserialize, Serialize};

/// 缓存清除请求
#[derive(Debug, Serialize)]
pub struct PurgeCacheRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purge_everything: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hosts: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefixes: Option<Vec<String>>,
}

/// 缓存级别
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum CacheLevel {
    Aggressive,
    Basic,
    Simplified,
}

impl std::fmt::Display for CacheLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheLevel::Aggressive => write!(f, "aggressive"),
            CacheLevel::Basic => write!(f, "basic"),
            CacheLevel::Simplified => write!(f, "simplified"),
        }
    }
}

/// 浏览器缓存 TTL 设置
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BrowserCacheTtl {
    pub value: u32,
}

/// 开发模式设置
#[derive(Debug, Serialize)]
pub struct DevModeRequest {
    pub value: String,
}

/// 缓存规则
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CacheRule {
    pub id: Option<String>,
    pub expression: Option<String>,
    pub description: Option<String>,
    pub action: Option<String>,
    pub action_parameters: Option<serde_json::Value>,
    pub enabled: Option<bool>,
}
