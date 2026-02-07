use anyhow::{Context, Result};

use crate::api::client::CfClient;
use crate::models::cache::*;
use crate::models::common::CfResponse;

impl CfClient {
    // ==================== 缓存管理 ====================

    /// 清除全部缓存
    pub async fn purge_all_cache(&self, zone_id: &str) -> Result<serde_json::Value> {
        let body = PurgeCacheRequest {
            purge_everything: Some(true),
            files: None,
            tags: None,
            hosts: None,
            prefixes: None,
        };
        let resp: CfResponse<serde_json::Value> = self
            .post(&format!("/zones/{}/purge_cache", zone_id), &body)
            .await?;
        resp.result.context("清除全部缓存失败")
    }

    /// 按 URL 清除缓存
    pub async fn purge_cache_by_urls(
        &self,
        zone_id: &str,
        urls: Vec<String>,
    ) -> Result<serde_json::Value> {
        let body = PurgeCacheRequest {
            purge_everything: None,
            files: Some(urls),
            tags: None,
            hosts: None,
            prefixes: None,
        };
        let resp: CfResponse<serde_json::Value> = self
            .post(&format!("/zones/{}/purge_cache", zone_id), &body)
            .await?;
        resp.result.context("按 URL 清除缓存失败")
    }

    /// 按 Tag 清除缓存
    pub async fn purge_cache_by_tags(
        &self,
        zone_id: &str,
        tags: Vec<String>,
    ) -> Result<serde_json::Value> {
        let body = PurgeCacheRequest {
            purge_everything: None,
            files: None,
            tags: Some(tags),
            hosts: None,
            prefixes: None,
        };
        let resp: CfResponse<serde_json::Value> = self
            .post(&format!("/zones/{}/purge_cache", zone_id), &body)
            .await?;
        resp.result.context("按 Tag 清除缓存失败")
    }

    /// 按主机名清除缓存
    pub async fn purge_cache_by_hosts(
        &self,
        zone_id: &str,
        hosts: Vec<String>,
    ) -> Result<serde_json::Value> {
        let body = PurgeCacheRequest {
            purge_everything: None,
            files: None,
            tags: None,
            hosts: Some(hosts),
            prefixes: None,
        };
        let resp: CfResponse<serde_json::Value> = self
            .post(&format!("/zones/{}/purge_cache", zone_id), &body)
            .await?;
        resp.result.context("按主机名清除缓存失败")
    }

    /// 获取缓存级别
    pub async fn get_cache_level(&self, zone_id: &str) -> Result<String> {
        let resp: CfResponse<serde_json::Value> = self
            .get(&format!("/zones/{}/settings/cache_level", zone_id))
            .await?;
        let result = resp.result.context("获取缓存级别失败")?;
        result["value"]
            .as_str()
            .map(|s| s.to_string())
            .context("解析缓存级别失败")
    }

    /// 设置缓存级别
    pub async fn set_cache_level(
        &self,
        zone_id: &str,
        level: &str,
    ) -> Result<serde_json::Value> {
        let body = serde_json::json!({ "value": level });
        let resp: CfResponse<serde_json::Value> = self
            .patch(
                &format!("/zones/{}/settings/cache_level", zone_id),
                &body,
            )
            .await?;
        resp.result.context("设置缓存级别失败")
    }

    /// 获取浏览器缓存 TTL
    pub async fn get_browser_cache_ttl(&self, zone_id: &str) -> Result<u32> {
        let resp: CfResponse<serde_json::Value> = self
            .get(&format!(
                "/zones/{}/settings/browser_cache_ttl",
                zone_id
            ))
            .await?;
        let result = resp.result.context("获取浏览器缓存 TTL 失败")?;
        result["value"]
            .as_u64()
            .map(|v| v as u32)
            .context("解析浏览器缓存 TTL 失败")
    }

    /// 设置浏览器缓存 TTL
    pub async fn set_browser_cache_ttl(
        &self,
        zone_id: &str,
        ttl: u32,
    ) -> Result<serde_json::Value> {
        let body = serde_json::json!({ "value": ttl });
        let resp: CfResponse<serde_json::Value> = self
            .patch(
                &format!("/zones/{}/settings/browser_cache_ttl", zone_id),
                &body,
            )
            .await?;
        resp.result.context("设置浏览器缓存 TTL 失败")
    }

    /// 开启/关闭开发模式 (通过缓存模块)
    pub async fn set_development_mode(
        &self,
        zone_id: &str,
        enable: bool,
    ) -> Result<serde_json::Value> {
        let value = if enable { "on" } else { "off" };
        let body = serde_json::json!({ "value": value });
        let resp: CfResponse<serde_json::Value> = self
            .patch(
                &format!("/zones/{}/settings/development_mode", zone_id),
                &body,
            )
            .await?;
        resp.result.context("设置开发模式失败")
    }
}
