use anyhow::{Context, Result};

use crate::api::client::CfClient;
use crate::models::common::CfResponse;
use crate::models::workers::*;

impl CfClient {
    // ==================== Workers 管理 ====================

    /// 列出 Workers 脚本
    pub async fn list_workers(&self, account_id: &str) -> Result<Vec<WorkerScript>> {
        let resp: CfResponse<Vec<WorkerScript>> = self
            .get(&format!("/accounts/{}/workers/scripts", account_id))
            .await?;
        resp.result.context("获取 Workers 脚本列表失败")
    }

    /// 删除 Workers 脚本
    pub async fn delete_worker(&self, account_id: &str, script_name: &str) -> Result<()> {
        let _resp: CfResponse<serde_json::Value> = self
            .delete(&format!(
                "/accounts/{}/workers/scripts/{}",
                account_id, script_name
            ))
            .await?;
        Ok(())
    }

    /// 列出 Workers 路由
    pub async fn list_worker_routes(&self, zone_id: &str) -> Result<Vec<WorkerRoute>> {
        let resp: CfResponse<Vec<WorkerRoute>> = self
            .get(&format!("/zones/{}/workers/routes", zone_id))
            .await?;
        resp.result.context("获取 Workers 路由失败")
    }

    /// 创建 Workers 路由
    pub async fn create_worker_route(
        &self,
        zone_id: &str,
        request: &CreateWorkerRouteRequest,
    ) -> Result<WorkerRoute> {
        let resp: CfResponse<WorkerRoute> = self
            .post(&format!("/zones/{}/workers/routes", zone_id), request)
            .await?;
        resp.result.context("创建 Workers 路由失败")
    }

    /// 删除 Workers 路由
    pub async fn delete_worker_route(&self, zone_id: &str, route_id: &str) -> Result<()> {
        let _resp: CfResponse<serde_json::Value> = self
            .delete(&format!(
                "/zones/{}/workers/routes/{}",
                zone_id, route_id
            ))
            .await?;
        Ok(())
    }

    /// 列出 Workers KV 命名空间
    pub async fn list_kv_namespaces(&self, account_id: &str) -> Result<Vec<KvNamespace>> {
        let resp: CfResponse<Vec<KvNamespace>> = self
            .get(&format!(
                "/accounts/{}/storage/kv/namespaces",
                account_id
            ))
            .await?;
        resp.result.context("获取 KV 命名空间失败")
    }

    /// 列出 Workers 自定义域名
    pub async fn list_worker_domains(&self, account_id: &str) -> Result<Vec<WorkerDomain>> {
        let resp: CfResponse<Vec<WorkerDomain>> = self
            .get(&format!("/accounts/{}/workers/domains", account_id))
            .await?;
        resp.result.context("获取 Workers 域名失败")
    }
}
