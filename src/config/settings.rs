use anyhow::{Context, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 应用配置
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub cloudflare: CloudflareConfig,
    pub ai: AiConfig,
    #[serde(default)]
    pub defaults: DefaultsConfig,
}

/// Cloudflare 配置
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CloudflareConfig {
    /// API Token (推荐方式)
    pub api_token: Option<String>,
    /// 邮箱 (配合 api_key 使用)
    pub email: Option<String>,
    /// Global API Key
    pub api_key: Option<String>,
    /// 账户 ID
    pub account_id: Option<String>,
}

/// AI 配置
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiConfig {
    /// AI API 地址 (OpenAI 兼容)
    pub api_url: Option<String>,
    /// AI API Key
    pub api_key: Option<String>,
    /// 模型名称
    pub model: Option<String>,
    /// 最大 Token 数
    pub max_tokens: Option<u32>,
    /// 温度参数
    pub temperature: Option<f32>,
}

/// 默认配置
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct DefaultsConfig {
    /// 默认域名
    pub domain: Option<String>,
    /// 默认输出格式 (table/json/yaml)
    pub output_format: Option<String>,
    /// 是否启用颜色输出
    pub color: Option<bool>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            cloudflare: CloudflareConfig {
                api_token: None,
                email: None,
                api_key: None,
                account_id: None,
            },
            ai: AiConfig {
                api_url: Some("https://api.openai.com/v1".to_string()),
                api_key: None,
                model: Some("gpt-4o".to_string()),
                max_tokens: Some(4096),
                temperature: Some(0.7),
            },
            defaults: DefaultsConfig::default(),
        }
    }
}

