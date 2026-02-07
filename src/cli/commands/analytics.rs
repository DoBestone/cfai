use anyhow::Result;
use clap::{Args, Subcommand};

use crate::api::client::CfClient;
use crate::cli::output;
use crate::cli::commands::zone::resolve_zone_id;
use crate::models::analytics::AnalyticsParams;

#[derive(Args, Debug)]
pub struct AnalyticsArgs {
    #[command(subcommand)]
    pub command: AnalyticsCommands,
}

#[derive(Subcommand, Debug)]
pub enum AnalyticsCommands {
    /// 查看域名流量概览 (最近 24 小时)
    Overview {
        /// 域名或 Zone ID
        domain: String,
    },

    /// 查看详细分析数据
    Detail {
        /// 域名或 Zone ID
        domain: String,
        /// 起始时间 (如 -1440 表示 24 小时前, 或 ISO8601 格式)
        #[arg(short, long, default_value = "-1440")]
        since: String,
        /// 结束时间
        #[arg(short, long, default_value = "0")]
        until: String,
    },
}

impl AnalyticsArgs {
    pub async fn execute(&self, client: &CfClient, format: &str) -> Result<()> {
        match &self.command {
            AnalyticsCommands::Overview { domain } => {
                let zone_id = resolve_zone_id(client, domain).await?;
                let dashboard = client.get_analytics_24h(&zone_id).await?;

                if format == "json" {
                    output::print_json(&dashboard);
                    return Ok(());
                }

                output::title(&format!("流量概览 - {} (最近 24 小时)", domain));

                if let Some(totals) = &dashboard.totals {
                    // 请求统计
                    if let Some(requests) = &totals.requests {
                        output::info("📊 请求统计");
                        output::kv(
                            "总请求数",
                            &output::format_number(requests.all.unwrap_or(0)),
                        );
                        output::kv(
                            "已缓存",
                            &output::format_number(requests.cached.unwrap_or(0)),
                        );
                        output::kv(
                            "未缓存",
                            &output::format_number(requests.uncached.unwrap_or(0)),
                        );

                        let total = requests.all.unwrap_or(1).max(1);
                        let cached = requests.cached.unwrap_or(0);
                        let cache_rate = (cached as f64 / total as f64) * 100.0;
                        output::kv_colored(
                            "缓存命中率",
                            &format!("{:.1}%", cache_rate),
                            cache_rate > 50.0,
                        );

                        if let Some(ssl) = &requests.ssl {
                            output::kv(
                                "HTTPS 请求",
                                &output::format_number(ssl.encrypted.unwrap_or(0)),
                            );
                            output::kv(
                                "HTTP 请求",
                                &output::format_number(ssl.unencrypted.unwrap_or(0)),
                            );
                        }
                    }

                    println!();

                    // 带宽统计
                    if let Some(bandwidth) = &totals.bandwidth {
                        output::info("📶 带宽统计");
                        output::kv(
                            "总带宽",
                            &output::format_bytes(bandwidth.all.unwrap_or(0)),
                        );
                        output::kv(
                            "已缓存",
                            &output::format_bytes(bandwidth.cached.unwrap_or(0)),
                        );
                        output::kv(
                            "未缓存",
                            &output::format_bytes(bandwidth.uncached.unwrap_or(0)),
                        );
                    }

                    println!();

                    // 安全统计
                    if let Some(threats) = &totals.threats {
                        output::info("🛡️ 安全统计");
                        output::kv_colored(
                            "威胁总数",
                            &output::format_number(threats.all.unwrap_or(0)),
                            threats.all.unwrap_or(0) == 0,
                        );
                    }

                    // 页面浏览
                    if let Some(pageviews) = &totals.pageviews {
                        output::info("👁️ 页面浏览");
                        output::kv(
                            "总浏览量",
                            &output::format_number(pageviews.all.unwrap_or(0)),
                        );
                    }

                    // 独立访客
                    if let Some(uniques) = &totals.uniques {
                        output::kv(
                            "独立访客",
                            &output::format_number(uniques.all.unwrap_or(0)),
                        );
                    }
                }
            }

            AnalyticsCommands::Detail {
                domain,
                since,
                until,
            } => {
                let zone_id = resolve_zone_id(client, domain).await?;
                let params = AnalyticsParams {
                    since: Some(since.clone()),
                    until: Some(until.clone()),
                    continuous: Some(true),
                };
                let dashboard = client.get_analytics(&zone_id, &params).await?;

                if format == "json" {
                    output::print_json(&dashboard);
                    return Ok(());
                }

                output::title(&format!("详细分析 - {} ({} ~ {})", domain, since, until));
                // 打印与 Overview 相同的摘要
                if let Some(totals) = &dashboard.totals {
                    if let Some(requests) = &totals.requests {
                        output::kv(
                            "总请求数",
                            &output::format_number(requests.all.unwrap_or(0)),
                        );
                    }
                    if let Some(bandwidth) = &totals.bandwidth {
                        output::kv(
                            "总带宽",
                            &output::format_bytes(bandwidth.all.unwrap_or(0)),
                        );
                    }
                }

                output::info("💡 提示: 使用 --format json 获取完整的时间序列数据");
            }
        }

        Ok(())
    }
}
