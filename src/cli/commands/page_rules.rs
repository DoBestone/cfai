use anyhow::Result;
use clap::{Args, Subcommand};

use crate::api::client::CfClient;
use crate::cli::output;
use crate::cli::commands::zone::resolve_zone_id;

#[derive(Args, Debug)]
pub struct PageRulesArgs {
    #[command(subcommand)]
    pub command: PageRulesCommands,
}

#[derive(Subcommand, Debug)]
pub enum PageRulesCommands {
    /// 列出页面规则
    #[command(alias = "ls")]
    List {
        /// 域名或 Zone ID
        domain: String,
    },

    /// 查看页面规则详情
    Get {
        /// 域名或 Zone ID
        domain: String,
        /// 规则 ID
        rule_id: String,
    },

    /// 删除页面规则
    #[command(alias = "rm")]
    Delete {
        /// 域名或 Zone ID
        domain: String,
        /// 规则 ID
        rule_id: String,
        /// 跳过确认
        #[arg(short = 'y', long)]
        yes: bool,
    },

    /// 创建 URL 跳转规则
    Redirect {
        /// 域名或 Zone ID
        domain: String,
        /// URL 匹配模式 (如 *example.com/old/*)
        pattern: String,
        /// 跳转目标 URL
        target: String,
        /// HTTP 状态码 (301/302)
        #[arg(short, long, default_value = "301")]
        status: u16,
    },
}

impl PageRulesArgs {
    pub async fn execute(&self, client: &CfClient, format: &str) -> Result<()> {
        match &self.command {
            PageRulesCommands::List { domain } => {
                let zone_id = resolve_zone_id(client, domain).await?;
                let rules = client.list_page_rules(&zone_id).await?;

                if format == "json" {
                    output::print_json(&rules);
                    return Ok(());
                }

                output::title(&format!("页面规则 - {} (共 {} 条)", domain, rules.len()));

                if rules.is_empty() {
                    output::info("没有页面规则");
                    return Ok(());
                }

                let mut table =
                    output::create_table(vec!["ID", "URL 模式", "动作", "优先级", "状态"]);

                for rule in &rules {
                    let pattern = rule
                        .targets
                        .as_ref()
                        .and_then(|t| t.first())
                        .and_then(|t| t.constraint.as_ref())
                        .and_then(|c| c.value.clone())
                        .unwrap_or("-".into());

                    let actions: Vec<String> = rule
                        .actions
                        .as_ref()
                        .map(|acts| {
                            acts.iter()
                                .map(|a| a.id.clone().unwrap_or("-".into()))
                                .collect()
                        })
                        .unwrap_or_default();

                    table.add_row(vec![
                        &rule.id.as_deref().unwrap_or("-")[..8.min(rule.id.as_deref().unwrap_or("-").len())],
                        &pattern,
                        &actions.join(", "),
                        &rule.priority.map(|p| p.to_string()).unwrap_or("-".into()),
                        &output::status_badge(rule.status.as_deref().unwrap_or("-")),
                    ]);
                }
                println!("{table}");
            }

            PageRulesCommands::Get { domain, rule_id } => {
                let zone_id = resolve_zone_id(client, domain).await?;
                let rule = client.get_page_rule(&zone_id, rule_id).await?;

                if format == "json" {
                    output::print_json(&rule);
                    return Ok(());
                }

                output::title("页面规则详情");
                output::kv("ID", rule.id.as_deref().unwrap_or("-"));
                output::kv("状态", rule.status.as_deref().unwrap_or("-"));
                output::kv(
                    "优先级",
                    &rule.priority.map(|p| p.to_string()).unwrap_or("-".into()),
                );

                if let Some(targets) = &rule.targets {
                    for t in targets {
                        if let Some(c) = &t.constraint {
                            output::kv("URL 模式", c.value.as_deref().unwrap_or("-"));
                        }
                    }
                }

                if let Some(actions) = &rule.actions {
                    output::info("动作:");
                    for a in actions {
                        println!(
                            "  • {} = {}",
                            a.id.as_deref().unwrap_or("-"),
                            serde_json::to_string(&a.value).unwrap_or("-".into())
                        );
                    }
                }
            }

            PageRulesCommands::Delete {
                domain,
                rule_id,
                yes,
            } => {
                let zone_id = resolve_zone_id(client, domain).await?;

                if !yes {
                    let confirm = dialoguer::Confirm::new()
                        .with_prompt("确定要删除此页面规则吗？")
                        .default(false)
                        .interact()?;
                    if !confirm {
                        output::info("已取消");
                        return Ok(());
                    }
                }

                client.delete_page_rule(&zone_id, rule_id).await?;
                output::success("页面规则已删除");
            }

            PageRulesCommands::Redirect {
                domain,
                pattern,
                target,
                status,
            } => {
                let zone_id = resolve_zone_id(client, domain).await?;
                let rule = client
                    .create_redirect_rule(&zone_id, pattern, target, *status)
                    .await?;
                output::success(&format!(
                    "URL 跳转规则已创建 ({}): {} → {}",
                    status,
                    pattern,
                    target
                ));
                output::kv("规则 ID", rule.id.as_deref().unwrap_or("-"));
            }
        }

        Ok(())
    }
}
