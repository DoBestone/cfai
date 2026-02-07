use anyhow::{Context, Result};

use crate::api::client::CfClient;
use crate::models::common::CfResponse;
use crate::models::zone::*;

impl CfClient {
    // ==================== Zone (域名) 管理 ====================

    /// 列出所有域名
    pub async fn list_zones(&self, params: &ZoneListParams) -> Result<CfResponse<Vec<Zone>>> {
        self.get_with_params("/zones", params).await
    }

    /// 获取域名详情
    pub async fn get_zone(&self, zone_id: &str) -> Result<Zone> {
        let resp: CfResponse<Zone> = self.get(&format!("/zones/{}", zone_id)).await?;
        resp.result.context("获取域名详情失败")
    }

    /// 通过域名名称查找 Zone ID
    pub async fn find_zone_id(&self, domain: &str) -> Result<String> {
        let params = ZoneListParams {
            name: Some(domain.to_string()),
            ..Default::default()
        };
        let resp = self.list_zones(&params).await?;
        let zones = resp.result.context("查询域名失败")?;
        zones
            .first()
            .map(|z| z.id.clone())
            .context(format!("未找到域名: {}", domain))
    }

    /// 创建域名
    pub async fn create_zone(&self, request: &CreateZoneRequest) -> Result<Zone> {
        let resp: CfResponse<Zone> = self.post("/zones", request).await?;
        resp.result.context("创建域名失败")
    }

    /// 删除域名
    pub async fn delete_zone(&self, zone_id: &str) -> Result<()> {
        let _resp: CfResponse<serde_json::Value> =
            self.delete(&format!("/zones/{}", zone_id)).await?;
        Ok(())
    }

    /// 暂停/恢复域名
    pub async fn toggle_zone_pause(&self, zone_id: &str, paused: bool) -> Result<Zone> {
        let body = serde_json::json!({ "paused": paused });
        let resp: CfResponse<Zone> = self.patch(&format!("/zones/{}", zone_id), &body).await?;
        resp.result.context("更新域名状态失败")
    }

    /// 激活域名检查
    pub async fn check_zone_activation(&self, zone_id: &str) -> Result<serde_json::Value> {
        let resp: CfResponse<serde_json::Value> = self
            .put(&format!("/zones/{}/activation_check", zone_id), &serde_json::json!({}))
            .await?;
        resp.result.context("检查域名激活状态失败")
    }

    // ==================== Zone 设置 ====================

    /// 获取所有 Zone 设置
    pub async fn get_zone_settings(&self, zone_id: &str) -> Result<Vec<ZoneSetting>> {
        let resp: CfResponse<Vec<ZoneSetting>> = self
            .get(&format!("/zones/{}/settings", zone_id))
            .await?;
        resp.result.context("获取域名设置失败")
    }

    /// 获取单个 Zone 设置
    pub async fn get_zone_setting(&self, zone_id: &str, setting_id: &str) -> Result<ZoneSetting> {
        let resp: CfResponse<ZoneSetting> = self
            .get(&format!("/zones/{}/settings/{}", zone_id, setting_id))
            .await?;
        resp.result.context("获取域名设置失败")
    }

    /// 修改 Zone 设置
    pub async fn update_zone_setting(
        &self,
        zone_id: &str,
        setting_id: &str,
        value: serde_json::Value,
    ) -> Result<ZoneSetting> {
        let body = ZoneSettingPatch { value };
        let resp: CfResponse<ZoneSetting> = self
            .patch(&format!("/zones/{}/settings/{}", zone_id, setting_id), &body)
            .await?;
        resp.result.context("更新域名设置失败")
    }

    /// 开启/关闭开发模式
    pub async fn toggle_dev_mode(&self, zone_id: &str, enable: bool) -> Result<ZoneSetting> {
        let value = if enable {
            serde_json::json!("on")
        } else {
            serde_json::json!("off")
        };
        self.update_zone_setting(zone_id, "development_mode", value)
            .await
    }

    /// 设置安全级别
    pub async fn set_security_level(&self, zone_id: &str, level: &str) -> Result<ZoneSetting> {
        self.update_zone_setting(zone_id, "security_level", serde_json::json!(level))
            .await
    }

    /// 设置最小 TLS 版本
    pub async fn set_min_tls_version(&self, zone_id: &str, version: &str) -> Result<ZoneSetting> {
        self.update_zone_setting(zone_id, "min_tls_version", serde_json::json!(version))
            .await
    }
}
