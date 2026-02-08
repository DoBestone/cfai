use anyhow::{Context, Result};
use colored::Colorize;
use dialoguer::Confirm;

use crate::ai::analyzer::SuggestedAction;
use crate::api::client::CfClient;
use crate::cli::output;
use crate::models::dns::DnsRecordRequest;

/// 执行 AI 建议的操作列表
pub async fn execute_actions(
    client: &CfClient,
    zone_id: &str,
    actions: &[SuggestedAction],
) -> Result<()> {
    if actions.is_empty() {
        return Ok(());
    }

    println!("\n{}", "🚀 准备执行以下操作:".bold().yellow());
    output::separator();

    for (i, action) in actions.iter().enumerate() {
        let risk_icon = match action.risk.as_str() {
            "low" => "🟢",
            "medium" => "🟡",
            "high" => "🔴",
            _ => "⚪",
        };
        println!(
            "  {}. {} {} [风险: {}]",
            i + 1,
            risk_icon,
            action.description,
            action.risk
        );
    }

    output::separator();

    // 总体确认
    let confirm = Confirm::new()
        .with_prompt("是否执行以上操作?")
        .default(false)
        .interact()?;

    if !confirm {
        println!("{}", "已取消执行".dimmed());
        return Ok(());
    }

    let total = actions.len();
    let mut success_count = 0;
    let mut fail_count = 0;

    for (i, action) in actions.iter().enumerate() {
        println!(
            "\n{} [{}/{}] {}",
            "▶".cyan(),
            i + 1,
            total,
            action.description
        );

        // 高风险操作需要单独确认
        if action.risk == "high" {
            let high_confirm = Confirm::new()
                .with_prompt(format!(
                    "🔴 高风险操作: {}，确认执行?",
                    action.description
                ))
                .default(false)
                .interact()?;

            if !high_confirm {
                println!("  {} 已跳过", "⏭️".dimmed());
                continue;
            }
        }

        match execute_single_action(client, zone_id, action).await {
            Ok(msg) => {
                success_count += 1;
                output::success(&format!("{}", msg));
            }
            Err(e) => {
                fail_count += 1;
                output::error(&format!("执行失败: {}", e));

                if i + 1 < total {
                    let cont = Confirm::new()
                        .with_prompt("是否继续执行剩余操作?")
                        .default(true)
                        .interact()?;
                    if !cont {
                        println!("{}", "已中止剩余操作".dimmed());
                        break;
                    }
                }
            }
        }
    }

    println!();
    output::separator();
    println!(
        "📊 执行完成: {} 成功, {} 失败, {} 总计",
        success_count.to_string().green(),
        fail_count.to_string().red(),
        total.to_string().dimmed()
    );

    Ok(())
}

/// 执行单个操作
async fn execute_single_action(
    client: &CfClient,
    zone_id: &str,
    action: &SuggestedAction,
) -> Result<String> {
    let params = &action.params;

    match action.action_type.as_str() {
        "ssl_set" => execute_ssl_action(client, zone_id, params).await,
        "setting_update" => execute_setting_update(client, zone_id, params).await,
        "dns_create" => execute_dns_create(client, zone_id, params).await,
        "dns_update" => execute_dns_update(client, zone_id, params).await,
        "dns_delete" => execute_dns_delete(client, zone_id, params).await,
        "cache_purge" => execute_cache_purge(client, zone_id, params).await,
        "firewall_rule" => execute_firewall_rule(client, zone_id, params).await,
        other => anyhow::bail!("未知的操作类型: {}", other),
    }
}

// ==================== SSL 操作 ====================

