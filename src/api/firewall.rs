use anyhow::{Context, Result};

use crate::api::client::CfClient;
use crate::models::common::CfResponse;
use crate::models::firewall::*;

impl CfClient {
    // ==================== 防火墙管理 ====================

    /// 列出防火墙规则
    pub async fn list_firewall_rules(
        &self,
        zone_id: &str,
    ) -> Result<Vec<FirewallRule>> {
        let resp: CfResponse<Vec<FirewallRule>> = self
            .get(&format!("/zones/{}/firewall/rules", zone_id))
            .await?;
        resp.result.context("获取防火墙规则失败")
    }

    /// 获取安全级别
    pub async fn get_security_level(&self, zone_id: &str) -> Result<String> {
        let resp: CfResponse<serde_json::Value> = self
            .get(&format!("/zones/{}/settings/security_level", zone_id))
            .await?;
        let result = resp.result.context("获取安全级别失败")?;
        result["value"]
            .as_str()
            .map(|s| s.to_string())
            .context("解析安全级别失败")
    }

    /// 列出 IP 访问规则
    pub async fn list_ip_access_rules(
        &self,
        zone_id: &str,
    ) -> Result<Vec<IpAccessRule>> {
        let resp: CfResponse<Vec<IpAccessRule>> = self
            .get(&format!("/zones/{}/firewall/access_rules/rules", zone_id))
            .await?;
        resp.result.context("获取 IP 访问规则失败")
    }

    /// 创建 IP 访问规则 (封禁/白名单)
    pub async fn create_ip_access_rule(
        &self,
        zone_id: &str,
        request: &CreateIpAccessRuleRequest,
    ) -> Result<IpAccessRule> {
        let resp: CfResponse<IpAccessRule> = self
            .post(
                &format!("/zones/{}/firewall/access_rules/rules", zone_id),
                request,
            )
            .await?;
        resp.result.context("创建 IP 访问规则失败")
    }

    /// 删除 IP 访问规则
    pub async fn delete_ip_access_rule(&self, zone_id: &str, rule_id: &str) -> Result<()> {
        let _resp: CfResponse<serde_json::Value> = self
            .delete(&format!(
                "/zones/{}/firewall/access_rules/rules/{}",
                zone_id, rule_id
            ))
            .await?;
        Ok(())
    }

    /// 封禁 IP
    pub async fn block_ip(&self, zone_id: &str, ip: &str, note: Option<&str>) -> Result<IpAccessRule> {
        let request = CreateIpAccessRuleRequest {
            mode: "block".to_string(),
            configuration: IpAccessRuleConfig {
                target: "ip".to_string(),
                value: ip.to_string(),
            },
            notes: note.map(|n| n.to_string()),
        };
        self.create_ip_access_rule(zone_id, &request).await
    }

    /// IP 白名单
    pub async fn whitelist_ip(
        &self,
        zone_id: &str,
        ip: &str,
        note: Option<&str>,
    ) -> Result<IpAccessRule> {
        let request = CreateIpAccessRuleRequest {
            mode: "whitelist".to_string(),
            configuration: IpAccessRuleConfig {
                target: "ip".to_string(),
                value: ip.to_string(),
            },
            notes: note.map(|n| n.to_string()),
        };
        self.create_ip_access_rule(zone_id, &request).await
    }

    /// 列出速率限制规则
    pub async fn list_rate_limits(&self, zone_id: &str) -> Result<Vec<RateLimitRule>> {
        let resp: CfResponse<Vec<RateLimitRule>> = self
            .get(&format!("/zones/{}/rate_limits", zone_id))
            .await?;
        resp.result.context("获取速率限制规则失败")
    }

    /// 开启/关闭 Under Attack 模式
    pub async fn set_under_attack_mode(
        &self,
        zone_id: &str,
        enable: bool,
    ) -> Result<serde_json::Value> {
        let level = if enable { "under_attack" } else { "medium" };
        let body = serde_json::json!({ "value": level });
        let resp: CfResponse<serde_json::Value> = self
            .patch(
                &format!("/zones/{}/settings/security_level", zone_id),
                &body,
            )
            .await?;
        resp.result.context("设置 Under Attack 模式失败")
    }

    /// 设置浏览器完整性检查
    pub async fn set_browser_check(
        &self,
        zone_id: &str,
        enable: bool,
    ) -> Result<serde_json::Value> {
        let value = if enable { "on" } else { "off" };
        let body = serde_json::json!({ "value": value });
        let resp: CfResponse<serde_json::Value> = self
            .patch(
                &format!("/zones/{}/settings/browser_check", zone_id),
                &body,
            )
            .await?;
        resp.result.context("设置浏览器完整性检查失败")
    }
}
