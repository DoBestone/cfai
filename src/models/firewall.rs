use serde::{Deserialize, Serialize};

/// 防火墙规则
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FirewallRule {
    pub id: Option<String>,
    pub paused: Option<bool>,
    pub description: Option<String>,
    pub action: Option<String>,
    pub priority: Option<i32>,
    pub filter: Option<FirewallFilter>,
    pub created_on: Option<String>,
    pub modified_on: Option<String>,
}

/// 防火墙过滤器
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FirewallFilter {
    pub id: Option<String>,
    pub expression: Option<String>,
    pub paused: Option<bool>,
    pub description: Option<String>,
}

/// WAF 规则组
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WafRuleGroup {
    pub id: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub rules_count: Option<u32>,
    pub modified_rules_count: Option<u32>,
    pub mode: Option<String>,
}

/// IP 访问规则
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct IpAccessRule {
    pub id: Option<String>,
    pub mode: Option<String>,
    pub notes: Option<String>,
    pub configuration: Option<IpAccessConfig>,
    pub created_on: Option<String>,
    pub modified_on: Option<String>,
    pub scope: Option<IpAccessScope>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct IpAccessConfig {
    pub target: Option<String>,
    pub value: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct IpAccessScope {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub scope_type: Option<String>,
}

/// 创建 IP 访问规则请求
#[derive(Debug, Serialize)]
pub struct CreateIpAccessRuleRequest {
    pub mode: String,
    pub configuration: IpAccessRuleConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct IpAccessRuleConfig {
    pub target: String,
    pub value: String,
}

/// 用户代理规则
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserAgentRule {
    pub id: Option<String>,
    pub description: Option<String>,
    pub mode: Option<String>,
    pub paused: Option<bool>,
    pub configuration: Option<UserAgentConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserAgentConfig {
    pub target: Option<String>,
    pub value: Option<String>,
}

/// 速率限制规则
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RateLimitRule {
    pub id: Option<String>,
    pub disabled: Option<bool>,
    pub description: Option<String>,
    pub threshold: Option<u32>,
    pub period: Option<u32>,
    pub action: Option<RateLimitAction>,
    #[serde(rename = "match")]
    pub match_config: Option<RateLimitMatch>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RateLimitAction {
    pub mode: Option<String>,
    pub timeout: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RateLimitMatch {
    pub request: Option<RateLimitMatchRequest>,
    pub response: Option<RateLimitMatchResponse>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RateLimitMatchRequest {
    pub methods: Option<Vec<String>>,
    pub schemes: Option<Vec<String>>,
    pub url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RateLimitMatchResponse {
    pub status: Option<Vec<u32>>,
    pub origin_traffic: Option<bool>,
}

/// 安全级别
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum SecurityLevel {
    Off,
    EssentiallyOff,
    Low,
    Medium,
    High,
    UnderAttack,
}

impl std::fmt::Display for SecurityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecurityLevel::Off => write!(f, "off"),
            SecurityLevel::EssentiallyOff => write!(f, "essentially_off"),
            SecurityLevel::Low => write!(f, "low"),
            SecurityLevel::Medium => write!(f, "medium"),
            SecurityLevel::High => write!(f, "high"),
            SecurityLevel::UnderAttack => write!(f, "under_attack"),
        }
    }
}

impl std::str::FromStr for SecurityLevel {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "off" => Ok(SecurityLevel::Off),
            "essentially_off" => Ok(SecurityLevel::EssentiallyOff),
            "low" => Ok(SecurityLevel::Low),
            "medium" => Ok(SecurityLevel::Medium),
            "high" => Ok(SecurityLevel::High),
            "under_attack" => Ok(SecurityLevel::UnderAttack),
            _ => Err(format!("未知的安全级别: {}", s)),
        }
    }
}
