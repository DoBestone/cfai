use chrono::{Duration, Utc};
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
#[derive(Debug, Serialize, Default, Clone)]
pub struct AnalyticsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub until: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continuous: Option<bool>,
}

impl AnalyticsParams {
    /// 创建最近 24 小时的参数
    pub fn last_24h() -> Self {
        let now = Utc::now();
        let yesterday = now - Duration::hours(24);
        Self {
            since: Some(yesterday.format("%Y-%m-%dT%H:%M:%SZ").to_string()),
            until: Some(now.format("%Y-%m-%dT%H:%M:%SZ").to_string()),
            continuous: Some(true),
        }
    }

    /// 创建最近 7 天的参数
    pub fn last_7d() -> Self {
        let now = Utc::now();
        let week_ago = now - Duration::days(7);
        Self {
            since: Some(week_ago.format("%Y-%m-%d").to_string()),
            until: Some(now.format("%Y-%m-%d").to_string()),
            continuous: Some(true),
        }
    }

    /// 获取时间范围 (用于 GraphQL 查询)
    pub fn get_time_range(&self) -> (String, String) {
        let now = Utc::now();

        let since = self.since.clone().unwrap_or_else(|| {
            (now - Duration::hours(24)).format("%Y-%m-%dT%H:%M:%SZ").to_string()
        });

        let until = self.until.clone().unwrap_or_else(|| {
            now.format("%Y-%m-%dT%H:%M:%SZ").to_string()
        });

        // 如果是相对时间 (如 -1440 分钟)，转换为 ISO8601
        let since = if since.starts_with('-') {
            if let Ok(minutes) = since.parse::<i64>() {
                (now + Duration::minutes(minutes)).format("%Y-%m-%dT%H:%M:%SZ").to_string()
            } else {
                since
            }
        } else {
            since
        };

        let until = if until == "0" {
            now.format("%Y-%m-%dT%H:%M:%SZ").to_string()
        } else if until.starts_with('-') {
            if let Ok(minutes) = until.parse::<i64>() {
                (now + Duration::minutes(minutes)).format("%Y-%m-%dT%H:%M:%SZ").to_string()
            } else {
                until
            }
        } else {
            until
        };

        (since, until)
    }
}
