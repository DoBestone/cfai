use anyhow::Result;
use clap::{Args, Subcommand};
use colored::Colorize;

use crate::api::client::CfClient;
use crate::cli::output;
use crate::cli::commands::zone::resolve_zone_id;
use crate::models::dns::*;

#[derive(Args, Debug)]
pub struct DnsArgs {
    #[command(subcommand)]
    pub command: DnsCommands,
}

#[derive(Subcommand, Debug)]
pub enum DnsCommands {
    /// 列出 DNS 记录
    #[command(alias = "ls")]
    List {
        /// 域名或 Zone ID
        domain: String,
        /// 按类型过滤 (A/AAAA/CNAME/TXT/MX 等)
        #[arg(short = 't', long)]
        record_type: Option<String>,
        /// 按名称过滤
        #[arg(short, long)]
        name: Option<String>,
        /// 每页数量
        #[arg(long, default_value = "100")]
        per_page: u32,
    },

    /// 查看 DNS 记录详情
    Get {
        /// 域名或 Zone ID
        domain: String,
        /// 记录 ID
        record_id: String,
    },

    /// 添加 DNS 记录
    Add {
        /// 域名或 Zone ID
        domain: String,
        /// 记录类型 (A/AAAA/CNAME/TXT/MX 等)
        #[arg(short = 't', long)]
        record_type: String,
        /// 记录名称 (如 www, @, sub)
        #[arg(short, long)]
        name: String,
        /// 记录值
        #[arg(short, long)]
        content: String,
        /// TTL (秒, 1=自动)
        #[arg(long, default_value = "1")]
        ttl: u32,
        /// 是否开启 Cloudflare 代理
        #[arg(short, long)]
        proxied: Option<bool>,
        /// MX 优先级
        #[arg(long)]
        priority: Option<u16>,
        /// 备注
        #[arg(long)]
        comment: Option<String>,
    },

    /// 更新 DNS 记录
    Update {
        /// 域名或 Zone ID
        domain: String,
        /// 记录 ID
        record_id: String,
        /// 记录类型
        #[arg(short = 't', long)]
        record_type: Option<String>,
        /// 记录名称
        #[arg(short, long)]
        name: Option<String>,
        /// 记录值
        #[arg(short, long)]
        content: Option<String>,
        /// TTL
        #[arg(long)]
        ttl: Option<u32>,
        /// 是否开启代理
        #[arg(short, long)]
        proxied: Option<bool>,
        /// 备注
        #[arg(long)]
        comment: Option<String>,
    },

    /// 删除 DNS 记录
    #[command(alias = "rm")]
    Delete {
        /// 域名或 Zone ID
        domain: String,
        /// 记录 ID
        record_id: String,
        /// 跳过确认
        #[arg(short = 'y', long)]
        yes: bool,
    },

    /// 快速添加 A 记录
    #[command(name = "add-a")]
    AddA {
        /// 域名或 Zone ID
        domain: String,
        /// 子域名 (如 www, @, sub)
        name: String,
        /// IP 地址
        ip: String,
        /// 开启代理
        #[arg(short, long, default_value = "true")]
        proxied: bool,
    },

    /// 快速添加 CNAME 记录
    #[command(name = "add-cname")]
    AddCname {
        /// 域名或 Zone ID
        domain: String,
        /// 子域名
        name: String,
        /// 目标域名
        target: String,
        /// 开启代理
        #[arg(short, long, default_value = "true")]
        proxied: bool,
    },

    /// 导出 DNS 记录
    Export {
        /// 域名或 Zone ID
        domain: String,
    },

    /// 查找 DNS 记录
    Find {
        /// 域名或 Zone ID
        domain: String,
        /// 搜索名称
        name: String,
        /// 记录类型
        #[arg(short = 't', long)]
        record_type: Option<String>,
    },
}

