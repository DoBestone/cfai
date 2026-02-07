pub mod zone;
pub mod dns;
pub mod ssl;
pub mod firewall;
pub mod cache;
pub mod page_rules;
pub mod workers;
pub mod analytics;
pub mod ai;
pub mod config;
pub mod install;
pub mod interactive;
pub mod self_update;
pub mod update;

use clap::{Parser, Subcommand};

/// CFAI - AI 驱动的 Cloudflare 管理工具
#[derive(Parser, Debug)]
#[command(
    name = "cfai",
    version,
    about = "🚀 AI 驱动的 Cloudflare 域名管理工具",
    long_about = "CFAI 是一个强大的 CLI 工具，集成 AI 智能分析，\n帮助你高效管理 Cloudflare 域名、DNS、SSL、防火墙等所有功能。",
    after_help = "使用示例:\n  cfai zone list                    # 列出所有域名\n  cfai dns list example.com          # 列出 DNS 记录\n  cfai ai ask \"如何优化我的域名\"     # AI 智能问答\n  cfai ai analyze example.com        # AI 全面分析"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// 输出格式 (table/json/plain)
    #[arg(long, global = true, default_value = "table")]
    pub format: String,

    /// 启用详细输出
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// 域名 (Zone) 管理
    #[command(alias = "z")]
    Zone(zone::ZoneArgs),

    /// DNS 记录管理
    #[command(alias = "d")]
    Dns(dns::DnsArgs),

    /// SSL/TLS 证书管理
    Ssl(ssl::SslArgs),

    /// 防火墙和安全管理
    #[command(alias = "fw")]
    Firewall(firewall::FirewallArgs),

    /// 缓存管理
    Cache(cache::CacheArgs),

    /// 页面规则管理
    #[command(alias = "pr")]
    PageRules(page_rules::PageRulesArgs),

    /// Workers 管理
    #[command(alias = "w")]
    Workers(workers::WorkersArgs),

    /// 流量分析
    #[command(alias = "stats")]
    Analytics(analytics::AnalyticsArgs),

    /// AI 智能助手
    Ai(ai::AiArgs),

    /// 配置管理
    Config(config::ConfigArgs),

    /// 安装 CFAI (下载 Release 二进制)
    Install(install::InstallArgs),

    /// 更新 CFAI (下载 Release 二进制)
    Update(update::UpdateArgs),

    /// 交互模式
    Interactive(interactive::InteractiveArgs),
}
