use anyhow::{Context, Result};
use reqwest::{header, Client, Response};
use serde::de::DeserializeOwned;
use tracing::debug;

use crate::models::common::CfResponse;

const CF_API_BASE: &str = "https://api.cloudflare.com/client/v4";

/// Cloudflare API 客户端
#[derive(Clone)]
pub struct CfClient {
    client: Client,
    base_url: String,
}

/// 认证方式
pub enum AuthMethod {
    /// API Token (推荐)
    ApiToken(String),
    /// Email + Global API Key
    ApiKey { email: String, key: String },
}

impl CfClient {
    /// 创建新的 Cloudflare API 客户端
    pub fn new(auth: AuthMethod) -> Result<Self> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );

        match &auth {
            AuthMethod::ApiToken(token) => {
                headers.insert(
                    header::AUTHORIZATION,
                    header::HeaderValue::from_str(&format!("Bearer {}", token))
                        .context("无效的 API Token")?,
                );
            }
            AuthMethod::ApiKey { email, key } => {
                headers.insert(
                    "X-Auth-Email",
                    header::HeaderValue::from_str(email).context("无效的邮箱地址")?,
                );
                headers.insert(
                    "X-Auth-Key",
                    header::HeaderValue::from_str(key).context("无效的 API Key")?,
                );
            }
        }

        let client = Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("创建 HTTP 客户端失败")?;

        Ok(Self {
            client,
            base_url: CF_API_BASE.to_string(),
        })
    }

    /// 构建完整 URL
    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    /// GET 请求
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<CfResponse<T>> {
        let url = self.url(path);
        debug!("GET {}", url);
        let resp = self.client.get(&url).send().await.context("GET 请求失败")?;
        self.handle_response(resp).await
    }

    /// GET 请求 (带查询参数)
    pub async fn get_with_params<T: DeserializeOwned, P: serde::Serialize>(
        &self,
        path: &str,
        params: &P,
    ) -> Result<CfResponse<T>> {
        let url = self.url(path);
        debug!("GET {} (with params)", url);
        let resp = self
            .client
            .get(&url)
            .query(params)
            .send()
            .await
            .context("GET 请求失败")?;
        self.handle_response(resp).await
    }

    /// POST 请求
    pub async fn post<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<CfResponse<T>> {
        let url = self.url(path);
        debug!("POST {}", url);
        let resp = self
            .client
            .post(&url)
            .json(body)
            .send()
            .await
            .context("POST 请求失败")?;
        self.handle_response(resp).await
    }

    /// PUT 请求
    pub async fn put<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<CfResponse<T>> {
        let url = self.url(path);
        debug!("PUT {}", url);
        let resp = self
            .client
            .put(&url)
            .json(body)
            .send()
            .await
            .context("PUT 请求失败")?;
        self.handle_response(resp).await
    }

    /// PATCH 请求
    pub async fn patch<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<CfResponse<T>> {
        let url = self.url(path);
        debug!("PATCH {}", url);
        let resp = self
            .client
            .patch(&url)
            .json(body)
            .send()
            .await
            .context("PATCH 请求失败")?;
        self.handle_response(resp).await
    }

    /// DELETE 请求
    pub async fn delete<T: DeserializeOwned>(&self, path: &str) -> Result<CfResponse<T>> {
        let url = self.url(path);
        debug!("DELETE {}", url);
        let resp = self
            .client
            .delete(&url)
            .send()
            .await
            .context("DELETE 请求失败")?;
        self.handle_response(resp).await
    }

    /// DELETE 请求 (带请求体)
    pub async fn delete_with_body<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<CfResponse<T>> {
        let url = self.url(path);
        debug!("DELETE {} (with body)", url);
        let resp = self
            .client
            .delete(&url)
            .json(body)
            .send()
            .await
            .context("DELETE 请求失败")?;
        self.handle_response(resp).await
    }

    /// 处理响应
    async fn handle_response<T: DeserializeOwned>(
        &self,
        resp: Response,
    ) -> Result<CfResponse<T>> {
        let status = resp.status();
        let body = resp.text().await.context("读取响应体失败")?;

        debug!("Response status: {}, body length: {}", status, body.len());

        if !status.is_success() {
            // 尝试解析错误响应
            if let Ok(cf_resp) = serde_json::from_str::<CfResponse<serde_json::Value>>(&body) {
                let errors: Vec<String> = cf_resp.errors.iter().map(|e| e.to_string()).collect();
                anyhow::bail!(
                    "Cloudflare API 错误 (HTTP {}): {}",
                    status.as_u16(),
                    if errors.is_empty() {
                        body.clone()
                    } else {
                        errors.join("; ")
                    }
                );
            }
            anyhow::bail!("HTTP 错误 {}: {}", status.as_u16(), body);
        }

        serde_json::from_str::<CfResponse<T>>(&body)
            .with_context(|| format!("解析 Cloudflare API 响应失败: {}", &body[..body.len().min(500)]))
    }

    /// 验证 Token 有效性
    pub async fn verify_token(&self) -> Result<bool> {
        let resp: CfResponse<serde_json::Value> = self.get("/user/tokens/verify").await?;
        Ok(resp.success)
    }

    /// 获取当前用户信息
    pub async fn get_user(&self) -> Result<serde_json::Value> {
        let resp: CfResponse<serde_json::Value> = self.get("/user").await?;
        resp.result.context("获取用户信息失败")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_construction() {
        let client = CfClient {
            client: Client::new(),
            base_url: CF_API_BASE.to_string(),
        };
        assert_eq!(
            client.url("/zones"),
            "https://api.cloudflare.com/client/v4/zones"
        );
    }
}
