use anyhow::Result;
use clap::{Args, Subcommand};
use colored::Colorize;
use dialoguer::Confirm;

use crate::ai::analyzer::AiAnalyzer;
use crate::ai::executor;
use crate::api::client::CfClient;
use crate::cli::output;
use crate::cli::commands::zone::resolve_zone_id;
use crate::config::settings::AppConfig;
use crate::models::dns::DnsListParams;

#[derive(Args, Debug)]
pub struct AiArgs {
    #[command(subcommand)]
    pub command: AiCommands,
}

#[derive(Subcommand, Debug)]
pub enum AiCommands {
    /// 自由问答 - 向 AI 提问关于 Cloudflare 的任何问题
    Ask {
        /// 你的问题
        question: Vec<String>,
    },

    /// 全面分析域名配置
    Analyze {
        /// 域名或 Zone ID
        domain: String,
        /// 分析类型 (all/dns/security/performance)
        #[arg(short = 't', long, default_value = "all")]
        analysis_type: String,
    },

    /// 故障诊断 - 描述问题让 AI 帮你排查
    Troubleshoot {
        /// 问题描述
        issue: Vec<String>,
        /// 相关域名 (可选)
        #[arg(short, long)]
        domain: Option<String>,
    },

    /// 自动配置 - 描述需求让 AI 生成配置方案
    AutoConfig {
        /// 配置需求描述
        requirement: Vec<String>,
        /// 相关域名 (可选)
        #[arg(short, long)]
        domain: Option<String>,
        /// 自动执行建议的操作 (危险!)
        #[arg(long)]
        auto_apply: bool,
    },
}

