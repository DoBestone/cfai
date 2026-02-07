use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::config::settings::AppConfig;

/// AI 分析引擎
pub struct AiAnalyzer {
    client: reqwest::Client,
    api_url: String,
    api_key: String,
    model: String,
    max_tokens: u32,
    temperature: f32,
}

/// OpenAI 兼容的聊天请求
#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// OpenAI 兼容的聊天响应
#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
    usage: Option<ChatUsage>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ChatUsage {
    prompt_tokens: Option<u32>,
    completion_tokens: Option<u32>,
    total_tokens: Option<u32>,
}

/// AI 分析结果
#[derive(Debug)]
pub struct AnalysisResult {
    pub content: String,
    pub actions: Option<Vec<SuggestedAction>>,
    pub tokens_used: Option<u32>,
}

/// AI 建议的操作
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SuggestedAction {
    #[serde(rename = "type")]
    pub action_type: String,
    pub description: String,
    pub params: serde_json::Value,
    pub risk: String,
}

/// AI 操作方案
#[derive(Debug, Deserialize)]
struct AiActionPlan {
    actions: Option<Vec<SuggestedAction>>,
    explanation: Option<String>,
}

impl AiAnalyzer {
    /// 创建 AI 分析引擎
    pub fn new(config: &AppConfig) -> Result<Self> {
        let api_key = config
            .ai
            .api_key
            .clone()
            .context("未配置 AI API Key，请运行 `cfai config setup` 或设置 AI_API_KEY 环境变量")?;

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .context("创建 HTTP 客户端失败")?;

        Ok(Self {
            client,
            api_url: config.ai_api_url(),
            api_key,
            model: config.ai_model(),
            max_tokens: config.ai.max_tokens.unwrap_or(4096),
            temperature: config.ai.temperature.unwrap_or(0.7),
        })
    }

    /// 发送聊天请求
    async fn chat(&self, system_prompt: &str, user_message: &str) -> Result<AnalysisResult> {
        let request = ChatRequest {
            model: self.model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: user_message.to_string(),
                },
            ],
            max_tokens: self.max_tokens,
            temperature: self.temperature,
        };

        let url = format!("{}/chat/completions", self.api_url);
        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("AI API 请求失败")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("AI API 错误 (HTTP {}): {}", status, body);
        }

        let chat_resp: ChatResponse = resp.json().await.context("解析 AI 响应失败")?;

        let content = chat_resp
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        let tokens_used = chat_resp.usage.and_then(|u| u.total_tokens);

        // 尝试解析 AI 建议的操作
        let actions = self.extract_actions(&content);

        Ok(AnalysisResult {
            content,
            actions,
            tokens_used,
        })
    }

    /// 从 AI 响应中提取操作建议
    fn extract_actions(&self, content: &str) -> Option<Vec<SuggestedAction>> {
        // 查找 JSON 代码块
        if let Some(start) = content.find("```json") {
            if let Some(end) = content[start + 7..].find("```") {
                let json_str = &content[start + 7..start + 7 + end].trim();
                if let Ok(plan) = serde_json::from_str::<AiActionPlan>(json_str) {
                    return plan.actions;
                }
            }
        }

        // 尝试直接解析整个内容
        if let Ok(plan) = serde_json::from_str::<AiActionPlan>(content) {
            return plan.actions;
        }

        None
    }

    /// 分析 DNS 配置
    pub async fn analyze_dns(&self, dns_records: &str) -> Result<AnalysisResult> {
        let prompt = format!("{}{}", super::prompts::DNS_ANALYSIS_PROMPT, dns_records);
        self.chat(super::prompts::SYSTEM_PROMPT, &prompt).await
    }

    /// 分析安全配置
    pub async fn analyze_security(&self, security_config: &str) -> Result<AnalysisResult> {
        let prompt = format!(
            "{}{}",
            super::prompts::SECURITY_ANALYSIS_PROMPT,
            security_config
        );
        self.chat(super::prompts::SYSTEM_PROMPT, &prompt).await
    }

    /// 分析性能配置
    pub async fn analyze_performance(&self, perf_config: &str) -> Result<AnalysisResult> {
        let prompt = format!(
            "{}{}",
            super::prompts::PERFORMANCE_ANALYSIS_PROMPT,
            perf_config
        );
        self.chat(super::prompts::SYSTEM_PROMPT, &prompt).await
    }

    /// 故障诊断
    pub async fn troubleshoot(&self, issue_description: &str) -> Result<AnalysisResult> {
        let prompt = format!(
            "{}{}",
            super::prompts::TROUBLESHOOT_PROMPT,
            issue_description
        );
        self.chat(super::prompts::SYSTEM_PROMPT, &prompt).await
    }

    /// 自动配置建议
    pub async fn auto_config(&self, requirement: &str) -> Result<AnalysisResult> {
        let prompt = format!("{}{}", super::prompts::AUTO_CONFIG_PROMPT, requirement);
        self.chat(super::prompts::SYSTEM_PROMPT, &prompt).await
    }

    /// 自由问答
    pub async fn ask(&self, question: &str) -> Result<AnalysisResult> {
        self.chat(super::prompts::SYSTEM_PROMPT, question).await
    }

    /// 带上下文的问答（传入当前域名的完整配置）
    pub async fn ask_with_context(
        &self,
        question: &str,
        context: &str,
    ) -> Result<AnalysisResult> {
        let full_question = format!(
            "当前域名配置信息:\n{}\n\n用户问题:\n{}",
            context, question
        );
        self.chat(super::prompts::SYSTEM_PROMPT, &full_question)
            .await
    }
}
