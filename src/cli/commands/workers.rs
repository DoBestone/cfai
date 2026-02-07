use anyhow::Result;
use clap::{Args, Subcommand};

use crate::api::client::CfClient;
use crate::cli::output;
use crate::config::settings::AppConfig;

#[derive(Args, Debug)]
pub struct WorkersArgs {
    #[command(subcommand)]
    pub command: WorkersCommands,
}

#[derive(Subcommand, Debug)]
pub enum WorkersCommands {
    /// 列出 Workers 脚本
    #[command(alias = "ls")]
    List,

    /// 删除 Workers 脚本
    #[command(alias = "rm")]
    Delete {
        /// 脚本名称
        name: String,
        /// 跳过确认
        #[arg(short = 'y', long)]
        yes: bool,
    },

    /// 列出 Workers 路由
    Routes {
        /// 域名或 Zone ID
        domain: String,
    },

    /// 列出 KV 命名空间
    Kv,

    /// 列出 Workers 自定义域名
    Domains,
}

impl WorkersArgs {
    pub async fn execute(&self, client: &CfClient, config: &AppConfig, format: &str) -> Result<()> {
        let account_id = config
            .cloudflare
            .account_id
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("Workers 管理需要 Account ID，请运行 `cfai config setup`"))?;

        match &self.command {
            WorkersCommands::List => {
                let scripts = client.list_workers(account_id).await?;

                if format == "json" {
                    output::print_json(&scripts);
                    return Ok(());
                }

                output::title(&format!("Workers 脚本 (共 {} 个)", scripts.len()));

                if scripts.is_empty() {
                    output::info("没有 Workers 脚本");
                    return Ok(());
                }

                let mut table =
                    output::create_table(vec!["名称", "用量模型", "处理器", "创建时间", "修改时间"]);
                for s in &scripts {
                    table.add_row(vec![
                        s.id.as_deref().unwrap_or("-"),
                        s.usage_model.as_deref().unwrap_or("-"),
                        &s.handlers
                            .as_ref()
                            .map(|h| h.join(", "))
                            .unwrap_or("-".into()),
                        s.created_on.as_deref().unwrap_or("-"),
                        s.modified_on.as_deref().unwrap_or("-"),
                    ]);
                }
                println!("{table}");
            }

            WorkersCommands::Delete { name, yes } => {
                if !yes {
                    let confirm = dialoguer::Confirm::new()
                        .with_prompt(format!("确定要删除 Worker {} 吗？", name))
                        .default(false)
                        .interact()?;
                    if !confirm {
                        output::info("已取消");
                        return Ok(());
                    }
                }

                client.delete_worker(account_id, name).await?;
                output::success(&format!("Worker {} 已删除", name));
            }

            WorkersCommands::Routes { domain } => {
                let zone_id = crate::cli::commands::zone::resolve_zone_id(client, domain).await?;
                let routes = client.list_worker_routes(&zone_id).await?;

                if format == "json" {
                    output::print_json(&routes);
                    return Ok(());
                }

                output::title(&format!("Workers 路由 - {} (共 {} 条)", domain, routes.len()));

                let mut table = output::create_table(vec!["ID", "模式", "脚本"]);
                for r in &routes {
                    table.add_row(vec![
                        r.id.as_deref().unwrap_or("-"),
                        r.pattern.as_deref().unwrap_or("-"),
                        r.script.as_deref().unwrap_or("(无)"),
                    ]);
                }
                println!("{table}");
            }

            WorkersCommands::Kv => {
                let namespaces = client.list_kv_namespaces(account_id).await?;

                if format == "json" {
                    output::print_json(&namespaces);
                    return Ok(());
                }

                output::title(&format!("KV 命名空间 (共 {} 个)", namespaces.len()));

                let mut table = output::create_table(vec!["ID", "名称"]);
                for ns in &namespaces {
                    table.add_row(vec![
                        ns.id.as_deref().unwrap_or("-"),
                        ns.title.as_deref().unwrap_or("-"),
                    ]);
                }
                println!("{table}");
            }

            WorkersCommands::Domains => {
                let domains = client.list_worker_domains(account_id).await?;

                if format == "json" {
                    output::print_json(&domains);
                    return Ok(());
                }

                output::title(&format!("Workers 自定义域名 (共 {} 个)", domains.len()));

                let mut table =
                    output::create_table(vec!["ID", "主机名", "服务", "环境", "Zone"]);
                for d in &domains {
                    table.add_row(vec![
                        d.id.as_deref().unwrap_or("-"),
                        d.hostname.as_deref().unwrap_or("-"),
                        d.service.as_deref().unwrap_or("-"),
                        d.environment.as_deref().unwrap_or("-"),
                        d.zone_name.as_deref().unwrap_or("-"),
                    ]);
                }
                println!("{table}");
            }
        }

        Ok(())
    }
}
