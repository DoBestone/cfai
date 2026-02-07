use anyhow::Result;
use clap::{Args, Subcommand};
use colored::Colorize;

use crate::api::client::CfClient;
use crate::cli::output;
use crate::cli::commands::zone::resolve_zone_id;

#[derive(Args, Debug)]
pub struct FirewallArgs {
    #[command(subcommand)]
    pub command: FirewallCommands,
}

#[derive(Subcommand, Debug)]
pub enum FirewallCommands {
    /// 查看安全概览
    Status {
        /// 域名或 Zone ID
        domain: String,
    },

    /// 列出防火墙规则
    #[command(alias = "ls")]
    List {
        /// 域名或 Zone ID
        domain: String,
    },

    /// 列出 IP 访问规则
    #[command(name = "ip-rules")]
    IpRules {
        /// 域名或 Zone ID
        domain: String,
    },

    /// 封禁 IP
    Block {
        /// 域名或 Zone ID
        domain: String,
        /// IP 地址
        ip: String,
        /// 备注
        #[arg(short, long)]
        note: Option<String>,
    },

    /// IP 白名单
    Whitelist {
        /// 域名或 Zone ID
        domain: String,
        /// IP 地址
        ip: String,
        /// 备注
        #[arg(short, long)]
        note: Option<String>,
    },

    /// 删除 IP 访问规则
    Unblock {
        /// 域名或 Zone ID
        domain: String,
        /// 规则 ID
        rule_id: String,
    },

    /// 设置安全级别
    Level {
        /// 域名或 Zone ID
        domain: String,
        /// 安全级别 (off/essentially_off/low/medium/high/under_attack)
        level: String,
    },

    /// 开启 Under Attack 模式
    #[command(name = "ua-on")]
    UnderAttackOn {
        /// 域名或 Zone ID
        domain: String,
    },

    /// 关闭 Under Attack 模式
    #[command(name = "ua-off")]
    UnderAttackOff {
        /// 域名或 Zone ID
        domain: String,
    },

    /// 列出速率限制规则
    RateLimits {
        /// 域名或 Zone ID
        domain: String,
    },
}

