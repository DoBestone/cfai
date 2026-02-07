use serde::{Deserialize, Serialize};

/// Cloudflare API 通用响应包装
#[derive(Debug, Deserialize)]
pub struct CfResponse<T> {
    pub success: bool,
    pub errors: Vec<CfError>,
    pub messages: Vec<CfMessage>,
    pub result: Option<T>,
    pub result_info: Option<ResultInfo>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CfError {
    pub code: i64,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct CfMessage {
    pub code: Option<i64>,
    pub message: String,
}

/// 分页信息
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ResultInfo {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub total_pages: Option<u32>,
    pub count: Option<u32>,
    pub total_count: Option<u32>,
}

/// 通用分页参数
#[derive(Debug, Serialize, Default)]
pub struct PaginationParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_page: Option<u32>,
}

/// 排序方向
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum SortDirection {
    Asc,
    Desc,
}

impl std::fmt::Display for CfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::fmt::Display for CfResponse<serde_json::Value> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.success {
            write!(f, "Success")
        } else {
            let errors: Vec<String> = self.errors.iter().map(|e| e.to_string()).collect();
            write!(f, "Failed: {}", errors.join(", "))
        }
    }
}
