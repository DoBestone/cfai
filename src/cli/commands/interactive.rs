use anyhow::{anyhow, Result};
use clap::Args;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use std::process::Command;

use crate::cli::output;

#[derive(Args, Debug)]
pub struct InteractiveArgs {
    /// 只执行一次操作后退出
    #[arg(long)]
    pub once: bool,
}

impl InteractiveArgs {
    pub async fn execute(&self, format: &str, verbose: bool) -> Result<()> {
        let theme = ColorfulTheme::default();

        loop {
            output::title_box("🚀 CFAI 交互式菜单");
            println!();

            let items = vec![
                "1️⃣  域名管理 (Zone)",
                "2️⃣  DNS 管理",
                "3️⃣  SSL/TLS 管理",
                "4️⃣  防火墙管理",
                "5️⃣  缓存管理",
                "6️⃣  页面规则",
                "7️⃣  Workers 管理",
                "8️⃣  流量分析",
                "9️⃣  AI 智能助手 🤖",
                "🔧 配置管理",
                "📥 安装 CFAI",
                "🔄 更新 CFAI",
                "⌨️  自定义命令",
                "❌ 退出",
            ];

            let selection = Select::with_theme(&theme)
                .with_prompt("请选择功能")
                .items(&items)
                .default(0)
                .interact()?;

            let args = match selection {
                0 => build_zone_args(&theme)?,
                1 => build_dns_args(&theme)?,
                2 => build_ssl_args(&theme)?,
                3 => build_firewall_args(&theme)?,
                4 => build_cache_args(&theme)?,
                5 => build_page_rules_args(&theme)?,
                6 => build_workers_args(&theme)?,
                7 => build_analytics_args(&theme)?,
                8 => build_ai_args(&theme)?,
                9 => build_config_args(&theme)?,
                10 => Some(vec!["install".to_string()]),
                11 => Some(vec!["update".to_string()]),
                12 => build_custom_args(&theme)?,
                _ => {
                    output::success("感谢使用 CFAI！");
                    break;
                }
            };

            if let Some(mut args) = args {
                if !format.is_empty() && format != "table" {
                    args.push("--format".to_string());
                    args.push(format.to_string());
                }
                if verbose {
                    args.push("--verbose".to_string());
                }

                println!();
                output::separator();
                match run_cfai(args) {
                    Ok(_) => {}
                    Err(e) => {
                        if e.to_string() != "用户取消操作" {
                            output::error(&format!("{}", e));
                        }
                    }
                }
                output::separator();
                println!();
            }

            if self.once {
                break;
            }

            let cont = Confirm::with_theme(&theme)
                .with_prompt("是否继续其它操作?")
                .default(true)
                .interact()?;
            if !cont {
                output::success("感谢使用 CFAI！");
                break;
            }

            println!("\n");
        }

        Ok(())
    }
}

fn run_cfai(args: Vec<String>) -> Result<()> {
    let exe = std::env::current_exe().map_err(|e| anyhow!("获取可执行文件失败: {}", e))?;
    let status = Command::new(exe).args(&args).status()?;
    if !status.success() {
        return Err(anyhow!("命令执行失败"));
    }
    Ok(())
}

fn build_zone_args(theme: &ColorfulTheme) -> Result<Option<Vec<String>>> {
    output::step(1, "域名管理");

    let items = vec![
        "📋 列出所有域名",
        "🔍 查看域名详情",
        "➕ 添加域名",
        "⏸️  暂停域名",
        "▶️  恢复域名",
        "⚙️  域名设置",
        "⬅️  返回上级菜单",
    ];
    let selection = Select::with_theme(theme)
        .with_prompt("选择操作")
        .items(&items)
        .default(0)
        .interact()?;

    match selection {
        0 => Ok(Some(vec!["zone".into(), "list".into()])),
        1 => {
            let domain = prompt_domain(theme)?;
            Ok(Some(vec!["zone".into(), "get".into(), domain]))
        }
        2 => {
            let domain = prompt_domain(theme)?;
            Ok(Some(vec!["zone".into(), "add".into(), domain]))
        }
        3 => {
            let domain = prompt_domain(theme)?;
            Ok(Some(vec!["zone".into(), "pause".into(), domain]))
        }
        4 => {
            let domain = prompt_domain(theme)?;
            Ok(Some(vec!["zone".into(), "resume".into(), domain]))
        }
        5 => {
            let domain = prompt_domain(theme)?;
            Ok(Some(vec!["zone".into(), "settings".into(), domain]))
        }
        _ => Ok(None),
    }
}

