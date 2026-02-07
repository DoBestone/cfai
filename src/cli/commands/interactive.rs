use anyhow::{anyhow, Result};
use clap::Args;
use dialoguer::{Confirm, Input, Select};
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
        loop {
            output::title("CFAI 交互模式");

            let items = vec![
                "域名管理 (zone)",
                "DNS 管理 (dns)",
                "SSL/TLS 管理 (ssl)",
                "防火墙管理 (firewall)",
                "缓存管理 (cache)",
                "页面规则 (page-rules)",
                "Workers 管理 (workers)",
                "流量分析 (analytics)",
                "AI 智能助手 (ai)",
                "配置管理 (config)",
                "安装 CFAI (install)",
                "更新 CFAI (update)",
                "自定义命令",
                "退出",
            ];

            let selection = Select::new()
                .items(&items)
                .default(0)
                .interact()?;

            let args = match selection {
                0 => build_zone_args()?,
                1 => build_dns_args()?,
                2 => build_ssl_args()?,
                3 => build_firewall_args()?,
                4 => build_cache_args()?,
                5 => build_page_rules_args()?,
                6 => build_workers_args()?,
                7 => build_analytics_args()?,
                8 => build_ai_args()?,
                9 => build_config_args()?,
                10 => Some(vec!["install".to_string()]),
                11 => Some(vec!["update".to_string()]),
                12 => build_custom_args()?,
                _ => break,
            };

            if let Some(mut args) = args {
                if !format.is_empty() {
                    args.push("--format".to_string());
                    args.push(format.to_string());
                }
                if verbose {
                    args.push("--verbose".to_string());
                }

                run_cfai(args)?;
            }

            if self.once {
                break;
            }

            let cont = Confirm::new()
                .with_prompt("是否继续其它操作?")
                .default(true)
                .interact()?;
            if !cont {
                break;
            }
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

fn build_zone_args() -> Result<Option<Vec<String>>> {
    let items = vec![
        "列出域名", "查看域名详情", "添加域名", "暂停域名", "恢复域名", "返回",
    ];
    let selection = Select::new().items(&items).default(0).interact()?;
    match selection {
        0 => Ok(Some(vec!["zone".into(), "list".into()])),
        1 => Ok(Some(vec![
            "zone".into(),
            "get".into(),
            prompt_domain()?,
        ])),
        2 => Ok(Some(vec![
            "zone".into(),
            "add".into(),
            prompt_domain()?,
        ])),
        3 => Ok(Some(vec![
            "zone".into(),
            "pause".into(),
            prompt_domain()?,
        ])),
        4 => Ok(Some(vec![
            "zone".into(),
            "resume".into(),
            prompt_domain()?,
        ])),
        _ => Ok(None),
    }
}

fn build_dns_args() -> Result<Option<Vec<String>>> {
    let items = vec![
        "列出记录",
        "添加 A 记录",
        "添加 CNAME",
        "删除记录",
        "返回",
    ];
    let selection = Select::new().items(&items).default(0).interact()?;
    match selection {
        0 => {
            let domain = prompt_domain()?;
            let record_type: String = Input::new()
                .with_prompt("记录类型 (可选, 如 A/AAAA/CNAME)")
                .allow_empty(true)
                .interact_text()?;
            let mut args = vec!["dns".into(), "list".into(), domain];
            if !record_type.trim().is_empty() {
                args.push("-t".into());
                args.push(record_type.trim().to_string());
            }
            Ok(Some(args))
        }
        1 => Ok(Some(vec![
            "dns".into(),
            "add-a".into(),
            prompt_domain()?,
            prompt_text("主机名 (如 www)")?,
            prompt_text("IP 地址")?,
        ])),
        2 => Ok(Some(vec![
            "dns".into(),
            "add-cname".into(),
            prompt_domain()?,
            prompt_text("主机名 (如 blog)")?,
            prompt_text("目标域名")?,
        ])),
        3 => Ok(Some(vec![
            "dns".into(),
            "delete".into(),
            prompt_domain()?,
            prompt_text("记录 ID")?,
        ])),
        _ => Ok(None),
    }
}

fn build_ssl_args() -> Result<Option<Vec<String>>> {
    let items = vec!["查看 SSL 状态", "设置 SSL 模式", "返回"];
    let selection = Select::new().items(&items).default(0).interact()?;
    match selection {
        0 => Ok(Some(vec![
            "ssl".into(),
            "status".into(),
            prompt_domain()?,
        ])),
        1 => Ok(Some(vec![
            "ssl".into(),
            "mode".into(),
            prompt_domain()?,
            prompt_text("模式 (off/flexible/full/strict)")?,
        ])),
        _ => Ok(None),
    }
}

fn build_firewall_args() -> Result<Option<Vec<String>>> {
    let items = vec!["安全概览", "封禁 IP", "解封 IP", "返回"];
    let selection = Select::new().items(&items).default(0).interact()?;
    match selection {
        0 => Ok(Some(vec![
            "firewall".into(),
            "status".into(),
            prompt_domain()?,
        ])),
        1 => Ok(Some(vec![
            "firewall".into(),
            "block".into(),
            prompt_domain()?,
            prompt_text("IP 地址")?,
        ])),
        2 => Ok(Some(vec![
            "firewall".into(),
            "unblock".into(),
            prompt_domain()?,
            prompt_text("规则 ID")?,
        ])),
        _ => Ok(None),
    }
}

fn build_cache_args() -> Result<Option<Vec<String>>> {
    let items = vec!["查看缓存状态", "清除全部缓存", "返回"];
    let selection = Select::new().items(&items).default(0).interact()?;
    match selection {
        0 => Ok(Some(vec![
            "cache".into(),
            "status".into(),
            prompt_domain()?,
        ])),
        1 => Ok(Some(vec![
            "cache".into(),
            "purge-all".into(),
            prompt_domain()?,
        ])),
        _ => Ok(None),
    }
}

fn build_page_rules_args() -> Result<Option<Vec<String>>> {
    let items = vec!["列出页面规则", "删除规则", "返回"];
    let selection = Select::new().items(&items).default(0).interact()?;
    match selection {
        0 => Ok(Some(vec![
            "page-rules".into(),
            "list".into(),
            prompt_domain()?,
        ])),
        1 => Ok(Some(vec![
            "page-rules".into(),
            "delete".into(),
            prompt_domain()?,
            prompt_text("规则 ID")?,
        ])),
        _ => Ok(None),
    }
}

fn build_workers_args() -> Result<Option<Vec<String>>> {
    let items = vec!["列出 Workers", "删除脚本", "返回"];
    let selection = Select::new().items(&items).default(0).interact()?;
    match selection {
        0 => Ok(Some(vec!["workers".into(), "list".into()])),
        1 => Ok(Some(vec![
            "workers".into(),
            "delete".into(),
            prompt_text("脚本名称")?,
        ])),
        _ => Ok(None),
    }
}

fn build_analytics_args() -> Result<Option<Vec<String>>> {
    let items = vec!["概览", "详细分析", "返回"];
    let selection = Select::new().items(&items).default(0).interact()?;
    match selection {
        0 => Ok(Some(vec![
            "analytics".into(),
            "overview".into(),
            prompt_domain()?,
        ])),
        1 => Ok(Some(vec![
            "analytics".into(),
            "detail".into(),
            prompt_domain()?,
        ])),
        _ => Ok(None),
    }
}

fn build_ai_args() -> Result<Option<Vec<String>>> {
    let items = vec!["AI 问答", "AI 分析域名", "返回"];
    let selection = Select::new().items(&items).default(0).interact()?;
    match selection {
        0 => Ok(Some(vec!["ai".into(), "ask".into(), prompt_text("问题")?])),
        1 => {
            let domain = prompt_domain()?;
            let analyze_type: String = Input::new()
                .with_prompt("分析类型 (可选: dns/security/performance)")
                .allow_empty(true)
                .interact_text()?;
            let mut args = vec!["ai".into(), "analyze".into(), domain];
            if !analyze_type.trim().is_empty() {
                args.push("-t".into());
                args.push(analyze_type.trim().to_string());
            }
            Ok(Some(args))
        }
        _ => Ok(None),
    }
}

fn build_config_args() -> Result<Option<Vec<String>>> {
    let items = vec!["配置向导", "查看配置", "验证配置", "返回"];
    let selection = Select::new().items(&items).default(0).interact()?;
    match selection {
        0 => Ok(Some(vec!["config".into(), "setup".into()])),
        1 => Ok(Some(vec!["config".into(), "show".into()])),
        2 => Ok(Some(vec!["config".into(), "verify".into()])),
        _ => Ok(None),
    }
}

fn build_custom_args() -> Result<Option<Vec<String>>> {
    let input: String = Input::new()
        .with_prompt("输入完整命令参数 (不含 cfai)")
        .allow_empty(true)
        .interact_text()?;
    if input.trim().is_empty() {
        return Ok(None);
    }

    let args = shell_words::split(&input).map_err(|e| anyhow!("解析参数失败: {}", e))?;
    Ok(Some(args))
}

fn prompt_domain() -> Result<String> {
    prompt_text("域名")
}

fn prompt_text(prompt: &str) -> Result<String> {
    Ok(Input::new().with_prompt(prompt).interact_text()?)
}
