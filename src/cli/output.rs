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

/// 打印欢迎横幅
pub fn print_banner() {
    println!("{}", r#"
   ____  _____    _    ___
  / ___|  ___|  / \  |_ _|
 | |   | |_    / _ \  | |
 | |___|  _   / ___ \ | |
  \____|_|  /_/   \_\___|

  🚀 AI-Powered Cloudflare Management Tool
"#.cyan().bold());
}

/// 打印分隔线
pub fn separator() {
    println!("{}", "─".repeat(60).dimmed());
}

/// 打印双线分隔线
pub fn separator_bold() {
    println!("{}", "═".repeat(60).bold());
}

/// 打印带图标的步骤
pub fn step(num: usize, msg: &str) {
    println!("\n{} {}", format!("步骤 {}:", num).bold().cyan(), msg);
    separator();
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
    println!("{} {}", "⚠️ ".yellow(), msg.yellow());
}

/// 打印信息消息
pub fn info(msg: &str) {
    println!("{} {}", "ℹ️ ".blue(), msg);
}

/// 打印提示消息
pub fn tip(msg: &str) {
    println!("{} {}", "💡".bright_yellow(), msg.bright_yellow());
}

/// 打印加载中消息
pub fn loading(msg: &str) {
    println!("{} {}...", "⏳".cyan(), msg.cyan());
}

/// 打印标题
pub fn title(msg: &str) {
    println!("\n{}", msg.bold().cyan());
    separator();
}

/// 打印大标题（带边框）
pub fn title_box(msg: &str) {
    let width = 60;
    let padding = (width - msg.len() - 4) / 2;
    let left_pad = " ".repeat(padding);
    let right_pad = " ".repeat(width - msg.len() - 4 - padding);

    println!("\n{}", "╔".to_string() + &"═".repeat(width - 2) + "╗");
    println!("{}", format!("║{}{}{}║", left_pad, msg, right_pad).cyan().bold());
    println!("{}", "╚".to_string() + &"═".repeat(width - 2) + "╝");
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

/// 打印列表项
pub fn list_item(msg: &str) {
    println!("  {} {}", "•".cyan(), msg);
}

/// 打印带编号的列表项
pub fn list_numbered(num: usize, msg: &str) {
    println!("  {} {}", format!("{}.", num).cyan(), msg);
}

/// 打印进度信息
pub fn progress(current: usize, total: usize, msg: &str) {
    println!(
        "{} [{}/{}] {}",
        "▶".cyan(),
        current.to_string().green(),
        total.to_string().dimmed(),
        msg
    );
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
    separator();
    println!("{}", content);

    if let Some(t) = tokens {
        println!("\n{}", format!("💬 Token 用量: {}", t).dimmed());
    }
    println!();
}

/// 打印状态徽章
pub fn badge(label: &str, status: &str, is_good: bool) {
    let colored_status = if is_good {
        format!(" {} ", status).black().on_green()
    } else {
        format!(" {} ", status).black().on_red()
    };
    println!("{} {}", label.dimmed(), colored_status);
}

/// 打印命令建议
pub fn suggest_command(desc: &str, cmd: &str) {
    println!("  {} {}", desc.dimmed(), cmd.cyan());
}

/// 打印空行
pub fn newline() {
    println!();
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
