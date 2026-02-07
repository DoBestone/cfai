use anyhow::{Context, Result};

use crate::api::client::CfClient;
use crate::models::common::CfResponse;
use crate::models::page_rules::*;

impl CfClient {
    // ==================== 页面规则管理 ====================

    /// 列出页面规则
    pub async fn list_page_rules(&self, zone_id: &str) -> Result<Vec<PageRule>> {
        let resp: CfResponse<Vec<PageRule>> = self
            .get(&format!("/zones/{}/pagerules", zone_id))
            .await?;
        resp.result.context("获取页面规则失败")
    }

    /// 获取页面规则详情
    pub async fn get_page_rule(&self, zone_id: &str, rule_id: &str) -> Result<PageRule> {
        let resp: CfResponse<PageRule> = self
            .get(&format!("/zones/{}/pagerules/{}", zone_id, rule_id))
            .await?;
        resp.result.context("获取页面规则详情失败")
    }

    /// 创建页面规则
    pub async fn create_page_rule(
        &self,
        zone_id: &str,
        request: &CreatePageRuleRequest,
    ) -> Result<PageRule> {
        let resp: CfResponse<PageRule> = self
            .post(&format!("/zones/{}/pagerules", zone_id), request)
            .await?;
        resp.result.context("创建页面规则失败")
    }

    /// 更新页面规则
    pub async fn update_page_rule(
        &self,
        zone_id: &str,
        rule_id: &str,
        request: &CreatePageRuleRequest,
    ) -> Result<PageRule> {
        let resp: CfResponse<PageRule> = self
            .put(
                &format!("/zones/{}/pagerules/{}", zone_id, rule_id),
                request,
            )
            .await?;
        resp.result.context("更新页面规则失败")
    }

    /// 删除页面规则
    pub async fn delete_page_rule(&self, zone_id: &str, rule_id: &str) -> Result<()> {
        let _resp: CfResponse<serde_json::Value> = self
            .delete(&format!("/zones/{}/pagerules/{}", zone_id, rule_id))
            .await?;
        Ok(())
    }

    /// 创建 URL 跳转规则
    pub async fn create_redirect_rule(
        &self,
        zone_id: &str,
        url_pattern: &str,
        redirect_url: &str,
        status_code: u16,
    ) -> Result<PageRule> {
        let request = CreatePageRuleRequest {
            targets: vec![PageRuleTarget {
                target: Some("url".to_string()),
                constraint: Some(PageRuleConstraint {
                    operator: Some("matches".to_string()),
                    value: Some(url_pattern.to_string()),
                }),
            }],
            actions: vec![PageRuleAction {
                id: Some("forwarding_url".to_string()),
                value: Some(serde_json::json!({
                    "url": redirect_url,
                    "status_code": status_code
                })),
            }],
            priority: None,
            status: Some("active".to_string()),
        };
        self.create_page_rule(zone_id, &request).await
    }
}