fn build_dns_args(theme: &ColorfulTheme) -> Result<Option<Vec<String>>> {
    output::step(2, "DNS 管理");

    let items = vec![
        "📋 列出 DNS 记录",
        "➕ 添加 A 记录",
        "➕ 添加 AAAA 记录",
        "➕ 添加 CNAME 记录",
        "➕ 添加 MX 记录",
        "➕ 添加 TXT 记录",
        "🗑️  删除记录",
        "🔍 搜索记录",
        "⬅️  返回上级菜单",
    ];
    let selection = Select::with_theme(theme)
        .with_prompt("选择操作")
        .items(&items)
        .default(0)
        .interact()?;

    match selection {
        0 => {
            let domain = prompt_domain(theme)?;
            let record_type: String = Input::with_theme(theme)
                .with_prompt("记录类型 (可选, 如 A/AAAA/CNAME，留空显示全部)")
                .allow_empty(true)
                .interact_text()?;
            let mut args = vec!["dns".into(), "list".into(), domain];
            if !record_type.trim().is_empty() {
                args.push("-t".into());
                args.push(record_type.trim().to_uppercase());
            }
            Ok(Some(args))
        }
        1 => Ok(Some(vec![
            "dns".into(),
            "add-a".into(),
            prompt_domain(theme)?,
            prompt_text(theme, "主机名 (如 www, 或 @ 表示根域名)")?,
            prompt_text(theme, "IPv4 地址")?,
        ])),
        2 => Ok(Some(vec![
            "dns".into(),
            "add".into(),
            prompt_domain(theme)?,
            "-t".into(),
            "AAAA".into(),
            "-n".into(),
            prompt_text(theme, "主机名")?,
            "-c".into(),
            prompt_text(theme, "IPv6 地址")?,
        ])),
        3 => Ok(Some(vec![
            "dns".into(),
            "add-cname".into(),
            prompt_domain(theme)?,
            prompt_text(theme, "主机名 (如 blog)")?,
            prompt_text(theme, "目标域名")?,
        ])),
        4 => Ok(Some(vec![
            "dns".into(),
            "add".into(),
            prompt_domain(theme)?,
            "-t".into(),
            "MX".into(),
            "-n".into(),
            prompt_text(theme, "主机名")?,
            "-c".into(),
            prompt_text(theme, "邮件服务器")?,
        ])),
        5 => Ok(Some(vec![
            "dns".into(),
            "add".into(),
            prompt_domain(theme)?,
            "-t".into(),
            "TXT".into(),
            "-n".into(),
            prompt_text(theme, "主机名")?,
            "-c".into(),
            prompt_text(theme, "文本内容")?,
        ])),
        6 => Ok(Some(vec![
            "dns".into(),
            "delete".into(),
            prompt_domain(theme)?,
            prompt_text(theme, "记录 ID")?,
        ])),
        7 => Ok(Some(vec![
            "dns".into(),
            "find".into(),
            prompt_domain(theme)?,
            prompt_text(theme, "搜索关键词")?,
        ])),
        _ => Ok(None),
    }
}

fn build_ssl_args(theme: &ColorfulTheme) -> Result<Option<Vec<String>>> {
    output::step(3, "SSL/TLS 管理");

    let items = vec![
        "🔍 查看 SSL 状态",
        "⚙️  设置 SSL 模式",
        "🔒 开启 Always HTTPS",
        "🔓 关闭 Always HTTPS",
        "📜 列出证书",
        "⬅️  返回上级菜单",
    ];
    let selection = Select::with_theme(theme)
        .with_prompt("选择操作")
        .items(&items)
        .default(0)
        .interact()?;

    match selection {
        0 => Ok(Some(vec![
            "ssl".into(),
            "status".into(),
            prompt_domain(theme)?,
        ])),
        1 => {
            let domain = prompt_domain(theme)?;
            let modes = vec!["off (关闭)", "flexible (灵活)", "full (完全)", "strict (严格)"];
            let mode_sel = Select::with_theme(theme)
                .with_prompt("选择 SSL 模式")
                .items(&modes)
                .default(3)
                .interact()?;
            let mode = match mode_sel {
                0 => "off",
                1 => "flexible",
                2 => "full",
                _ => "strict",
            };
            Ok(Some(vec!["ssl".into(), "mode".into(), domain, mode.into()]))
        }
        2 => Ok(Some(vec![
            "ssl".into(),
            "https".into(),
            prompt_domain(theme)?,
            "on".into(),
        ])),
        3 => Ok(Some(vec![
            "ssl".into(),
            "https".into(),
            prompt_domain(theme)?,
            "off".into(),
        ])),
        4 => Ok(Some(vec![
            "ssl".into(),
            "list".into(),
            prompt_domain(theme)?,
        ])),
        _ => Ok(None),
    }
}

