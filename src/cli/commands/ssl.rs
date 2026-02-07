use anyhow::Result;
use clap::{Args, Subcommand};

use crate::api::client::CfClient;
use crate::cli::output;
use crate::cli::commands::zone::resolve_zone_id;

#[derive(Args, Debug)]
pub struct SslArgs {
    #[command(subcommand)]
    pub command: SslCommands,
}

#[derive(Subcommand, Debug)]
pub enum SslCommands {
    /// 查看 SSL/TLS 模式
    Status {
        /// 域名或 Zone ID
        domain: String,
    },

    /// 设置 SSL/TLS 模式 (off/flexible/full/strict)
    Mode {
        /// 域名或 Zone ID
        domain: String,
        /// SSL 模式
        mode: String,
    },

    /// 查看 SSL 验证状态
    Verify {
        /// 域名或 Zone ID
        domain: String,
    },

    /// 列出 SSL 证书
    #[command(alias = "ls")]
    List {
        /// 域名或 Zone ID
        domain: String,
    },

    /// 设置 Always Use HTTPS
    Https {
        /// 域名或 Zone ID
        domain: String,
        /// on/off
        #[arg(default_value = "on")]
        toggle: String,
    },

    /// 设置最小 TLS 版本
    MinTls {
        /// 域名或 Zone ID
        domain: String,
        /// TLS 版本 (1.0/1.1/1.2/1.3)
        version: String,
    },

    /// 列出源服务器证书
    OriginCerts {
        /// 域名或 Zone ID
        domain: String,
    },

    /// 设置自动 HTTPS 重写
    AutoRewrite {
        /// 域名或 Zone ID
        domain: String,
        /// on/off
        #[arg(default_value = "on")]
        toggle: String,
    },
}

impl SslArgs {
    pub async fn execute(&self, client: &CfClient, format: &str) -> Result<()> {
        match &self.command {
            SslCommands::Status { domain } => {
                let zone_id = resolve_zone_id(client, domain).await?;
                let mode = client.get_ssl_mode(&zone_id).await?;
                let always_https = client.get_always_https(&zone_id).await?;

                if format == "json" {
                    output::print_json(&serde_json::json!({
                        "ssl_mode": mode,
                        "always_https": always_https,
                    }));
                    return Ok(());
                }

                output::title(&format!("SSL/TLS 状态 - {}", domain));
                output::kv_colored("SSL 模式", &mode, mode != "off");
                output::kv_colored(
                    "Always HTTPS",
                    if always_https { "开启" } else { "关闭" },
                    always_https,
                );
            }

            SslCommands::Mode { domain, mode } => {
                let zone_id = resolve_zone_id(client, domain).await?;
                client.set_ssl_mode(&zone_id, mode).await?;
                output::success(&format!("SSL 模式已设置为: {}", mode));
            }

            SslCommands::Verify { domain } => {
                let zone_id = resolve_zone_id(client, domain).await?;
                let verifications = client.get_ssl_verification(&zone_id).await?;

                if format == "json" {
                    output::print_json(&verifications);
                    return Ok(());
                }

                output::title(&format!("SSL 验证状态 - {}", domain));
                for v in &verifications {
                    output::kv(
                        "主机名",
                        v.hostname.as_deref().unwrap_or("-"),
                    );
                    output::kv_colored(
                        "证书状态",
                        v.certificate_status.as_deref().unwrap_or("-"),
                        v.certificate_status.as_deref() == Some("active"),
                    );
                    output::kv(
                        "验证类型",
                        v.verification_type.as_deref().unwrap_or("-"),
                    );
                    output::kv(
                        "验证状态",
                        v.verification_status.as_deref().unwrap_or("-"),
                    );
                    println!();
                }
            }

            SslCommands::List { domain } => {
                let zone_id = resolve_zone_id(client, domain).await?;
                let certs = client.list_ssl_certificates(&zone_id).await?;

                if format == "json" {
                    output::print_json(&certs);
                    return Ok(());
                }

                output::title(&format!("SSL 证书 - {} (共 {} 个)", domain, certs.len()));
                for cert in &certs {
                    output::kv("ID", cert.id.as_deref().unwrap_or("-"));
                    output::kv(
                        "主机",
                        &cert
                            .hosts
                            .as_ref()
                            .map(|h| h.join(", "))
                            .unwrap_or("-".into()),
                    );
                    output::kv("状态", cert.status.as_deref().unwrap_or("-"));
                    output::kv("签发者", cert.issuer.as_deref().unwrap_or("-"));
                    output::kv("过期时间", cert.expires_on.as_deref().unwrap_or("-"));
                    println!();
                }
            }

            SslCommands::Https { domain, toggle } => {
                let zone_id = resolve_zone_id(client, domain).await?;
                let enable = toggle == "on";
                client.set_always_https(&zone_id, enable).await?;
                output::success(&format!(
                    "Always Use HTTPS 已{}",
                    if enable { "开启" } else { "关闭" }
                ));
            }

            SslCommands::MinTls { domain, version } => {
                let zone_id = resolve_zone_id(client, domain).await?;
                client.set_ssl_min_tls(&zone_id, version).await?;
                output::success(&format!("最小 TLS 版本已设置为: {}", version));
            }

            SslCommands::OriginCerts { domain } => {
                let zone_id = resolve_zone_id(client, domain).await?;
                let certs = client.list_origin_certificates(&zone_id).await?;

                if format == "json" {
                    output::print_json(&certs);
                    return Ok(());
                }

                output::title(&format!("源服务器证书 - {} (共 {} 个)", domain, certs.len()));
                for cert in &certs {
                    output::kv("ID", cert.id.as_deref().unwrap_or("-"));
                    output::kv(
                        "主机名",
                        &cert
                            .hostnames
                            .as_ref()
                            .map(|h| h.join(", "))
                            .unwrap_or("-".into()),
                    );
                    output::kv("过期时间", cert.expires_on.as_deref().unwrap_or("-"));
                    println!();
                }
            }

            SslCommands::AutoRewrite { domain, toggle } => {
                let zone_id = resolve_zone_id(client, domain).await?;
                let enable = toggle == "on";
                client
                    .set_automatic_https_rewrites(&zone_id, enable)
                    .await?;
                output::success(&format!(
                    "自动 HTTPS 重写已{}",
                    if enable { "开启" } else { "关闭" }
                ));
            }
        }

        Ok(())
    }
}