async fn execute_ssl_action(
    client: &CfClient,
    zone_id: &str,
    params: &serde_json::Value,
) -> Result<String> {
    let setting = params["setting"]
        .as_str()
        .context("ssl_set 缺少 setting 参数")?;

    match setting {
        "ssl_mode" => {
            let value = params["value"].as_str().context("缺少 value 参数")?;
            client.set_ssl_mode(zone_id, value).await?;
            Ok(format!("SSL 模式已设置为: {}", value))
        }
        "always_https" => {
            let enable = params_to_bool(params, "enable")?;
            client.set_always_https(zone_id, enable).await?;
            Ok(format!("Always HTTPS 已{}", if enable { "开启" } else { "关闭" }))
        }
        "min_tls_version" => {
            let value = params["value"].as_str().context("缺少 value 参数")?;
            client.set_ssl_min_tls(zone_id, value).await?;
            Ok(format!("最小 TLS 版本已设置为: {}", value))
        }
        "opportunistic_encryption" => {
            let enable = params_to_bool(params, "enable")?;
            client.set_opportunistic_encryption(zone_id, enable).await?;
            Ok(format!(
                "Opportunistic Encryption 已{}",
                if enable { "开启" } else { "关闭" }
            ))
        }
        "automatic_https_rewrites" => {
            let enable = params_to_bool(params, "enable")?;
            client
                .set_automatic_https_rewrites(zone_id, enable)
                .await?;
            Ok(format!(
                "Automatic HTTPS Rewrites 已{}",
                if enable { "开启" } else { "关闭" }
            ))
        }
        _ => anyhow::bail!("未知的 SSL 设置: {}", setting),
    }
}

// ==================== Zone 设置更新 ====================

async fn execute_setting_update(
    client: &CfClient,
    zone_id: &str,
    params: &serde_json::Value,
) -> Result<String> {
    let setting_id = params["setting_id"]
        .as_str()
        .context("setting_update 缺少 setting_id 参数")?;
    let value = params
        .get("value")
        .context("setting_update 缺少 value 参数")?
        .clone();

    client
        .update_zone_setting(zone_id, setting_id, value.clone())
        .await?;
    Ok(format!("设置 {} 已更新为: {}", setting_id, value))
}

// ==================== DNS 操作 ====================

async fn execute_dns_create(
    client: &CfClient,
    zone_id: &str,
    params: &serde_json::Value,
) -> Result<String> {
    let record_type = params["type"]
        .as_str()
        .context("dns_create 缺少 type 参数")?;
    let name = params["name"]
        .as_str()
        .context("dns_create 缺少 name 参数")?;
    let content = params["content"]
        .as_str()
        .context("dns_create 缺少 content 参数")?;

    let request = DnsRecordRequest {
        record_type: record_type.to_string(),
        name: name.to_string(),
        content: content.to_string(),
        ttl: params["ttl"].as_u64().map(|v| v as u32),
        proxied: params["proxied"].as_bool(),
        priority: params["priority"].as_u64().map(|v| v as u16),
        comment: params["comment"].as_str().map(|s| s.to_string()),
        tags: None,
    };

    let record = client.create_dns_record(zone_id, &request).await?;
    Ok(format!(
        "DNS 记录已创建: {} {} → {} (ID: {})",
        record_type,
        name,
        content,
        record.id.unwrap_or_default()
    ))
}

async fn execute_dns_update(
    client: &CfClient,
    zone_id: &str,
    params: &serde_json::Value,
) -> Result<String> {
    let record_id = params["record_id"]
        .as_str()
        .context("dns_update 缺少 record_id 参数")?;
    let record_type = params["type"]
        .as_str()
        .context("dns_update 缺少 type 参数")?;
    let name = params["name"]
        .as_str()
        .context("dns_update 缺少 name 参数")?;
    let content = params["content"]
        .as_str()
        .context("dns_update 缺少 content 参数")?;

    let request = DnsRecordRequest {
        record_type: record_type.to_string(),
        name: name.to_string(),
        content: content.to_string(),
        ttl: params["ttl"].as_u64().map(|v| v as u32),
        proxied: params["proxied"].as_bool(),
        priority: params["priority"].as_u64().map(|v| v as u16),
        comment: params["comment"].as_str().map(|s| s.to_string()),
        tags: None,
    };

    client
        .update_dns_record(zone_id, record_id, &request)
        .await?;
    Ok(format!(
        "DNS 记录已更新: {} {} → {}",
        record_type, name, content
    ))
}

async fn execute_dns_delete(
    client: &CfClient,
    zone_id: &str,
    params: &serde_json::Value,
) -> Result<String> {
    let record_id = params["record_id"]
        .as_str()
        .context("dns_delete 缺少 record_id 参数")?;

    client.delete_dns_record(zone_id, record_id).await?;
    Ok(format!("DNS 记录已删除: {}", record_id))
}