fn build_firewall_args(theme: &ColorfulTheme) -> Result<Option<Vec<String>>> {
    output::step(4, "防火墙管理");

    let items = vec![
        "🛡️  安全概览",
        "📋 列出防火墙规则",
        "🚫 封禁 IP 地址",
        "✅ IP 白名单",
        "🗑️  删除 IP 规则",
        "⚠️  开启 Under Attack 模式",
        "✅ 关闭 Under Attack 模式",
        "⬅️  返回上级菜单",
    ];
    let selection = Select::with_theme(theme)
        .with_prompt("选择操作")
        .items(&items)
        .default(0)
        .interact()?;

    match selection {
        0 => Ok(Some(vec![
            "firewall".into(),
            "status".into(),
            prompt_domain(theme)?,
        ])),
        1 => Ok(Some(vec![
            "firewall".into(),
            "list".into(),
            prompt_domain(theme)?,
        ])),
        2 => Ok(Some(vec![
            "firewall".into(),
            "block".into(),
            prompt_domain(theme)?,
            prompt_text(theme, "IP 地址")?,
        ])),
        3 => Ok(Some(vec![
            "firewall".into(),
            "whitelist".into(),
            prompt_domain(theme)?,
            prompt_text(theme, "IP 地址")?,
        ])),
        4 => Ok(Some(vec![
            "firewall".into(),
            "unblock".into(),
            prompt_domain(theme)?,
            prompt_text(theme, "规则 ID")?,
        ])),
        5 => Ok(Some(vec![
            "firewall".into(),
            "ua-on".into(),
            prompt_domain(theme)?,
        ])),
        6 => Ok(Some(vec![
            "firewall".into(),
            "ua-off".into(),
            prompt_domain(theme)?,
        ])),
        _ => Ok(None),
    }
}

fn build_cache_args(theme: &ColorfulTheme) -> Result<Option<Vec<String>>> {
    output::step(5, "缓存管理");

    let items = vec![
        "🔍 查看缓存状态",
        "🗑️  清除全部缓存",
        "🎯 按 URL 清除缓存",
        "⚙️  设置缓存级别",
        "⏰ 设置浏览器缓存 TTL",
        "🔧 开启开发模式",
        "⬅️  返回上级菜单",
    ];
    let selection = Select::with_theme(theme)
        .with_prompt("选择操作")
        .items(&items)
        .default(0)
        .interact()?;

    match selection {
        0 => Ok(Some(vec![
            "cache".into(),
            "status".into(),
            prompt_domain(theme)?,
        ])),
        1 => {
            let domain = prompt_domain(theme)?;
            let confirm = Confirm::with_theme(theme)
                .with_prompt("确认清除全部缓存？这将影响所有访问者")
                .default(false)
                .interact()?;
            if confirm {
                Ok(Some(vec!["cache".into(), "purge-all".into(), domain]))
            } else {
                output::info("已取消操作");
                Ok(None)
            }
        }
        2 => Ok(Some(vec![
            "cache".into(),
            "purge-url".into(),
            prompt_domain(theme)?,
            prompt_text(theme, "URL 地址")?,
        ])),
        3 => {
            let domain = prompt_domain(theme)?;
            let levels = vec!["basic (基础)", "simplified (简化)", "aggressive (激进)"];
            let level_sel = Select::with_theme(theme)
                .with_prompt("选择缓存级别")
                .items(&levels)
                .default(0)
                .interact()?;
            let level = match level_sel {
                0 => "basic",
                1 => "simplified",
                _ => "aggressive",
            };
            Ok(Some(vec!["cache".into(), "level".into(), domain, level.into()]))
        }
        4 => Ok(Some(vec![
            "cache".into(),
            "browser-ttl".into(),
            prompt_domain(theme)?,
            prompt_text(theme, "TTL 秒数")?,
        ])),
        5 => Ok(Some(vec![
            "cache".into(),
            "dev-mode".into(),
            prompt_domain(theme)?,
            "on".into(),
        ])),
        _ => Ok(None),
    }
}

