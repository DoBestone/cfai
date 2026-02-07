use serde::{Deserialize, Serialize};

/// SSL/TLS 模式
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum SslMode {
    Off,
    Flexible,
    Full,
    Strict,
}

impl std::fmt::Display for SslMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SslMode::Off => write!(f, "off"),
            SslMode::Flexible => write!(f, "flexible"),
            SslMode::Full => write!(f, "full"),
            SslMode::Strict => write!(f, "strict"),
        }
    }
}

impl std::str::FromStr for SslMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "off" => Ok(SslMode::Off),
            "flexible" => Ok(SslMode::Flexible),
            "full" => Ok(SslMode::Full),
            "strict" => Ok(SslMode::Strict),
            _ => Err(format!("未知的 SSL 模式: {}，可选: off/flexible/full/strict", s)),
        }
    }
}

/// SSL 证书信息
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SslCertificate {
    pub id: Option<String>,
    pub hosts: Option<Vec<String>>,
    pub issuer: Option<String>,
    pub signature: Option<String>,
    pub status: Option<String>,
    pub bundle_method: Option<String>,
    pub uploaded_on: Option<String>,
    pub modified_on: Option<String>,
    pub expires_on: Option<String>,
    pub priority: Option<i32>,
}

/// SSL 验证记录
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SslVerification {
    pub certificate_status: Option<String>,
    pub hostname: Option<String>,
    pub verification_type: Option<String>,
    pub verification_status: Option<String>,
    pub verification_info: Option<serde_json::Value>,
}

/// 源服务器证书请求
#[derive(Debug, Serialize)]
pub struct OriginCertificateRequest {
    pub hostnames: Vec<String>,
    pub requested_validity: Option<u32>,
    pub request_type: Option<String>,
    pub csr: Option<String>,
}

/// 源服务器证书
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OriginCertificate {
    pub id: Option<String>,
    pub certificate: Option<String>,
    pub hostnames: Option<Vec<String>>,
    pub expires_on: Option<String>,
    pub request_type: Option<String>,
    pub requested_validity: Option<u32>,
    pub private_key: Option<String>,
}

/// HTTPS 重定向设置
#[derive(Debug, Serialize)]
pub struct AlwaysUseHttps {
    pub value: String,
}

/// 最低 TLS 版本
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum MinTlsVersion {
    #[serde(rename = "1.0")]
    Tls10,
    #[serde(rename = "1.1")]
    Tls11,
    #[serde(rename = "1.2")]
    Tls12,
    #[serde(rename = "1.3")]
    Tls13,
}

impl std::fmt::Display for MinTlsVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MinTlsVersion::Tls10 => write!(f, "1.0"),
            MinTlsVersion::Tls11 => write!(f, "1.1"),
            MinTlsVersion::Tls12 => write!(f, "1.2"),
            MinTlsVersion::Tls13 => write!(f, "1.3"),
        }
    }
}