impl AppConfig {
    /// 获取配置文件路径
    pub fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("无法获取配置目录")?
            .join("cfai");
        Ok(config_dir.join("config.toml"))
    }

    /// 加载配置
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;

        if !path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("读取配置文件失败: {}", path.display()))?;

        let config: AppConfig = toml::from_str(&content)
            .with_context(|| format!("解析配置文件失败: {}", path.display()))?;

        Ok(config)
    }

    /// 从环境变量覆盖
    pub fn merge_env(mut self) -> Self {
        if let Ok(token) = std::env::var("CLOUDFLARE_API_TOKEN") {
            self.cloudflare.api_token = Some(token);
        }
        if let Ok(email) = std::env::var("CLOUDFLARE_EMAIL") {
            self.cloudflare.email = Some(email);
        }
        if let Ok(key) = std::env::var("CLOUDFLARE_API_KEY") {
            self.cloudflare.api_key = Some(key);
        }
        if let Ok(account_id) = std::env::var("CLOUDFLARE_ACCOUNT_ID") {
            self.cloudflare.account_id = Some(account_id);
        }
        if let Ok(url) = std::env::var("AI_API_URL") {
            self.ai.api_url = Some(url);
        }
        if let Ok(key) = std::env::var("AI_API_KEY") {
            self.ai.api_key = Some(key);
        }
        if let Ok(model) = std::env::var("AI_MODEL") {
            self.ai.model = Some(model);
        }
        if let Ok(tokens) = std::env::var("AI_MAX_TOKENS") {
            if let Ok(t) = tokens.parse() {
                self.ai.max_tokens = Some(t);
            }
        }
        if let Ok(temp) = std::env::var("AI_TEMPERATURE") {
            if let Ok(t) = temp.parse() {
                self.ai.temperature = Some(t);
            }
        }
        self
    }

    /// 保存配置
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("创建配置目录失败: {}", parent.display()))?;
        }

        let content = toml::to_string_pretty(self).context("序列化配置失败")?;
        std::fs::write(&path, content)
            .with_context(|| format!("写入配置文件失败: {}", path.display()))?;

        Ok(())
    }

    /// 验证配置是否有效
    pub fn validate(&self) -> Result<()> {
        // 检查 Cloudflare 认证信息
        let has_token = self.cloudflare.api_token.is_some();
        let has_key = self.cloudflare.email.is_some() && self.cloudflare.api_key.is_some();

        if !has_token && !has_key {
            anyhow::bail!(
                "未配置 Cloudflare 认证信息！\n\
                请设置以下任一方式:\n\
                  1. API Token: cfai config set cloudflare.api_token <TOKEN>\n\
                  2. Email + API Key:\n\
                     cfai config set cloudflare.email <EMAIL>\n\
                     cfai config set cloudflare.api_key <KEY>\n\
                或设置环境变量: CLOUDFLARE_API_TOKEN"
            );
        }

        Ok(())
    }

    /// 获取 AI 配置中的 API URL
    pub fn ai_api_url(&self) -> String {
        self.ai
            .api_url
            .clone()
            .unwrap_or_else(|| "https://api.openai.com/v1".to_string())
    }

    /// 获取 AI 模型名
    pub fn ai_model(&self) -> String {
        self.ai
            .model
            .clone()
            .unwrap_or_else(|| "gpt-4o".to_string())
    }

    /// 交互式配置向导
    pub fn interactive_setup() -> Result<Self> {
        use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};

        let theme = ColorfulTheme::default();

        println!("\n{}", "╔══════════════════════════════════════════════════╗".cyan());
        println!("{}", "║        🚀 CFAI 配置向导 - 交互式设置             ║".cyan());
        println!("{}", "╚══════════════════════════════════════════════════╝".cyan());

        let mut config = AppConfig::default();

        // ========== Cloudflare 配置 ==========
        println!("\n{}", "📡 第一步：配置 Cloudflare API 访问".bold().green());
        println!("{}", "─".repeat(50).dimmed());
        println!("\n{}", "Cloudflare API 有两种认证方式：".dimmed());
        println!("  {} API Token - 更安全，权限可控 (推荐)", "1.".cyan());
        println!("  {} Email + Global API Key - 传统方式", "2.".cyan());
        println!();

        let auth_options = vec![
            "🔑 API Token (推荐 - 更安全)",
            "📧 Email + Global API Key"
        ];
        let auth_choice = Select::with_theme(&theme)
            .with_prompt("请选择认证方式")
            .items(&auth_options)
            .default(0)
            .interact()?;

        match auth_choice {
            0 => {
                println!("\n{}", "获取 API Token:".yellow());
                println!("  1. 访问: {}", "https://dash.cloudflare.com/profile/api-tokens".cyan());
                println!("  2. 点击 'Create Token'");
                println!("  3. 选择适当的权限模板或自定义权限");
                println!();

                let token: String = Input::with_theme(&theme)
                    .with_prompt("请输入您的 Cloudflare API Token")
                    .interact_text()?;

                if token.trim().is_empty() {
                    anyhow::bail!("API Token 不能为空");
                }
                config.cloudflare.api_token = Some(token.trim().to_string());
                println!("{}", "✓ API Token 已设置".green());
            }
            1 => {
                println!("\n{}", "获取 Global API Key:".yellow());
                println!("  1. 访问: {}", "https://dash.cloudflare.com/profile/api-tokens".cyan());
                println!("  2. 找到 'Global API Key' 部分");
                println!("  3. 点击 'View' 查看密钥");
                println!();

                let email: String = Input::with_theme(&theme)
                    .with_prompt("请输入您的 Cloudflare 账户邮箱")
                    .interact_text()?;

                if email.trim().is_empty() || !email.contains('@') {
                    anyhow::bail!("请输入有效的邮箱地址");
                }

                let key: String = Input::with_theme(&theme)
                    .with_prompt("请输入 Global API Key")
                    .interact_text()?;

                if key.trim().is_empty() {
                    anyhow::bail!("API Key 不能为空");
                }

                config.cloudflare.email = Some(email.trim().to_string());
                config.cloudflare.api_key = Some(key.trim().to_string());
                println!("{}", "✓ Email + API Key 已设置".green());
            }
            _ => unreachable!(),
        }

        // Account ID (可选)
        println!();
        let need_account_id = Confirm::with_theme(&theme)
            .with_prompt("是否需要配置 Account ID？(某些 Workers 功能需要)")
            .default(false)
            .interact()?;

        if need_account_id {
            println!("\n{}", "获取 Account ID:".yellow());
            println!("  1. 访问: {}", "https://dash.cloudflare.com/".cyan());
            println!("  2. 在右侧边栏可以找到 Account ID");
            println!();

            let account_id: String = Input::with_theme(&theme)
                .with_prompt("请输入 Account ID")
                .allow_empty(true)
                .interact_text()?;

            if !account_id.trim().is_empty() {
                config.cloudflare.account_id = Some(account_id.trim().to_string());
                println!("{}", "✓ Account ID 已设置".green());
            }
        }

        // ========== AI 配置 ==========
        println!("\n{}", "🤖 第二步：配置 AI 智能助手 (可选)".bold().green());
        println!("{}", "─".repeat(50).dimmed());
        println!("\n{}", "AI 功能可以帮助您：".dimmed());
        println!("  • 智能分析域名配置");
        println!("  • 提供安全建议和优化方案");
        println!("  • 故障诊断和问题解答");
        println!();

        let setup_ai = Confirm::with_theme(&theme)
            .with_prompt("是否配置 AI 功能？")
            .default(true)
            .interact()?;

        if setup_ai {
            println!("\n{}", "支持的 AI 服务：".yellow());
            println!("  • OpenAI (GPT-4, GPT-3.5)");
            println!("  • DeepSeek");
            println!("  • 任何兼容 OpenAI API 的服务");
            println!();

            let ai_presets = vec![
                "OpenAI (https://api.openai.com/v1)",
                "DeepSeek (https://api.deepseek.com)",
                "自定义 API 地址"
            ];

            let ai_preset = Select::with_theme(&theme)
                .with_prompt("选择 AI 服务提供商")
                .items(&ai_presets)
                .default(0)
                .interact()?;

            let ai_url = match ai_preset {
                0 => "https://api.openai.com/v1".to_string(),
                1 => "https://api.deepseek.com".to_string(),
                2 => {
                    Input::with_theme(&theme)
                        .with_prompt("请输入自定义 API 地址")
                        .interact_text()?
                }
                _ => unreachable!(),
            };
            config.ai.api_url = Some(ai_url.clone());
            println!("{}", format!("✓ AI API 地址已设置: {}", ai_url).green());

            let ai_key: String = Input::with_theme(&theme)
                .with_prompt("请输入 AI API Key")
                .allow_empty(true)
                .interact_text()?;

            if !ai_key.trim().is_empty() {
                config.ai.api_key = Some(ai_key.trim().to_string());
                println!("{}", "✓ AI API Key 已设置".green());
            } else {
                println!("{}", "⚠ 未设置 AI API Key，AI 功能将不可用".yellow());
            }

            // 模型选择
            let model_options = vec![
                "gpt-4o (推荐 - 最强大)",
                "gpt-4o-mini (更快，成本更低)",
                "gpt-3.5-turbo (经济实惠)",
                "deepseek-chat",
                "自定义模型"
            ];

            let model_choice = Select::with_theme(&theme)
                .with_prompt("选择 AI 模型")
                .items(&model_options)
                .default(0)
                .interact()?;

            let model = match model_choice {
                0 => "gpt-4o".to_string(),
                1 => "gpt-4o-mini".to_string(),
                2 => "gpt-3.5-turbo".to_string(),
                3 => "deepseek-chat".to_string(),
                4 => {
                    Input::with_theme(&theme)
                        .with_prompt("请输入模型名称")
                        .interact_text()?
                }
                _ => unreachable!(),
            };
            config.ai.model = Some(model.clone());
            println!("{}", format!("✓ AI 模型已设置: {}", model).green());
        } else {
            println!("{}", "ℹ 跳过 AI 配置，您可以稍后运行 'cfai config setup' 重新配置".dimmed());
        }

        // ========== 默认设置 ==========
        println!("\n{}", "⚙️  第三步：其他设置 (可选)".bold().green());
        println!("{}", "─".repeat(50).dimmed());

        let need_defaults = Confirm::with_theme(&theme)
            .with_prompt("是否配置默认域名？(可以简化后续命令)")
            .default(false)
            .interact()?;

        if need_defaults {
            let default_domain: String = Input::with_theme(&theme)
                .with_prompt("请输入默认域名 (例如: example.com)")
                .allow_empty(true)
                .interact_text()?;

            if !default_domain.trim().is_empty() {
                config.defaults.domain = Some(default_domain.trim().to_string());
                println!("{}", format!("✓ 默认域名已设置: {}", default_domain.trim()).green());
            }
        }

        // ========== 保存配置 ==========
        println!("\n{}", "💾 保存配置...".bold().cyan());
        config.save()?;

        let config_path = Self::config_path()?;
        println!("\n{}", "╔══════════════════════════════════════════════════╗".green());
        println!("{}", "║            ✅ 配置完成！                          ║".green());
        println!("{}", "╚══════════════════════════════════════════════════╝".green());
        println!("\n{}", format!("配置文件保存在: {}", config_path.display()).dimmed());

        println!("\n{}", "🚀 快速开始：".bold().yellow());
        println!("  {} 列出所有域名", "cfai zone list".cyan());
        println!("  {} 查看 DNS 记录", "cfai dns list <domain>".cyan());
        println!("  {} AI 智能分析", "cfai ai analyze <domain>".cyan());
        println!("  {} 查看帮助", "cfai --help".cyan());
        println!();

        Ok(config)
    }
}
