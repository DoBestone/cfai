mod ai;
mod api;
mod cli;
mod config;
#[cfg(feature = "gui")]
mod gui;
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

    // 如果没有提供命令，自动进入交互模式
    let command = match cli.command {
        Some(cmd) => cmd,
        None => {
            // 直接进入交互模式
            output::print_banner();
            println!("💡 提示：直接运行 {} 进入交互模式\n", "cfai".cyan());

            let interactive_args = cli::commands::interactive::InteractiveArgs { once: false };
            return interactive_args.execute(&cli.format, cli.verbose).await;
        }
    };

    // Config / 安装 / 更新 / 交互 命令不需要认证
    match &command {
        Commands::Config(config_args) => return config_args.execute().await,
        Commands::Install(args) => return args.execute().await,
        Commands::Update(args) => return args.execute().await,
        Commands::Interactive(args) => {
            return args.execute(&cli.format, cli.verbose).await
        }
        #[cfg(feature = "gui")]
        Commands::Gui => {
            return crate::gui::launch_gui();
        }
        _ => {}
    }

    // 加载配置并检查是否需要初始化
    let config = ensure_config_exists().await?;

    // AI 命令可能不需要 Cloudflare 认证 (如纯问答)
    let needs_cf_client = !matches!(&command, Commands::Ai(ai_args) if matches!(&ai_args.command, cli::commands::ai::AiCommands::Ask { .. }));

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

    match &command {
        Commands::Zone(args) => args.execute(&client, format).await,
        Commands::Dns(args) => args.execute(&client, format).await,
        Commands::Ssl(args) => args.execute(&client, format).await,
        Commands::Firewall(args) => args.execute(&client, format).await,
        Commands::Cache(args) => args.execute(&client, format).await,
        Commands::PageRules(args) => args.execute(&client, format).await,
        Commands::Workers(args) => args.execute(&client, &config, format).await,
        Commands::Analytics(args) => args.execute(&client, format).await,
        Commands::Ai(args) => args.execute(&client, &config, format).await,
        Commands::Config(_) | Commands::Install(_) | Commands::Update(_) | Commands::Interactive(_) => {
            unreachable!()
        }
        #[cfg(feature = "gui")]
        Commands::Gui => {
            unreachable!()
        }
    }
}

/// 确保配置文件存在，如果不存在则引导用户创建
async fn ensure_config_exists() -> Result<AppConfig> {
    use dialoguer::Confirm;

    let config = AppConfig::load()?.merge_env();

    // 检查是否已配置 Cloudflare 认证
    let has_cf_token = config.cloudflare.api_token.is_some();
    let has_cf_key = config.cloudflare.email.is_some() && config.cloudflare.api_key.is_some();

    if !has_cf_token && !has_cf_key {
        output::title("🎉 欢迎使用 CFAI");
        println!("\n检测到您是第一次使用 CFAI，需要进行初始配置。");
        println!("CFAI 是一个 AI 驱动的 Cloudflare 管理工具，可以帮助您：");
        println!("  • 管理域名、DNS、SSL/TLS");
        println!("  • 配置防火墙和缓存策略");
        println!("  • 使用 AI 进行智能分析和优化");
        println!();

        let should_setup = Confirm::new()
            .with_prompt("是否现在进行配置？")
            .default(true)
            .interact()?;

        if should_setup {
            return AppConfig::interactive_setup();
        } else {
            output::info("您可以稍后运行 'cfai config setup' 进行配置");
            std::process::exit(0);
        }
    }

    Ok(config)
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
