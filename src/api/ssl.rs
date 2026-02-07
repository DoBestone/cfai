use anyhow::{Context, Result};

use crate::api::client::CfClient;
use crate::models::common::CfResponse;
use crate::models::ssl::*;

impl CfClient {
    // ==================== SSL/TLS 管理 ====================

    /// 获取 SSL/TLS 模式
    pub async fn get_ssl_mode(&self, zone_id: &str) -> Result<String> {
        let resp: CfResponse<serde_json::Value> = self
            .get(&format!("/zones/{}/settings/ssl", zone_id))
            .await?;
        let result = resp.result.context("获取 SSL 模式失败")?;
        result["value"]
            .as_str()
            .map(|s| s.to_string())
            .context("解析 SSL 模式失败")
    }

    /// 设置 SSL/TLS 模式
    pub async fn set_ssl_mode(&self, zone_id: &str, mode: &str) -> Result<serde_json::Value> {
        let body = serde_json::json!({ "value": mode });
        let resp: CfResponse<serde_json::Value> = self
            .patch(&format!("/zones/{}/settings/ssl", zone_id), &body)
            .await?;
        resp.result.context("设置 SSL 模式失败")
    }

    /// 获取 SSL 验证状态
    pub async fn get_ssl_verification(&self, zone_id: &str) -> Result<Vec<SslVerification>> {
        let resp: CfResponse<Vec<SslVerification>> = self
            .get(&format!("/zones/{}/ssl/verification", zone_id))
            .await?;
        resp.result.context("获取 SSL 验证状态失败")
    }

    /// 获取 SSL 证书包
    pub async fn list_ssl_certificates(&self, zone_id: &str) -> Result<Vec<SslCertificate>> {
        let resp: CfResponse<Vec<SslCertificate>> = self
            .get(&format!("/zones/{}/ssl/certificate_packs", zone_id))
            .await?;
        resp.result.context("获取 SSL 证书失败")
    }

    /// 设置 Always Use HTTPS
    pub async fn set_always_https(&self, zone_id: &str, enable: bool) -> Result<serde_json::Value> {
        let value = if enable { "on" } else { "off" };
        let body = serde_json::json!({ "value": value });
        let resp: CfResponse<serde_json::Value> = self
            .patch(
                &format!("/zones/{}/settings/always_use_https", zone_id),
                &body,
            )
            .await?;
        resp.result.context("设置 Always Use HTTPS 失败")
    }

    /// 获取 Always Use HTTPS 状态
    pub async fn get_always_https(&self, zone_id: &str) -> Result<bool> {
        let resp: CfResponse<serde_json::Value> = self
            .get(&format!("/zones/{}/settings/always_use_https", zone_id))
            .await?;
        let result = resp.result.context("获取 Always Use HTTPS 状态失败")?;
        Ok(result["value"].as_str() == Some("on"))
    }

    /// 设置最小 TLS 版本 (通过 SSL 模块)
    pub async fn set_ssl_min_tls(&self, zone_id: &str, version: &str) -> Result<serde_json::Value> {
        let body = serde_json::json!({ "value": version });
        let resp: CfResponse<serde_json::Value> = self
            .patch(
                &format!("/zones/{}/settings/min_tls_version", zone_id),
                &body,
            )
            .await?;
        resp.result.context("设置最小 TLS 版本失败")
    }

    /// 获取源服务器证书列表
    pub async fn list_origin_certificates(&self, zone_id: &str) -> Result<Vec<OriginCertificate>> {
        let resp: CfResponse<Vec<OriginCertificate>> = self
            .get_with_params(
                "/certificates",
                &serde_json::json!({ "zone_id": zone_id }),
            )
            .await?;
        resp.result.context("获取源服务器证书失败")
    }

    /// 创建源服务器证书
    pub async fn create_origin_certificate(
        &self,
        request: &OriginCertificateRequest,
    ) -> Result<OriginCertificate> {
        let resp: CfResponse<OriginCertificate> = self.post("/certificates", request).await?;
        resp.result.context("创建源服务器证书失败")
    }

    /// 吊销源服务器证书
    pub async fn revoke_origin_certificate(&self, cert_id: &str) -> Result<()> {
        let _resp: CfResponse<serde_json::Value> =
            self.delete(&format!("/certificates/{}", cert_id)).await?;
        Ok(())
    }

    /// 设置 Opportunistic Encryption
    pub async fn set_opportunistic_encryption(
        &self,
        zone_id: &str,
        enable: bool,
    ) -> Result<serde_json::Value> {
        let value = if enable { "on" } else { "off" };
        let body = serde_json::json!({ "value": value });
        let resp: CfResponse<serde_json::Value> = self
            .patch(
                &format!("/zones/{}/settings/opportunistic_encryption", zone_id),
                &body,
            )
            .await?;
        resp.result.context("设置 Opportunistic Encryption 失败")
    }

    /// 设置 Automatic HTTPS Rewrites
    pub async fn set_automatic_https_rewrites(
        &self,
        zone_id: &str,
        enable: bool,
    ) -> Result<serde_json::Value> {
        let value = if enable { "on" } else { "off" };
        let body = serde_json::json!({ "value": value });
        let resp: CfResponse<serde_json::Value> = self
            .patch(
                &format!("/zones/{}/settings/automatic_https_rewrites", zone_id),
                &body,
            )
            .await?;
        resp.result
            .context("设置 Automatic HTTPS Rewrites 失败")
    }
}