impl AiArgs {
    pub async fn execute(&self, client: &CfClient, config: &AppConfig, _format: &str) -> Result<()> {
        let analyzer = AiAnalyzer::new(config)?;

        match &self.command {
            AiCommands::Ask { question } => {
                let question_str = question.join(" ");
                let spinner = indicatif::ProgressBar::new_spinner();
                spinner.set_message("🤖 AI 正在思考...");
                spinner.enable_steady_tick(std::time::Duration::from_millis(100));

                let result = analyzer.ask(&question_str).await?;

                spinner.finish_and_clear();
                output::print_ai_result(&result.content, result.tokens_used);

                if let Some(actions) = &result.actions {
                    output::print_ai_actions(actions);
                    if !actions.is_empty() {
                        println!(
                            "\n{}",
                            "💡 Ask 模式无域名上下文，如需执行建议操作请使用 analyze/troubleshoot/auto-config 并指定域名"
                                .dimmed()
                        );
                    }
                }
            }

            AiCommands::Analyze {
                domain,
                analysis_type,
            } => {
                let zone_id = resolve_zone_id(client, domain).await?;

                let spinner = indicatif::ProgressBar::new_spinner();
                spinner.set_message("📊 正在收集域名配置信息...");
                spinner.enable_steady_tick(std::time::Duration::from_millis(100));

                // 收集配置信息
                let mut context = String::new();

                match analysis_type.as_str() {
                    "dns" | "all" => {
                        context.push_str("## DNS 记录\n");
                        let dns_params = DnsListParams::default();
                        if let Ok(resp) = client.list_dns_records(&zone_id, &dns_params).await {
                            if let Some(records) = resp.result {
                                for r in &records {
                                    context.push_str(&format!(
                                        "{} {} → {} (代理: {}, TTL: {})\n",
                                        r.record_type,
                                        r.name,
                                        r.content,
                                        r.proxied.map(|p| p.to_string()).unwrap_or("-".into()),
                                        r.ttl.map(|t| t.to_string()).unwrap_or("-".into()),
                                    ));
                                }
                            }
                        }
                    }
                    _ => {}
                }

                match analysis_type.as_str() {
                    "security" | "all" => {
                        context.push_str("\n## 安全配置\n");
                        if let Ok(mode) = client.get_ssl_mode(&zone_id).await {
                            context.push_str(&format!("SSL 模式: {}\n", mode));
                        }
                        if let Ok(https) = client.get_always_https(&zone_id).await {
                            context.push_str(&format!("Always HTTPS: {}\n", https));
                        }
                        if let Ok(level) = client.get_security_level(&zone_id).await {
                            context.push_str(&format!("安全级别: {}\n", level));
                        }
                    }
                    _ => {}
                }

                match analysis_type.as_str() {
                    "performance" | "all" => {
                        context.push_str("\n## 性能配置\n");
                        if let Ok(level) = client.get_cache_level(&zone_id).await {
                            context.push_str(&format!("缓存级别: {}\n", level));
                        }
                        if let Ok(ttl) = client.get_browser_cache_ttl(&zone_id).await {
                            context.push_str(&format!("浏览器缓存 TTL: {}s\n", ttl));
                        }
                    }
                    _ => {}
                }

                spinner.set_message("🤖 AI 正在分析...");

                let result = match analysis_type.as_str() {
                    "dns" => analyzer.analyze_dns(&context).await?,
                    "security" => analyzer.analyze_security(&context).await?,
                    "performance" => analyzer.analyze_performance(&context).await?,
                    "all" => {
                        let full_prompt = format!(
                            "请对域名 {} 进行全面分析，包括 DNS、安全和性能方面:\n\n{}",
                            domain, context
                        );
                        analyzer.ask(&full_prompt).await?
                    }
                    _ => anyhow::bail!("未知的分析类型: {}", analysis_type),
                };

                spinner.finish_and_clear();
                output::print_ai_result(&result.content, result.tokens_used);

                if let Some(actions) = &result.actions {
                    output::print_ai_actions(actions);
                    prompt_execute_actions(client, &zone_id, actions).await?;
                }
            }

            AiCommands::Troubleshoot { issue, domain } => {
                let issue_str = issue.join(" ");
                let resolved_zone_id = if let Some(d) = domain {
                    Some(resolve_zone_id(client, d).await?)
                } else {
                    None
                };

                let spinner = indicatif::ProgressBar::new_spinner();
                spinner.set_message("🔍 正在诊断...");
                spinner.enable_steady_tick(std::time::Duration::from_millis(100));

                let result = if let (Some(domain), Some(zone_id)) = (domain, &resolved_zone_id) {
                    let mut context = format!("域名: {}\n", domain);

                    if let Ok(zone) = client.get_zone(zone_id).await {
                        context.push_str(&format!("状态: {}\n", zone.status));
                    }
                    if let Ok(mode) = client.get_ssl_mode(zone_id).await {
                        context.push_str(&format!("SSL: {}\n", mode));
                    }

                    analyzer
                        .ask_with_context(&format!("故障诊断请求: {}", issue_str), &context)
                        .await?
                } else {
                    analyzer.troubleshoot(&issue_str).await?
                };

                spinner.finish_and_clear();
                output::print_ai_result(&result.content, result.tokens_used);

                if let Some(actions) = &result.actions {
                    output::print_ai_actions(actions);
                    if let Some(zone_id) = &resolved_zone_id {
                        prompt_execute_actions(client, zone_id, actions).await?;
                    } else if !actions.is_empty() {
                        println!(
                            "\n{}",
                            "💡 指定 --domain 参数后可执行建议操作".dimmed()
                        );
                    }
                }
            }

            AiCommands::AutoConfig {
                requirement,
                domain,
                auto_apply,
            } => {
                let req_str = requirement.join(" ");

                let spinner = indicatif::ProgressBar::new_spinner();
                spinner.set_message("🤖 AI 正在生成配置方案...");
                spinner.enable_steady_tick(std::time::Duration::from_millis(100));

                let result = analyzer.auto_config(&req_str).await?;

                spinner.finish_and_clear();
                output::print_ai_result(&result.content, result.tokens_used);

                if let Some(actions) = &result.actions {
                    output::print_ai_actions(actions);

                    if !actions.is_empty() {
                        if let Some(domain) = domain {
                            let zone_id = resolve_zone_id(client, domain).await?;
                            if *auto_apply {
                                executor::execute_actions(client, &zone_id, actions).await?;
                            } else {
                                prompt_execute_actions(client, &zone_id, actions).await?;
                            }
                        } else {
                            println!(
                                "\n{}",
                                "💡 指定 --domain 参数后可执行建议操作".dimmed()
                            );
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

/// 交互式提示用户是否执行 AI 建议的操作
async fn prompt_execute_actions(
    client: &CfClient,
    zone_id: &str,
    actions: &[crate::ai::analyzer::SuggestedAction],
) -> Result<()> {
    if actions.is_empty() {
        return Ok(());
    }

    println!();
    let confirm = Confirm::new()
        .with_prompt("是否执行以上建议操作?")
        .default(false)
        .interact()?;

    if confirm {
        executor::execute_actions(client, zone_id, actions).await?;
    }

    Ok(())
}
