use serde::{Deserialize, Serialize};

/// 分析数据总览
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AnalyticsDashboard {
    pub totals: Option<AnalyticsTotals>,
    pub timeseries: Option<Vec<AnalyticsTimeseries>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AnalyticsTotals {
    pub requests: Option<AnalyticsRequests>,
    pub bandwidth: Option<AnalyticsBandwidth>,
    pub threats: Option<AnalyticsThreats>,
    pub pageviews: Option<AnalyticsPageviews>,
    pub uniques: Option<AnalyticsUniques>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AnalyticsRequests {
    pub all: Option<u64>,
    pub cached: Option<u64>,
    pub uncached: Option<u64>,
    pub ssl: Option<AnalyticsSslRequests>,
    pub http_status: Option<serde_json::Value>,
    pub content_type: Option<serde_json::Value>,
    pub country: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AnalyticsSslRequests {
    pub encrypted: Option<u64>,
    pub unencrypted: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AnalyticsBandwidth {
    pub all: Option<u64>,
    pub cached: Option<u64>,
    pub uncached: Option<u64>,
    pub ssl: Option<AnalyticsSslBandwidth>,
    pub content_type: Option<serde_json::Value>,
    pub country: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AnalyticsSslBandwidth {
    pub encrypted: Option<u64>,
    pub unencrypted: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AnalyticsThreats {
    pub all: Option<u64>,
    pub country: Option<serde_json::Value>,
    #[serde(rename = "type")]
    pub threat_type: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AnalyticsPageviews {
    pub all: Option<u64>,
    pub search_engines: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AnalyticsUniques {
    pub all: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AnalyticsTimeseries {
    pub since: Option<String>,
    pub until: Option<String>,
    pub requests: Option<AnalyticsRequests>,
    pub bandwidth: Option<AnalyticsBandwidth>,
    pub threats: Option<AnalyticsThreats>,
    pub pageviews: Option<AnalyticsPageviews>,
    pub uniques: Option<AnalyticsUniques>,
}

/// 分析查询参数
#[derive(Debug, Serialize, Default)]
pub struct AnalyticsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub until: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continuous: Option<bool>,
}
