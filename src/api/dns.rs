use anyhow::{Context, Result};

use crate::api::client::CfClient;
use crate::models::common::CfResponse;
use crate::models::dns::*;

impl CfClient {
    // ==================== DNS 记录管理 ====================

    /// 列出 DNS 记录
    pub async fn list_dns_records(
        &self,
        zone_id: &str,
        params: &DnsListParams,
    ) -> Result<CfResponse<Vec<DnsRecord>>> {
        self.get_with_params(&format!("/zones/{}/dns_records", zone_id), params)
            .await
    }

    /// 获取 DNS 记录详情
    pub async fn get_dns_record(&self, zone_id: &str, record_id: &str) -> Result<DnsRecord> {
        let resp: CfResponse<DnsRecord> = self
            .get(&format!("/zones/{}/dns_records/{}", zone_id, record_id))
            .await?;
        resp.result.context("获取 DNS 记录失败")
    }

    /// 创建 DNS 记录
    pub async fn create_dns_record(
        &self,
        zone_id: &str,
        request: &DnsRecordRequest,
    ) -> Result<DnsRecord> {
        let resp: CfResponse<DnsRecord> = self
            .post(&format!("/zones/{}/dns_records", zone_id), request)
            .await?;
        resp.result.context("创建 DNS 记录失败")
    }

    /// 更新 DNS 记录 (全量)
    pub async fn update_dns_record(
        &self,
        zone_id: &str,
        record_id: &str,
        request: &DnsRecordRequest,
    ) -> Result<DnsRecord> {
        let resp: CfResponse<DnsRecord> = self
            .put(
                &format!("/zones/{}/dns_records/{}", zone_id, record_id),
                request,
            )
            .await?;
        resp.result.context("更新 DNS 记录失败")
    }

    /// 部分更新 DNS 记录
    pub async fn patch_dns_record(
        &self,
        zone_id: &str,
        record_id: &str,
        patch: &serde_json::Value,
    ) -> Result<DnsRecord> {
        let resp: CfResponse<DnsRecord> = self
            .patch(
                &format!("/zones/{}/dns_records/{}", zone_id, record_id),
                patch,
            )
            .await?;
        resp.result.context("更新 DNS 记录失败")
    }

    /// 删除 DNS 记录
    pub async fn delete_dns_record(&self, zone_id: &str, record_id: &str) -> Result<()> {
        let _resp: CfResponse<serde_json::Value> = self
            .delete(&format!("/zones/{}/dns_records/{}", zone_id, record_id))
            .await?;
        Ok(())
    }

    /// 导出 DNS 记录 (BIND 格式)
    pub async fn export_dns_records(&self, zone_id: &str) -> Result<String> {
        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records/export",
            zone_id
        );
        // 导出返回纯文本，需要特殊处理
        let resp: CfResponse<serde_json::Value> = self
            .get(&format!("/zones/{}/dns_records/export", zone_id))
            .await?;
        Ok(serde_json::to_string_pretty(&resp.result).unwrap_or_else(|_| url))
    }

    /// 根据名称和类型查找 DNS 记录
    pub async fn find_dns_record(
        &self,
        zone_id: &str,
        name: &str,
        record_type: Option<&str>,
    ) -> Result<Vec<DnsRecord>> {
        let params = DnsListParams {
            name: Some(name.to_string()),
            record_type: record_type.map(|t| t.to_string()),
            ..Default::default()
        };
        let resp = self.list_dns_records(zone_id, &params).await?;
        resp.result.context("查找 DNS 记录失败")
    }

    /// 批量创建 DNS 记录
    pub async fn batch_create_dns_records(
        &self,
        zone_id: &str,
        records: &[DnsRecordRequest],
    ) -> Result<Vec<Result<DnsRecord>>> {
        let mut results = Vec::new();
        for record in records {
            results.push(self.create_dns_record(zone_id, record).await);
        }
        Ok(results)
    }
}
