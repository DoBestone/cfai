use serde::{Deserialize, Serialize};

/// DNS 记录类型
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum DnsRecordType {
    A,
    AAAA,
    CNAME,
    TXT,
    MX,
    NS,
    SRV,
    CAA,
    LOC,
    SPF,
    CERT,
    DNSKEY,
    DS,
    NAPTR,
    SMIMEA,
    SSHFP,
    TLSA,
    URI,
}

impl std::fmt::Display for DnsRecordType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::str::FromStr for DnsRecordType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "A" => Ok(Self::A),
            "AAAA" => Ok(Self::AAAA),
            "CNAME" => Ok(Self::CNAME),
            "TXT" => Ok(Self::TXT),
            "MX" => Ok(Self::MX),
            "NS" => Ok(Self::NS),
            "SRV" => Ok(Self::SRV),
            "CAA" => Ok(Self::CAA),
            "LOC" => Ok(Self::LOC),
            "SPF" => Ok(Self::SPF),
            "CERT" => Ok(Self::CERT),
            "DNSKEY" => Ok(Self::DNSKEY),
            "DS" => Ok(Self::DS),
            "NAPTR" => Ok(Self::NAPTR),
            "SMIMEA" => Ok(Self::SMIMEA),
            "SSHFP" => Ok(Self::SSHFP),
            "TLSA" => Ok(Self::TLSA),
            "URI" => Ok(Self::URI),
            _ => Err(format!("未知的 DNS 记录类型: {}", s)),
        }
    }
}

/// DNS 记录
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DnsRecord {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub record_type: String,
    pub name: String,
    pub content: String,
    pub proxied: Option<bool>,
    pub proxiable: Option<bool>,
    pub ttl: Option<u32>,
    pub priority: Option<u16>,
    pub locked: Option<bool>,
    pub zone_id: Option<String>,
    pub zone_name: Option<String>,
    pub created_on: Option<String>,
    pub modified_on: Option<String>,
    pub comment: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// 创建/更新 DNS 记录请求
#[derive(Debug, Serialize)]
pub struct DnsRecordRequest {
    #[serde(rename = "type")]
    pub record_type: String,
    pub name: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxied: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

/// DNS 记录列表过滤
#[derive(Debug, Serialize, Default)]
pub struct DnsListParams {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub record_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxied: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_page: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
}

/// DNS 记录导入/导出格式
#[derive(Debug, Serialize, Deserialize)]
pub struct DnsImportResult {
    pub recs_added: Option<u32>,
    pub total_records_parsed: Option<u32>,
}
