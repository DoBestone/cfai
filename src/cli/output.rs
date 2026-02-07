use colored::Colorize;
use comfy_table::{Cell, CellAlignment, Color, ContentArrangement, Table};

/// 输出格式
#[derive(Debug, Clone, PartialEq)]
pub enum OutputFormat {
    Table,
    Json,
    Plain,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "table" => Ok(OutputFormat::Table),
            "json" => Ok(OutputFormat::Json),
            "plain" | "text" => Ok(OutputFormat::Plain),
            _ => Err(format!("未知的输出格式: {}", s)),
        }
    }
}

/// 打印成功消息
pub fn success(msg: &str) {
    println!("{} {}", "✅".green(), msg.green());
}

/// 打印错误消息
pub fn error(msg: &str) {
    eprintln!("{} {}", "❌".red(), msg.red());
}

/// 打印警告消息
pub fn warn(msg: &str) {
    println!("{} {}", "⚠️".yellow(), msg.yellow());
}

/// 打印信息消息
pub fn info(msg: &str) {
    println!("{} {}", "ℹ️".blue(), msg);
}

/// 打印标题
pub fn title(msg: &str) {
    println!("\n{}", msg.bold().cyan());
    println!("{}", "─".repeat(50).dimmed());
}

/// 打印键值对
pub fn kv(key: &str, value: &str) {
    println!("  {} {}", format!("{}:", key).dimmed(), value);
}

/// 打印带颜色的键值对
pub fn kv_colored(key: &str, value: &str, is_good: bool) {
    let colored_value = if is_good {
        value.green().to_string()
    } else {
        value.red().to_string()
    };
    println!("  {} {}", format!("{}:", key).dimmed(), colored_value);
}

/// 创建表格
pub fn create_table(headers: Vec<&str>) -> Table {
    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);

    let header_cells: Vec<Cell> = headers
        .iter()
        .map(|h| {
            Cell::new(h)
                .set_alignment(CellAlignment::Center)
                .fg(Color::Cyan)
        })
        .collect();
    table.set_header(header_cells);

    table
}

/// 打印 JSON 格式
pub fn print_json<T: serde::Serialize>(data: &T) {
    match serde_json::to_string_pretty(data) {
        Ok(json) => println!("{}", json),
        Err(e) => error(&format!("JSON 序列化失败: {}", e)),
    }
}

/// 打印 AI 分析结果
pub fn print_ai_result(content: &str, tokens: Option<u32>) {
    println!("\n{}", "🤖 AI 分析结果".bold().cyan());
    println!("{}", "─".repeat(50).dimmed());
    println!("{}", content);

    if let Some(t) = tokens {
        println!("\n{}", format!("Token 用量: {}", t).dimmed());
    }
}

/// 打印 AI 建议的操作
pub fn print_ai_actions(actions: &[crate::ai::analyzer::SuggestedAction]) {
    if actions.is_empty() {
        return;
    }

    println!("\n{}", "📋 建议操作".bold().yellow());
    println!("{}", "─".repeat(50).dimmed());

    for (i, action) in actions.iter().enumerate() {
        let risk_color = match action.risk.as_str() {
            "low" => "🟢",
            "medium" => "🟡",
            "high" => "🔴",
            _ => "⚪",
        };

        println!(
            "  {}. {} {} [{}]",
            i + 1,
            risk_color,
            action.description,
            action.action_type.dimmed()
        );
    }
}

/// 格式化字节大小
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// 格式化数字 (千分位)
pub fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

/// 状态徽标
pub fn status_badge(status: &str) -> String {
    match status.to_lowercase().as_str() {
        "active" => format!("{}", "● active".green()),
        "pending" => format!("{}", "● pending".yellow()),
        "initializing" => format!("{}", "● initializing".yellow()),
        "moved" => format!("{}", "● moved".blue()),
        "deleted" => format!("{}", "● deleted".red()),
        "deactivated" => format!("{}", "● deactivated".dimmed()),
        "on" | "true" | "enabled" => format!("{}", "● ON".green()),
        "off" | "false" | "disabled" => format!("{}", "● OFF".red()),
        _ => status.to_string(),
    }
}
