use anyhow::{Context, Result};
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
        use dialoguer::{Input, Select};

        println!("\n🔧 CFAI 配置向导\n");

        // Cloudflare 认证方式选择
        let auth_options = vec!["API Token (推荐)", "Email + Global API Key"];
        let auth_choice = Select::new()
            .with_prompt("选择 Cloudflare 认证方式")
            .items(&auth_options)
            .default(0)
            .interact()?;

        let mut config = AppConfig::default();

        match auth_choice {
            0 => {
                let token: String = Input::new()
                    .with_prompt("请输入 Cloudflare API Token")
                    .interact_text()?;
                config.cloudflare.api_token = Some(token);
            }
            1 => {
                let email: String = Input::new()
                    .with_prompt("请输入 Cloudflare 邮箱")
                    .interact_text()?;
                let key: String = Input::new()
                    .with_prompt("请输入 Global API Key")
                    .interact_text()?;
                config.cloudflare.email = Some(email);
                config.cloudflare.api_key = Some(key);
            }
            _ => unreachable!(),
        }

        // 可选: 账户 ID
        let account_id: String = Input::new()
            .with_prompt("Account ID (可选, 直接回车跳过)")
            .default(String::new())
            .interact_text()?;
        if !account_id.is_empty() {
            config.cloudflare.account_id = Some(account_id);
        }

        // AI 配置
        println!("\n🤖 AI 配置 (用于智能分析, 可选)\n");

        let ai_url: String = Input::new()
            .with_prompt("AI API URL")
            .default("https://api.openai.com/v1".to_string())
            .interact_text()?;
        config.ai.api_url = Some(ai_url);

        let ai_key: String = Input::new()
            .with_prompt("AI API Key (可选, 直接回车跳过)")
            .default(String::new())
            .interact_text()?;
        if !ai_key.is_empty() {
            config.ai.api_key = Some(ai_key);
        }

        let ai_model: String = Input::new()
            .with_prompt("AI 模型")
            .default("gpt-4o".to_string())
            .interact_text()?;
        config.ai.model = Some(ai_model);

        // 默认域名
        let default_domain: String = Input::new()
            .with_prompt("默认域名 (可选, 直接回车跳过)")
            .default(String::new())
            .interact_text()?;
        if !default_domain.is_empty() {
            config.defaults.domain = Some(default_domain);
        }

        config.save()?;
        println!("\n✅ 配置已保存到: {}", Self::config_path()?.display());

        Ok(config)
    }
}
