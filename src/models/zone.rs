use serde::{Deserialize, Serialize};

/// Zone (域名) 模型
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Zone {
    pub id: String,
    pub name: String,
    pub status: String,
    #[serde(rename = "type")]
    pub zone_type: Option<String>,
    pub paused: Option<bool>,
    pub development_mode: Option<i64>,
    pub name_servers: Option<Vec<String>>,
    pub original_name_servers: Option<Vec<String>>,
    pub created_on: Option<String>,
    pub modified_on: Option<String>,
    pub activated_on: Option<String>,
    pub plan: Option<ZonePlan>,
    pub account: Option<ZoneAccount>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ZonePlan {
    pub id: Option<String>,
    pub name: Option<String>,
    pub price: Option<f64>,
    pub currency: Option<String>,
    pub frequency: Option<String>,
    pub is_subscribed: Option<bool>,
    pub can_subscribe: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ZoneAccount {
    pub id: Option<String>,
    pub name: Option<String>,
}

/// 创建 Zone 的请求体
#[derive(Debug, Serialize)]
pub struct CreateZoneRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<CreateZoneAccount>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub zone_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jump_start: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct CreateZoneAccount {
    pub id: String,
}

/// 更新 Zone 设置
#[derive(Debug, Serialize)]
pub struct ZoneSettingPatch {
    pub value: serde_json::Value,
}

/// Zone 设置项
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ZoneSetting {
    pub id: String,
    pub value: serde_json::Value,
    pub editable: Option<bool>,
    pub modified_on: Option<String>,
}

/// Zone 列表过滤参数
#[derive(Debug, Serialize, Default)]
pub struct ZoneListParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_page: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,
}
