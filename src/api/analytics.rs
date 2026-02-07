use anyhow::{Context, Result};

use crate::api::client::CfClient;
use crate::models::analytics::*;
use crate::models::common::CfResponse;

impl CfClient {
    // ==================== 分析数据 ====================

    /// 获取域名分析数据
    pub async fn get_analytics(
        &self,
        zone_id: &str,
        params: &AnalyticsParams,
    ) -> Result<AnalyticsDashboard> {
        let resp: CfResponse<AnalyticsDashboard> = self
            .get_with_params(
                &format!("/zones/{}/analytics/dashboard", zone_id),
                params,
            )
            .await?;
        resp.result.context("获取分析数据失败")
    }

    /// 获取最近 24 小时的分析数据
    pub async fn get_analytics_24h(&self, zone_id: &str) -> Result<AnalyticsDashboard> {
        let params = AnalyticsParams {
            since: Some("-1440".to_string()), // 24 hours in minutes
            until: Some("0".to_string()),
            continuous: Some(true),
        };
        self.get_analytics(zone_id, &params).await
    }

    /// 获取 DNS 分析数据
    pub async fn get_dns_analytics(
        &self,
        zone_id: &str,
        params: &AnalyticsParams,
    ) -> Result<serde_json::Value> {
        let resp: CfResponse<serde_json::Value> = self
            .get_with_params(
                &format!("/zones/{}/dns_analytics/report", zone_id),
                params,
            )
            .await?;
        resp.result.context("获取 DNS 分析数据失败")
    }
}