// ==================== 缓存操作 ====================

async fn execute_cache_purge(
    client: &CfClient,
    zone_id: &str,
    params: &serde_json::Value,
) -> Result<String> {
    let purge_type = params["type"]
        .as_str()
        .unwrap_or("purge_all");

    match purge_type {
        "purge_all" => {
            client.purge_all_cache(zone_id).await?;
            Ok("已清除全部缓存".to_string())
        }
        "purge_urls" => {
            let urls = json_array_to_strings(&params["urls"])
                .context("cache_purge purge_urls 缺少 urls 参数")?;
            client.purge_cache_by_urls(zone_id, urls.clone()).await?;
            Ok(format!("已清除 {} 个 URL 的缓存", urls.len()))
        }
        "purge_tags" => {
            let tags = json_array_to_strings(&params["tags"])
                .context("cache_purge purge_tags 缺少 tags 参数")?;
            client.purge_cache_by_tags(zone_id, tags.clone()).await?;
            Ok(format!("已清除 {} 个 Tag 的缓存", tags.len()))
        }
        "purge_hosts" => {
            let hosts = json_array_to_strings(&params["hosts"])
                .context("cache_purge purge_hosts 缺少 hosts 参数")?;
            client.purge_cache_by_hosts(zone_id, hosts.clone()).await?;
            Ok(format!("已清除 {} 个主机名的缓存", hosts.len()))
        }
        _ => anyhow::bail!("未知的缓存清除类型: {}", purge_type),
    }
}

// ==================== 防火墙操作 ====================

async fn execute_firewall_rule(
    client: &CfClient,
    zone_id: &str,
    params: &serde_json::Value,
) -> Result<String> {
    let rule_type = params["type"]
        .as_str()
        .context("firewall_rule 缺少 type 参数")?;

    match rule_type {
        "block_ip" => {
            let ip = params["ip"]
                .as_str()
                .context("block_ip 缺少 ip 参数")?;
            let note = params["note"].as_str();
            client.block_ip(zone_id, ip, note).await?;
            Ok(format!("已封禁 IP: {}", ip))
        }
        "whitelist_ip" => {
            let ip = params["ip"]
                .as_str()
                .context("whitelist_ip 缺少 ip 参数")?;
            let note = params["note"].as_str();
            client.whitelist_ip(zone_id, ip, note).await?;
            Ok(format!("已添加 IP 白名单: {}", ip))
        }
        "security_level" => {
            let level = params["level"]
                .as_str()
                .context("security_level 缺少 level 参数")?;
            client.set_security_level(zone_id, level).await?;
            Ok(format!("安全级别已设置为: {}", level))
        }
        "under_attack" => {
            let enable = params_to_bool(params, "enable")?;
            client.set_under_attack_mode(zone_id, enable).await?;
            Ok(format!(
                "Under Attack 模式已{}",
                if enable { "开启" } else { "关闭" }
            ))
        }
        "browser_check" => {
            let enable = params_to_bool(params, "enable")?;
            client.set_browser_check(zone_id, enable).await?;
            Ok(format!(
                "浏览器完整性检查已{}",
                if enable { "开启" } else { "关闭" }
            ))
        }
        _ => anyhow::bail!("未知的防火墙规则类型: {}", rule_type),
    }
}

// ==================== 辅助函数 ====================

/// 从 params 中提取 bool 值，支持 bool 和 string 类型
fn params_to_bool(params: &serde_json::Value, key: &str) -> Result<bool> {
    if let Some(b) = params[key].as_bool() {
        return Ok(b);
    }
    if let Some(s) = params[key].as_str() {
        return match s.to_lowercase().as_str() {
            "true" | "on" | "yes" | "1" => Ok(true),
            "false" | "off" | "no" | "0" => Ok(false),
            _ => anyhow::bail!("无法解析 {} 的值: {}", key, s),
        };
    }
    // 默认 true（AI 建议开启某功能时通常省略 enable 参数）
    Ok(true)
}

/// 将 JSON 数组转为 Vec<String>
fn json_array_to_strings(value: &serde_json::Value) -> Option<Vec<String>> {
    value.as_array().map(|arr| {
        arr.iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect()
    })
}
