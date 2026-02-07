use serde::{Deserialize, Serialize};

/// 页面规则
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PageRule {
    pub id: Option<String>,
    pub targets: Option<Vec<PageRuleTarget>>,
    pub actions: Option<Vec<PageRuleAction>>,
    pub priority: Option<i32>,
    pub status: Option<String>,
    pub created_on: Option<String>,
    pub modified_on: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PageRuleTarget {
    pub target: Option<String>,
    pub constraint: Option<PageRuleConstraint>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PageRuleConstraint {
    pub operator: Option<String>,
    pub value: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PageRuleAction {
    pub id: Option<String>,
    pub value: Option<serde_json::Value>,
}

/// 创建页面规则请求
#[derive(Debug, Serialize)]
pub struct CreatePageRuleRequest {
    pub targets: Vec<PageRuleTarget>,
    pub actions: Vec<PageRuleAction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}
