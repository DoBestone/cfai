use anyhow::Result;
use clap::{Args, Subcommand};

use crate::cli::output;
use crate::config::settings::AppConfig;

#[derive(Args, Debug)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigCommands,
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// 交互式配置向导
    Setup,

    /// 查看当前配置
    Show {
        /// 显示敏感信息 (API Key 等)
        #[arg(long)]
        show_secrets: bool,
    },

    /// 设置配置项
    Set {
        /// 配置键 (如 cloudflare.api_token, ai.model)
        key: String,
        /// 配置值
        value: String,
    },

    /// 查看配置文件路径
    Path,

    /// 验证配置
    Verify,
}

impl ConfigArgs {
    pub async fn execute(&self) -> Result<()> {
        match &self.command {
            ConfigCommands::Setup => {
                AppConfig::interactive_setup()?;
            }

            ConfigCommands::Show { show_secrets } => {
                let config = AppConfig::load()?.merge_env();

                output::title("当前配置");

                output::info("Cloudflare:");
                output::kv(
                    "API Token",
                    &mask_secret(
                        config.cloudflare.api_token.as_deref(),
                        *show_secrets,
                    ),
                );
                output::kv(
                    "Email",
                    config.cloudflare.email.as_deref().unwrap_or("(未设置)"),
                );
                output::kv(
                    "API Key",
                    &mask_secret(config.cloudflare.api_key.as_deref(), *show_secrets),
                );
                output::kv(
                    "Account ID",
                    config.cloudflare.account_id.as_deref().unwrap_or("(未设置)"),
                );

                println!();
                output::info("AI:");
                output::kv(
                    "API URL",
                    config.ai.api_url.as_deref().unwrap_or("(未设置)"),
                );
                output::kv(
                    "API Key",
                    &mask_secret(config.ai.api_key.as_deref(), *show_secrets),
                );
                output::kv(
                    "模型",
                    config.ai.model.as_deref().unwrap_or("(未设置)"),
                );
                output::kv(
                    "Max Tokens",
                    &config
                        .ai
                        .max_tokens
                        .map(|t| t.to_string())
                        .unwrap_or("(默认)".into()),
                );
                output::kv(
                    "Temperature",
                    &config
                        .ai
                        .temperature
                        .map(|t| t.to_string())
                        .unwrap_or("(默认)".into()),
                );

                println!();
                output::info("默认设置:");
                output::kv(
                    "默认域名",
                    config.defaults.domain.as_deref().unwrap_or("(未设置)"),
                );
                output::kv(
                    "输出格式",
                    config
                        .defaults
                        .output_format
                        .as_deref()
                        .unwrap_or("table"),
                );
            }

            ConfigCommands::Set { key, value } => {
                let mut config = AppConfig::load()?.merge_env();

                match key.as_str() {
                    "cloudflare.api_token" => config.cloudflare.api_token = Some(value.clone()),
                    "cloudflare.email" => config.cloudflare.email = Some(value.clone()),
                    "cloudflare.api_key" => config.cloudflare.api_key = Some(value.clone()),
                    "cloudflare.account_id" => config.cloudflare.account_id = Some(value.clone()),
                    "ai.api_url" => config.ai.api_url = Some(value.clone()),
                    "ai.api_key" => config.ai.api_key = Some(value.clone()),
                    "ai.model" => config.ai.model = Some(value.clone()),
                    "ai.max_tokens" => {
                        config.ai.max_tokens = Some(value.parse().map_err(|_| {
                            anyhow::anyhow!("max_tokens 必须是数字")
                        })?);
                    }
                    "ai.temperature" => {
                        config.ai.temperature = Some(value.parse().map_err(|_| {
                            anyhow::anyhow!("temperature 必须是数字")
                        })?);
                    }
                    "defaults.domain" => config.defaults.domain = Some(value.clone()),
                    "defaults.output_format" => {
                        config.defaults.output_format = Some(value.clone());
                    }
                    _ => anyhow::bail!("未知的配置项: {}\n可用配置项: cloudflare.api_token, cloudflare.email, cloudflare.api_key, cloudflare.account_id, ai.api_url, ai.api_key, ai.model, ai.max_tokens, ai.temperature, defaults.domain, defaults.output_format", key),
                }

                config.save()?;
                output::success(&format!("配置 {} 已更新", key));
            }

            ConfigCommands::Path => {
                let path = AppConfig::config_path()?;
                println!("{}", path.display());
            }

            ConfigCommands::Verify => {
                let config = AppConfig::load()?.merge_env();

                output::title("验证配置");

                // 检查 Cloudflare 认证
                match config.validate() {
                    Ok(()) => output::success("Cloudflare 认证配置 ✓"),
                    Err(e) => output::error(&format!("Cloudflare 认证: {}", e)),
                }

                // 检查 AI 配置
                if config.ai.api_key.is_some() {
                    output::success("AI API Key 已配置 ✓");
                } else {
                    output::warn("AI API Key 未配置 (AI 功能将不可用)");
                }

                // 检查 Account ID
                if config.cloudflare.account_id.is_some() {
                    output::success("Account ID 已配置 ✓");
                } else {
                    output::warn("Account ID 未配置 (Workers 等功能将不可用)");
                }
            }
        }

        Ok(())
    }
}

/// 遮蔽敏感信息
fn mask_secret(value: Option<&str>, show: bool) -> String {
    match value {
        None => "(未设置)".to_string(),
        Some(v) if show => v.to_string(),
        Some(v) if v.len() > 8 => format!("{}...{}", &v[..4], &v[v.len() - 4..]),
        Some(_) => "****".to_string(),
    }
}
