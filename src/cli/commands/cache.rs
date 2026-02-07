use anyhow::Result;
use clap::{Args, Subcommand};

use crate::api::client::CfClient;
use crate::cli::output;
use crate::cli::commands::zone::resolve_zone_id;

#[derive(Args, Debug)]
pub struct CacheArgs {
    #[command(subcommand)]
    pub command: CacheCommands,
}

#[derive(Subcommand, Debug)]
pub enum CacheCommands {
    /// 清除全部缓存
    #[command(alias = "purge-all")]
    PurgeAll {
        /// 域名或 Zone ID
        domain: String,
        /// 跳过确认
        #[arg(short = 'y', long)]
        yes: bool,
    },

    /// 按 URL 清除缓存
    #[command(alias = "purge")]
    PurgeUrl {
        /// 域名或 Zone ID
        domain: String,
        /// 要清除缓存的 URL 列表
        #[arg(required = true)]
        urls: Vec<String>,
    },

    /// 按主机名清除缓存
    PurgeHost {
        /// 域名或 Zone ID
        domain: String,
        /// 主机名列表
        #[arg(required = true)]
        hosts: Vec<String>,
    },

    /// 查看缓存设置
    Status {
        /// 域名或 Zone ID
        domain: String,
    },

    /// 设置缓存级别 (aggressive/basic/simplified)
    Level {
        /// 域名或 Zone ID
        domain: String,
        /// 缓存级别
        level: String,
    },

    /// 设置浏览器缓存 TTL (秒)
    BrowserTtl {
        /// 域名或 Zone ID
        domain: String,
        /// TTL 值 (秒)
        ttl: u32,
    },

    /// 开启/关闭开发模式
    DevMode {
        /// 域名或 Zone ID
        domain: String,
        /// on/off
        #[arg(default_value = "on")]
        toggle: String,
    },
}

impl CacheArgs {
    pub async fn execute(&self, client: &CfClient, format: &str) -> Result<()> {
        match &self.command {
            CacheCommands::PurgeAll { domain, yes } => {
                let zone_id = resolve_zone_id(client, domain).await?;

                if !yes {
                    let confirm = dialoguer::Confirm::new()
                        .with_prompt(format!("确定要清除 {} 的全部缓存吗？", domain))
                        .default(false)
                        .interact()?;
                    if !confirm {
                        output::info("已取消");
                        return Ok(());
                    }
                }

                client.purge_all_cache(&zone_id).await?;
                output::success(&format!("已清除 {} 的全部缓存", domain));
            }

            CacheCommands::PurgeUrl { domain, urls } => {
                let zone_id = resolve_zone_id(client, domain).await?;
                client.purge_cache_by_urls(&zone_id, urls.clone()).await?;
                output::success(&format!("已清除 {} 个 URL 的缓存", urls.len()));
            }

            CacheCommands::PurgeHost { domain, hosts } => {
                let zone_id = resolve_zone_id(client, domain).await?;
                client
                    .purge_cache_by_hosts(&zone_id, hosts.clone())
                    .await?;
                output::success(&format!("已清除 {} 个主机名的缓存", hosts.len()));
            }

            CacheCommands::Status { domain } => {
                let zone_id = resolve_zone_id(client, domain).await?;
                let cache_level = client.get_cache_level(&zone_id).await?;
                let browser_ttl = client.get_browser_cache_ttl(&zone_id).await?;

                if format == "json" {
                    output::print_json(&serde_json::json!({
                        "cache_level": cache_level,
                        "browser_cache_ttl": browser_ttl,
                    }));
                    return Ok(());
                }

                output::title(&format!("缓存设置 - {}", domain));
                output::kv("缓存级别", &cache_level);
                output::kv(
                    "浏览器缓存 TTL",
                    &if browser_ttl == 0 {
                        "遵循源站".to_string()
                    } else {
                        format!("{} 秒 ({} 小时)", browser_ttl, browser_ttl / 3600)
                    },
                );
            }

            CacheCommands::Level { domain, level } => {
                let zone_id = resolve_zone_id(client, domain).await?;
                client.set_cache_level(&zone_id, level).await?;
                output::success(&format!("缓存级别已设置为: {}", level));
            }

            CacheCommands::BrowserTtl { domain, ttl } => {
                let zone_id = resolve_zone_id(client, domain).await?;
                client.set_browser_cache_ttl(&zone_id, *ttl).await?;
                output::success(&format!("浏览器缓存 TTL 已设置为: {} 秒", ttl));
            }

            CacheCommands::DevMode { domain, toggle } => {
                let zone_id = resolve_zone_id(client, domain).await?;
                let enable = toggle == "on";
                client.set_development_mode(&zone_id, enable).await?;
                output::success(&format!(
                    "开发模式已{}（缓存将在 3 小时后重新启用）",
                    if enable { "开启" } else { "关闭" }
                ));
            }
        }

        Ok(())
    }
}