fn build_page_rules_args(theme: &ColorfulTheme) -> Result<Option<Vec<String>>> {
    output::step(6, "页面规则");

    let items = vec![
        "📋 列出页面规则",
        "🔍 查看规则详情",
        "🗑️  删除规则",
        "⬅️  返回上级菜单",
    ];
    let selection = Select::with_theme(theme)
        .with_prompt("选择操作")
        .items(&items)
        .default(0)
        .interact()?;

    match selection {
        0 => Ok(Some(vec![
            "page-rules".into(),
            "list".into(),
            prompt_domain(theme)?,
        ])),
        1 => Ok(Some(vec![
            "page-rules".into(),
            "get".into(),
            prompt_domain(theme)?,
            prompt_text(theme, "规则 ID")?,
        ])),
        2 => Ok(Some(vec![
            "page-rules".into(),
            "delete".into(),
            prompt_domain(theme)?,
            prompt_text(theme, "规则 ID")?,
        ])),
        _ => Ok(None),
    }
}

fn build_workers_args(theme: &ColorfulTheme) -> Result<Option<Vec<String>>> {
    output::step(7, "Workers 管理");

    let items = vec![
        "📋 列出 Workers 脚本",
        "🗑️  删除脚本",
        "🔗 列出路由",
        "📦 列出 KV 命名空间",
        "⬅️  返回上级菜单",
    ];
    let selection = Select::with_theme(theme)
        .with_prompt("选择操作")
        .items(&items)
        .default(0)
        .interact()?;

    match selection {
        0 => Ok(Some(vec!["workers".into(), "list".into()])),
        1 => Ok(Some(vec![
            "workers".into(),
            "delete".into(),
            prompt_text(theme, "脚本名称")?,
        ])),
        2 => Ok(Some(vec![
            "workers".into(),
            "routes".into(),
            prompt_domain(theme)?,
        ])),
        3 => Ok(Some(vec!["workers".into(), "kv".into()])),
        _ => Ok(None),
    }
}

fn build_analytics_args(theme: &ColorfulTheme) -> Result<Option<Vec<String>>> {
    output::step(8, "流量分析");

    let items = vec![
        "📊 24小时流量概览",
        "📈 详细流量分析",
        "⬅️  返回上级菜单",
    ];
    let selection = Select::with_theme(theme)
        .with_prompt("选择操作")
        .items(&items)
        .default(0)
        .interact()?;

    match selection {
        0 => Ok(Some(vec![
            "analytics".into(),
            "overview".into(),
            prompt_domain(theme)?,
        ])),
        1 => Ok(Some(vec![
            "analytics".into(),
            "detail".into(),
            prompt_domain(theme)?,
        ])),
        _ => Ok(None),
    }
}

fn build_ai_args(theme: &ColorfulTheme) -> Result<Option<Vec<String>>> {
    output::step(9, "AI 智能助手 🤖");

    let items = vec![
        "💬 AI 自由问答",
        "🔍 AI 全面分析域名",
        "🔒 AI 安全分析",
        "⚡ AI 性能分析",
        "📡 AI DNS 分析",
        "🔧 AI 故障诊断",
        "⬅️  返回上级菜单",
    ];
    let selection = Select::with_theme(theme)
        .with_prompt("选择操作")
        .items(&items)
        .default(0)
        .interact()?;

    match selection {
        0 => Ok(Some(vec!["ai".into(), "ask".into(), prompt_text(theme, "请输入您的问题")?])),
        1 => Ok(Some(vec![
            "ai".into(),
            "analyze".into(),
            prompt_domain(theme)?,
        ])),
        2 => Ok(Some(vec![
            "ai".into(),
            "analyze".into(),
            prompt_domain(theme)?,
            "-t".into(),
            "security".into(),
        ])),
        3 => Ok(Some(vec![
            "ai".into(),
            "analyze".into(),
            prompt_domain(theme)?,
            "-t".into(),
            "performance".into(),
        ])),
        4 => Ok(Some(vec![
            "ai".into(),
            "analyze".into(),
            prompt_domain(theme)?,
            "-t".into(),
            "dns".into(),
        ])),
        5 => Ok(Some(vec![
            "ai".into(),
            "troubleshoot".into(),
            prompt_text(theme, "问题描述")?,
            "-d".into(),
            prompt_domain(theme)?,
        ])),
        _ => Ok(None),
    }
}

