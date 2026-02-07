use anyhow::Result;
use clap::{Args, Subcommand};
use colored::Colorize;

use crate::api::client::CfClient;
use crate::cli::output;
use crate::models::zone::*;

#[derive(Args, Debug)]
pub struct ZoneArgs {
    #[command(subcommand)]
    pub command: ZoneCommands,
}

#[derive(Subcommand, Debug)]
pub enum ZoneCommands {
    /// 列出所有域名
    #[command(alias = "ls")]
    List {
        /// 按名称过滤
        #[arg(short, long)]
        name: Option<String>,
        /// 按状态过滤 (active/pending/initializing/moved/deleted)
        #[arg(short, long)]
        status: Option<String>,
        /// 每页数量
        #[arg(long, default_value = "50")]
        per_page: u32,
    },

    /// 查看域名详情
    #[command(alias = "info")]
    Get {
        /// 域名或 Zone ID
        domain: String,
    },

    /// 添加域名
    Add {
        /// 域名
        domain: String,
        /// 账户 ID
        #[arg(long)]
        account_id: Option<String>,
        /// 是否自动导入已有 DNS 记录
        #[arg(long)]
        jump_start: Option<bool>,
    },

    /// 删除域名
    #[command(alias = "rm")]
    Delete {
        /// 域名或 Zone ID
        domain: String,
        /// 跳过确认
        #[arg(short = 'y', long)]
        yes: bool,
    },

    /// 暂停域名
    Pause {
        /// 域名或 Zone ID
        domain: String,
    },

    /// 恢复域名
    Resume {
        /// 域名或 Zone ID
        domain: String,
    },

    /// 检查域名激活状态
    Check {
        /// 域名或 Zone ID
        domain: String,
    },

    /// 查看域名设置
    Settings {
        /// 域名或 Zone ID
        domain: String,
        /// 设置项 ID (不指定则显示全部)
        #[arg(short, long)]
        setting: Option<String>,
    },

    /// 修改域名设置
    Set {
        /// 域名或 Zone ID
        domain: String,
        /// 设置项 ID
        key: String,
        /// 设置值
        value: String,
    },
}

