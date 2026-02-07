mod ai;
mod api;
mod cli;
mod config;
mod models;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;

use crate::api::client::{AuthMethod, CfClient};
use crate::cli::commands::{Cli, Commands};
use crate::cli::output;
use crate::config::settings::AppConfig;

#[tokio::main]
async fn main() {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::WARN.into()),
        )
        .init();

    if let Err(e) = run().await {
        output::error(&format!("{:#}", e));
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();

    // 设置 verbose 日志
    if cli.verbose {
        tracing::subscriber::set_global_default(
            tracing_subscriber::fmt()
                .with_env_filter("cfai=debug")
                .finish(),
        )
        .ok();
    }

    // Config 命令不需要认证
    if let Commands::Config(config_args) = &cli.command {
        return config_args.execute().await;
    }

    // 加载配置
    let config = AppConfig::load()?.merge_env();

    // AI 命令可能不需要 Cloudflare 认证 (如纯问答)
    let needs_cf_client = !matches!(&cli.command, Commands::Ai(ai_args) if matches!(&ai_args.command, cli::commands::ai::AiCommands::Ask { .. }));

    if needs_cf_client {
        if let Err(e) = config.validate() {
            eprintln!("{}", e);
            eprintln!(
                "\n{} 运行 {} 进行配置",
                "提示:".yellow(),
                "cfai config setup".cyan()
            );
            std::process::exit(1);
        }
    }

    // 创建 Cloudflare 客户端
    let client = create_client(&config)?;
    let format = &cli.format;

    match &cli.command {
        Commands::Zone(args) => args.execute(&client, format).await,
        Commands::Dns(args) => args.execute(&client, format).await,
        Commands::Ssl(args) => args.execute(&client, format).await,
        Commands::Firewall(args) => args.execute(&client, format).await,
        Commands::Cache(args) => args.execute(&client, format).await,
        Commands::PageRules(args) => args.execute(&client, format).await,
        Commands::Workers(args) => args.execute(&client, &config, format).await,
        Commands::Analytics(args) => args.execute(&client, format).await,
        Commands::Ai(args) => args.execute(&client, &config, format).await,
        Commands::Config(_) => unreachable!(), // 已在上面处理
    }
}

/// 创建 Cloudflare API 客户端
fn create_client(config: &AppConfig) -> Result<CfClient> {
    let auth = if let Some(token) = &config.cloudflare.api_token {
        AuthMethod::ApiToken(token.clone())
    } else if let (Some(email), Some(key)) = (&config.cloudflare.email, &config.cloudflare.api_key)
    {
        AuthMethod::ApiKey {
            email: email.clone(),
            key: key.clone(),
        }
    } else {
        // 返回一个空 token 的客户端，某些命令可能不需要
        AuthMethod::ApiToken(String::new())
    };

    CfClient::new(auth)
}