impl DnsArgs {
    pub async fn execute(&self, client: &CfClient, format: &str) -> Result<()> {
        match &self.command {
            DnsCommands::List {
                domain,
                record_type,
                name,
                per_page,
            } => {
                let zone_id = resolve_zone_id(client, domain).await?;
                let params = DnsListParams {
                    record_type: record_type.clone(),
                    name: name.clone(),
                    per_page: Some(*per_page),
                    ..Default::default()
                };
                let resp = client.list_dns_records(&zone_id, &params).await?;
                let records = resp.result.unwrap_or_default();

                if format == "json" {
                    output::print_json(&records);
                    return Ok(());
                }

                output::title(&format!("DNS 记录 - {} (共 {} 条)", domain, records.len()));

                if records.is_empty() {
                    output::warn("没有找到 DNS 记录");
                    return Ok(());
                }

                let mut table = output::create_table(vec![
                    "类型", "名称", "内容", "代理", "TTL", "ID",
                ]);

                for record in &records {
                    let proxied = record
                        .proxied
                        .map(|p| if p { "🟠 是".to_string() } else { "⚫ 否".to_string() })
                        .unwrap_or("-".to_string());
                    let ttl = record
                        .ttl
                        .map(|t| {
                            if t == 1 {
                                "自动".to_string()
                            } else {
                                format!("{}s", t)
                            }
                        })
                        .unwrap_or("-".to_string());

                    // 截断过长的内容
                    let content = if record.content.len() > 40 {
                        format!("{}...", &record.content[..37])
                    } else {
                        record.content.clone()
                    };

                    table.add_row(vec![
                        &record.record_type,
                        &record.name,
                        &content,
                        &proxied,
                        &ttl,
                        record.id.as_deref().unwrap_or("-"),
                    ]);
                }
                println!("{table}");
            }

            DnsCommands::Get { domain, record_id } => {
                let zone_id = resolve_zone_id(client, domain).await?;
                let record = client.get_dns_record(&zone_id, record_id).await?;

                if format == "json" {
                    output::print_json(&record);
                    return Ok(());
                }

                output::title(&format!("DNS 记录详情: {}", record.name));
                output::kv("ID", record.id.as_deref().unwrap_or("-"));
                output::kv("类型", &record.record_type);
                output::kv("名称", &record.name);
                output::kv("内容", &record.content);
                output::kv(
                    "代理",
                    &record
                        .proxied
                        .map(|p| p.to_string())
                        .unwrap_or("-".into()),
                );
                output::kv(
                    "TTL",
                    &record.ttl.map(|t| t.to_string()).unwrap_or("-".into()),
                );
                if let Some(p) = record.priority {
                    output::kv("优先级", &p.to_string());
                }
                output::kv("备注", record.comment.as_deref().unwrap_or("-"));
                output::kv("创建时间", record.created_on.as_deref().unwrap_or("-"));
                output::kv("修改时间", record.modified_on.as_deref().unwrap_or("-"));
            }

            DnsCommands::Add {
                domain,
                record_type,
                name,
                content,
                ttl,
                proxied,
                priority,
                comment,
            } => {
                let zone_id = resolve_zone_id(client, domain).await?;
                let request = DnsRecordRequest {
                    record_type: record_type.to_uppercase(),
                    name: name.clone(),
                    content: content.clone(),
                    ttl: Some(*ttl),
                    proxied: *proxied,
                    priority: *priority,
                    comment: comment.clone(),
                    tags: None,
                };

                let record = client.create_dns_record(&zone_id, &request).await?;
                output::success(&format!(
                    "DNS 记录创建成功: {} {} → {}",
                    record.record_type,
                    record.name,
                    record.content
                ));
            }

            DnsCommands::Update {
                domain,
                record_id,
                record_type,
                name,
                content,
                ttl,
                proxied,
                comment,
            } => {
                let zone_id = resolve_zone_id(client, domain).await?;

                // 先获取现有记录
                let existing = client.get_dns_record(&zone_id, record_id).await?;

                let mut patch = serde_json::Map::new();
                if let Some(t) = record_type {
                    patch.insert("type".to_string(), serde_json::json!(t.to_uppercase()));
                }
                if let Some(n) = name {
                    patch.insert("name".to_string(), serde_json::json!(n));
                }
                if let Some(c) = content {
                    patch.insert("content".to_string(), serde_json::json!(c));
                }
                if let Some(t) = ttl {
                    patch.insert("ttl".to_string(), serde_json::json!(t));
                }
                if let Some(p) = proxied {
                    patch.insert("proxied".to_string(), serde_json::json!(p));
                }
                if let Some(c) = comment {
                    patch.insert("comment".to_string(), serde_json::json!(c));
                }

                let patch_value = serde_json::Value::Object(patch);
                let record = client
                    .patch_dns_record(&zone_id, record_id, &patch_value)
                    .await?;
                output::success(&format!(
                    "DNS 记录已更新: {} {} → {}",
                    record.record_type, record.name, record.content
                ));
                let _ = existing; // suppress unused warning
            }

            DnsCommands::Delete {
                domain,
                record_id,
                yes,
            } => {
                let zone_id = resolve_zone_id(client, domain).await?;

                if !yes {
                    let record = client.get_dns_record(&zone_id, record_id).await?;
                    let confirm = dialoguer::Confirm::new()
                        .with_prompt(format!(
                            "确定要删除 DNS 记录 {} {} → {} 吗？",
                            record.record_type.red(),
                            record.name,
                            record.content
                        ))
                        .default(false)
                        .interact()?;
                    if !confirm {
                        output::info("已取消");
                        return Ok(());
                    }
                }

                client.delete_dns_record(&zone_id, record_id).await?;
                output::success("DNS 记录已删除");
            }

            DnsCommands::AddA {
                domain,
                name,
                ip,
                proxied,
            } => {
                let zone_id = resolve_zone_id(client, domain).await?;
                let request = DnsRecordRequest {
                    record_type: "A".to_string(),
                    name: name.clone(),
                    content: ip.clone(),
                    ttl: Some(1),
                    proxied: Some(*proxied),
                    priority: None,
                    comment: None,
                    tags: None,
                };
                let record = client.create_dns_record(&zone_id, &request).await?;
                output::success(&format!("A 记录创建成功: {} → {}", record.name, record.content));
            }

            DnsCommands::AddCname {
                domain,
                name,
                target,
                proxied,
            } => {
                let zone_id = resolve_zone_id(client, domain).await?;
                let request = DnsRecordRequest {
                    record_type: "CNAME".to_string(),
                    name: name.clone(),
                    content: target.clone(),
                    ttl: Some(1),
                    proxied: Some(*proxied),
                    priority: None,
                    comment: None,
                    tags: None,
                };
                let record = client.create_dns_record(&zone_id, &request).await?;
                output::success(&format!(
                    "CNAME 记录创建成功: {} → {}",
                    record.name, record.content
                ));
            }

            DnsCommands::Export { domain } => {
                let zone_id = resolve_zone_id(client, domain).await?;
                let export = client.export_dns_records(&zone_id).await?;
                println!("{}", export);
            }

            DnsCommands::Find {
                domain,
                name,
                record_type,
            } => {
                let zone_id = resolve_zone_id(client, domain).await?;
                let records = client
                    .find_dns_record(&zone_id, name, record_type.as_deref())
                    .await?;

                if format == "json" {
                    output::print_json(&records);
                    return Ok(());
                }

                output::title(&format!("搜索结果: {} (共 {} 条)", name, records.len()));
                for record in &records {
                    println!(
                        "  {} {} → {} {}",
                        record.record_type.cyan(),
                        record.name,
                        record.content,
                        record
                            .proxied
                            .map(|p| if p { "🟠" } else { "⚫" })
                            .unwrap_or("")
                    );
                }
            }
        }

        Ok(())
    }
}