impl FirewallArgs {
    pub async fn execute(&self, client: &CfClient, format: &str) -> Result<()> {
        match &self.command {
            FirewallCommands::Status { domain } => {
                let zone_id = resolve_zone_id(client, domain).await?;
                let security_level = client.get_security_level(&zone_id).await?;

                if format == "json" {
                    output::print_json(&serde_json::json!({
                        "security_level": security_level,
                    }));
                    return Ok(());
                }

                output::title(&format!("安全概览 - {}", domain));
                output::kv_colored(
                    "安全级别",
                    &security_level,
                    security_level != "off" && security_level != "essentially_off",
                );
                output::kv_colored(
                    "Under Attack 模式",
                    if security_level == "under_attack" {
                        "🔴 开启"
                    } else {
                        "关闭"
                    },
                    security_level != "under_attack",
                );
            }

            FirewallCommands::List { domain } => {
                let zone_id = resolve_zone_id(client, domain).await?;
                let rules = client.list_firewall_rules(&zone_id).await?;

                if format == "json" {
                    output::print_json(&rules);
                    return Ok(());
                }

                output::title(&format!("防火墙规则 - {} (共 {} 条)", domain, rules.len()));

                if rules.is_empty() {
                    output::info("没有防火墙规则");
                    return Ok(());
                }

                let mut table = output::create_table(vec!["ID", "描述", "动作", "暂停", "表达式"]);
                for rule in &rules {
                    let expression = rule
                        .filter
                        .as_ref()
                        .and_then(|f| f.expression.clone())
                        .unwrap_or("-".into());
                    let expr_short = if expression.len() > 50 {
                        format!("{}...", &expression[..47])
                    } else {
                        expression
                    };

                    table.add_row(vec![
                        rule.id.as_deref().unwrap_or("-"),
                        rule.description.as_deref().unwrap_or("-"),
                        rule.action.as_deref().unwrap_or("-"),
                        &rule.paused.map(|p| p.to_string()).unwrap_or("-".into()),
                        &expr_short,
                    ]);
                }
                println!("{table}");
            }

            FirewallCommands::IpRules { domain } => {
                let zone_id = resolve_zone_id(client, domain).await?;
                let rules = client.list_ip_access_rules(&zone_id).await?;

                if format == "json" {
                    output::print_json(&rules);
                    return Ok(());
                }

                output::title(&format!("IP 访问规则 - {} (共 {} 条)", domain, rules.len()));

                let mut table = output::create_table(vec!["ID", "模式", "目标", "值", "备注", "创建时间"]);
                for rule in &rules {
                    let (target, value) = rule
                        .configuration
                        .as_ref()
                        .map(|c| {
                            (
                                c.target.as_deref().unwrap_or("-"),
                                c.value.as_deref().unwrap_or("-"),
                            )
                        })
                        .unwrap_or(("-", "-"));

                    table.add_row(vec![
                        rule.id.as_deref().unwrap_or("-"),
                        rule.mode.as_deref().unwrap_or("-"),
                        target,
                        value,
                        rule.notes.as_deref().unwrap_or("-"),
                        rule.created_on.as_deref().unwrap_or("-"),
                    ]);
                }
                println!("{table}");
            }

            FirewallCommands::Block { domain, ip, note } => {
                let zone_id = resolve_zone_id(client, domain).await?;
                client.block_ip(&zone_id, ip, note.as_deref()).await?;
                output::success(&format!("已封禁 IP: {}", ip.red()));
            }

            FirewallCommands::Whitelist { domain, ip, note } => {
                let zone_id = resolve_zone_id(client, domain).await?;
                client.whitelist_ip(&zone_id, ip, note.as_deref()).await?;
                output::success(&format!("已添加白名单: {}", ip));
            }

            FirewallCommands::Unblock {
                domain,
                rule_id,
            } => {
                let zone_id = resolve_zone_id(client, domain).await?;
                client.delete_ip_access_rule(&zone_id, rule_id).await?;
                output::success("IP 访问规则已删除");
            }

            FirewallCommands::Level { domain, level } => {
                let zone_id = resolve_zone_id(client, domain).await?;
                client.set_security_level(&zone_id, level).await?;
                output::success(&format!("安全级别已设置为: {}", level));
            }

            FirewallCommands::UnderAttackOn { domain } => {
                let zone_id = resolve_zone_id(client, domain).await?;
                client.set_under_attack_mode(&zone_id, true).await?;
                output::success(&format!("🔴 {} Under Attack 模式已开启！", domain));
            }

            FirewallCommands::UnderAttackOff { domain } => {
                let zone_id = resolve_zone_id(client, domain).await?;
                client.set_under_attack_mode(&zone_id, false).await?;
                output::success(&format!("{} Under Attack 模式已关闭", domain));
            }

            FirewallCommands::RateLimits { domain } => {
                let zone_id = resolve_zone_id(client, domain).await?;
                let rules = client.list_rate_limits(&zone_id).await?;

                if format == "json" {
                    output::print_json(&rules);
                    return Ok(());
                }

                output::title(&format!("速率限制规则 - {} (共 {} 条)", domain, rules.len()));
                for rule in &rules {
                    output::kv("ID", rule.id.as_deref().unwrap_or("-"));
                    output::kv("描述", rule.description.as_deref().unwrap_or("-"));
                    output::kv(
                        "阈值",
                        &rule
                            .threshold
                            .map(|t| format!("{} 次/{}s", t, rule.period.unwrap_or(0)))
                            .unwrap_or("-".into()),
                    );
                    output::kv(
                        "动作",
                        &rule
                            .action
                            .as_ref()
                            .and_then(|a| a.mode.clone())
                            .unwrap_or("-".into()),
                    );
                    println!();
                }
            }
        }

        Ok(())
    }
}