impl ZoneArgs {
    pub async fn execute(&self, client: &CfClient, format: &str) -> Result<()> {
        match &self.command {
            ZoneCommands::List {
                name,
                status,
                per_page,
            } => {
                let params = ZoneListParams {
                    name: name.clone(),
                    status: status.clone(),
                    per_page: Some(*per_page),
                    ..Default::default()
                };
                let resp = client.list_zones(&params).await?;
                let zones = resp.result.unwrap_or_default();

                if format == "json" {
                    output::print_json(&zones);
                    return Ok(());
                }

                output::title(&format!("域名列表 (共 {}个)", zones.len()));

                if zones.is_empty() {
                    output::warn("没有找到域名");
                    return Ok(());
                }

                let mut table = output::create_table(vec![
                    "域名", "状态", "套餐", "NS 服务器", "ID",
                ]);

                for zone in &zones {
                    let plan_name = zone
                        .plan
                        .as_ref()
                        .and_then(|p| p.name.clone())
                        .unwrap_or_else(|| "-".to_string());
                    let ns = zone
                        .name_servers
                        .as_ref()
                        .map(|ns| ns.join(", "))
                        .unwrap_or_else(|| "-".to_string());

                    table.add_row(vec![
                        &zone.name,
                        &output::status_badge(&zone.status),
                        &plan_name,
                        &ns,
                        &zone.id[..8],
                    ]);
                }
                println!("{table}");
            }

            ZoneCommands::Get { domain } => {
                let zone_id = resolve_zone_id(client, domain).await?;
                let zone = client.get_zone(&zone_id).await?;

                if format == "json" {
                    output::print_json(&zone);
                    return Ok(());
                }

                output::title(&format!("域名详情: {}", zone.name));
                output::kv("Zone ID", &zone.id);
                output::kv("状态", &output::status_badge(&zone.status));
                output::kv("类型", zone.zone_type.as_deref().unwrap_or("-"));
                output::kv_colored(
                    "已暂停",
                    &zone.paused.map(|p| p.to_string()).unwrap_or("-".into()),
                    !zone.paused.unwrap_or(false),
                );
                output::kv(
                    "开发模式",
                    &zone
                        .development_mode
                        .map(|d| if d > 0 { format!("开启 (剩余 {}s)", d) } else { "关闭".into() })
                        .unwrap_or("-".into()),
                );

                if let Some(ns) = &zone.name_servers {
                    output::kv("NS 服务器", &ns.join(", "));
                }
                if let Some(ons) = &zone.original_name_servers {
                    output::kv("原始 NS", &ons.join(", "));
                }

                if let Some(plan) = &zone.plan {
                    output::kv("套餐", plan.name.as_deref().unwrap_or("-"));
                }
                if let Some(account) = &zone.account {
                    output::kv("账户", account.name.as_deref().unwrap_or("-"));
                }

                output::kv("创建时间", zone.created_on.as_deref().unwrap_or("-"));
                output::kv("修改时间", zone.modified_on.as_deref().unwrap_or("-"));
                output::kv("激活时间", zone.activated_on.as_deref().unwrap_or("-"));
            }

            ZoneCommands::Add {
                domain,
                account_id,
                jump_start,
            } => {
                let request = CreateZoneRequest {
                    name: domain.clone(),
                    account: account_id
                        .as_ref()
                        .map(|id| CreateZoneAccount { id: id.clone() }),
                    zone_type: None,
                    jump_start: *jump_start,
                };

                let zone = client.create_zone(&request).await?;
                output::success(&format!("域名 {} 添加成功！", zone.name));
                output::kv("Zone ID", &zone.id);
                output::kv("状态", &output::status_badge(&zone.status));
                if let Some(ns) = &zone.name_servers {
                    output::info("请将域名的 NS 记录修改为以下地址:");
                    for n in ns {
                        println!("  → {}", n.cyan());
                    }
                }
            }

            ZoneCommands::Delete { domain, yes } => {
                let zone_id = resolve_zone_id(client, domain).await?;

                if !yes {
                    let confirm = dialoguer::Confirm::new()
                        .with_prompt(format!("确定要删除域名 {} 吗？此操作不可逆！", domain.red()))
                        .default(false)
                        .interact()?;
                    if !confirm {
                        output::info("已取消删除操作");
                        return Ok(());
                    }
                }

                client.delete_zone(&zone_id).await?;
                output::success(&format!("域名 {} 已删除", domain));
            }

            ZoneCommands::Pause { domain } => {
                let zone_id = resolve_zone_id(client, domain).await?;
                let zone = client.toggle_zone_pause(&zone_id, true).await?;
                output::success(&format!("域名 {} 已暂停", zone.name));
            }

            ZoneCommands::Resume { domain } => {
                let zone_id = resolve_zone_id(client, domain).await?;
                let zone = client.toggle_zone_pause(&zone_id, false).await?;
                output::success(&format!("域名 {} 已恢复", zone.name));
            }

            ZoneCommands::Check { domain } => {
                let zone_id = resolve_zone_id(client, domain).await?;
                client.check_zone_activation(&zone_id).await?;
                output::success(&format!("已触发域名 {} 的激活检查", domain));
            }

            ZoneCommands::Settings { domain, setting } => {
                let zone_id = resolve_zone_id(client, domain).await?;

                if let Some(setting_id) = setting {
                    let s = client.get_zone_setting(&zone_id, setting_id).await?;
                    if format == "json" {
                        output::print_json(&s);
                    } else {
                        output::title(&format!("设置: {}", s.id));
                        output::kv("值", &serde_json::to_string(&s.value).unwrap_or_default());
                        output::kv(
                            "可编辑",
                            &s.editable.map(|e| e.to_string()).unwrap_or("-".into()),
                        );
                        output::kv("修改时间", s.modified_on.as_deref().unwrap_or("-"));
                    }
                } else {
                    let settings = client.get_zone_settings(&zone_id).await?;
                    if format == "json" {
                        output::print_json(&settings);
                        return Ok(());
                    }

                    output::title(&format!("域名 {} 的所有设置", domain));
                    let mut table =
                        output::create_table(vec!["设置项", "当前值", "可编辑", "修改时间"]);

                    for s in &settings {
                        let value_str = match &s.value {
                            serde_json::Value::String(v) => v.clone(),
                            other => serde_json::to_string(other).unwrap_or("-".into()),
                        };
                        table.add_row(vec![
                            &s.id,
                            &value_str,
                            &s.editable.map(|e| e.to_string()).unwrap_or("-".into()),
                            s.modified_on.as_deref().unwrap_or("-"),
                        ]);
                    }
                    println!("{table}");
                }
            }

            ZoneCommands::Set { domain, key, value } => {
                let zone_id = resolve_zone_id(client, domain).await?;

                // 尝试解析 value 为 JSON，否则当作字符串
                let json_value = serde_json::from_str(value)
                    .unwrap_or_else(|_| serde_json::Value::String(value.clone()));

                let setting = client
                    .update_zone_setting(&zone_id, key, json_value)
                    .await?;
                output::success(&format!(
                    "设置 {} = {} 已更新",
                    key,
                    serde_json::to_string(&setting.value).unwrap_or_default()
                ));
            }
        }

        Ok(())
    }
}

/// 解析域名或 Zone ID → Zone ID
pub async fn resolve_zone_id(client: &CfClient, domain_or_id: &str) -> Result<String> {
    // 如果看起来像是 Zone ID（32位十六进制），直接使用
    if domain_or_id.len() == 32 && domain_or_id.chars().all(|c| c.is_ascii_hexdigit()) {
        return Ok(domain_or_id.to_string());
    }
    // 否则按域名查找
    client.find_zone_id(domain_or_id).await
}