fn build_config_args(theme: &ColorfulTheme) -> Result<Option<Vec<String>>> {
    output::step(10, "配置管理");

    let items = vec![
        "✏️  编辑配置 (推荐)",
        "⚙️  配置向导 (完整设置)",
        "👀 查看配置",
        "🔑 查看配置（显示密钥）",
        "✅ 验证配置",
        "📂 配置文件路径",
        "⬅️  返回上级菜单",
    ];
    let selection = Select::with_theme(theme)
        .with_prompt("选择操作")
        .items(&items)
        .default(0)
        .interact()?;

    match selection {
        0 => Ok(Some(vec!["config".into(), "edit".into()])),
        1 => Ok(Some(vec!["config".into(), "setup".into()])),
        2 => Ok(Some(vec!["config".into(), "show".into()])),
        3 => Ok(Some(vec!["config".into(), "show".into(), "--show-secrets".into()])),
        4 => Ok(Some(vec!["config".into(), "verify".into()])),
        5 => Ok(Some(vec!["config".into(), "path".into()])),
        _ => Ok(None),
    }
}

fn build_custom_args(theme: &ColorfulTheme) -> Result<Option<Vec<String>>> {
    output::step(11, "自定义命令");
    output::info("您可以输入任何 cfai 命令（不含 'cfai' 本身）");
    output::tip("示例: zone list, dns list example.com, ai ask \"问题\"");

    let input: String = Input::with_theme(theme)
        .with_prompt("输入命令")
        .allow_empty(true)
        .interact_text()?;

    if input.trim().is_empty() {
        return Ok(None);
    }

    let args = shell_words::split(&input).map_err(|e| anyhow!("解析参数失败: {}", e))?;
    Ok(Some(args))
}

fn prompt_domain(theme: &ColorfulTheme) -> Result<String> {
    let items = vec![
        "📋 从域名列表中选择",
        "✍️  手动输入域名",
        "⬅️  返回上级菜单",
    ];
    let selection = Select::with_theme(theme)
        .with_prompt("选择域名输入方式")
        .items(&items)
        .default(0)
        .interact()?;

    match selection {
        0 => {
            // 从域名列表选择
            output::loading("正在获取域名列表...");
            let exe = std::env::current_exe().map_err(|e| anyhow!("获取可执行文件失败: {}", e))?;
            let output = Command::new(exe)
                .args(&["zone", "list", "--format", "json"])
                .output()?;

            if !output.status.success() {
                output::warn("获取域名列表失败，请手动输入");
                return prompt_text(theme, "域名 (如: example.com)");
            }

            let stdout = String::from_utf8_lossy(&output.stdout);

            // 解析 JSON 获取域名列表
            let domains: Vec<String> = match serde_json::from_str::<serde_json::Value>(&stdout) {
                Ok(json) => {
                    if let Some(arr) = json.as_array() {
                        arr.iter()
                            .filter_map(|v| v.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()))
                            .collect()
                    } else {
                        vec![]
                    }
                }
                Err(_) => vec![],
            };

            if domains.is_empty() {
                output::warn("未找到域名，请手动输入");
                return prompt_text(theme, "域名 (如: example.com)");
            }

            let mut domain_items: Vec<&str> = domains.iter().map(|s| s.as_str()).collect();
            domain_items.push("⬅️  返回");

            let domain_sel = Select::with_theme(theme)
                .with_prompt("选择域名")
                .items(&domain_items)
                .default(0)
                .interact()?;

            if domain_sel == domain_items.len() - 1 {
                return Err(anyhow!("用户取消操作"));
            }

            Ok(domains[domain_sel].clone())
        }
        1 => {
            // 手动输入
            prompt_text(theme, "域名 (如: example.com)")
        }
        _ => {
            // 返回上级菜单
            Err(anyhow!("用户取消操作"))
        }
    }
}

fn prompt_text(theme: &ColorfulTheme, prompt: &str) -> Result<String> {
    Ok(Input::with_theme(theme).with_prompt(prompt).interact_text()?)
}
