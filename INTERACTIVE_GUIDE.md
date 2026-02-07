# 🚀 CFAI 交互式使用指南

## ✨ 全新的交互式体验

CFAI v0.2.0 带来了全面优化的交互式用户体验，让配置和使用变得更加简单直观。

## 🎯 核心改进

### 1. **首次启动自动引导**

当您第一次运行 CFAI 时，系统会自动检测并引导您完成配置：

```bash
cfai zone list
```

如果未配置，会看到：

```
╔══════════════════════════════════════════════════╗
║            🎉 欢迎使用 CFAI                       ║
╚══════════════════════════════════════════════════╝

检测到您是第一次使用 CFAI，需要进行初始配置。
CFAI 是一个 AI 驱动的 Cloudflare 管理工具，可以帮助您：
  • 管理域名、DNS、SSL/TLS
  • 配置防火墙和缓存策略
  • 使用 AI 进行智能分析和优化

? 是否现在进行配置？ (Y/n)
```

### 2. **美化的配置向导**

#### 📡 第一步：Cloudflare API 配置

```
╔══════════════════════════════════════════════════╗
║        🚀 CFAI 配置向导 - 交互式设置             ║
╚══════════════════════════════════════════════════╝

📡 第一步：配置 Cloudflare API 访问
──────────────────────────────────────────────────

Cloudflare API 有两种认证方式：
  1. API Token - 更安全，权限可控 (推荐)
  2. Email + Global API Key - 传统方式

? 请选择认证方式 ›
❯ 🔑 API Token (推荐 - 更安全)
  📧 Email + Global API Key
```

**方式一：API Token（推荐）**

```
获取 API Token:
  1. 访问: https://dash.cloudflare.com/profile/api-tokens
  2. 点击 'Create Token'
  3. 选择适当的权限模板或自定义权限

? 请输入您的 Cloudflare API Token › **********************
✓ API Token 已设置
```

**方式二：Email + API Key**

```
获取 Global API Key:
  1. 访问: https://dash.cloudflare.com/profile/api-tokens
  2. 找到 'Global API Key' 部分
  3. 点击 'View' 查看密钥

? 请输入您的 Cloudflare 账户邮箱 › user@example.com
? 请输入 Global API Key › **********************
✓ Email + API Key 已设置
```

**可选：Account ID**

```
? 是否需要配置 Account ID？(某些 Workers 功能需要) (y/N)
```

#### 🤖 第二步：AI 智能助手配置

```
🤖 第二步：配置 AI 智能助手 (可选)
──────────────────────────────────────────────────

AI 功能可以帮助您：
  • 智能分析域名配置
  • 提供安全建议和优化方案
  • 故障诊断和问题解答

? 是否配置 AI 功能？ (Y/n)
```

**选择 AI 服务提供商**

```
支持的 AI 服务：
  • OpenAI (GPT-4, GPT-3.5)
  • DeepSeek
  • 任何兼容 OpenAI API 的服务

? 选择 AI 服务提供商 ›
❯ OpenAI (https://api.openai.com/v1)
  DeepSeek (https://api.deepseek.com)
  自定义 API 地址
```

**配置 AI 参数**

```
✓ AI API 地址已设置: https://api.openai.com/v1

? 请输入 AI API Key › sk-**********************
✓ AI API Key 已设置

? 选择 AI 模型 ›
❯ gpt-4o (推荐 - 最强大)
  gpt-4o-mini (更快，成本更低)
  gpt-3.5-turbo (经济实惠)
  deepseek-chat
  自定义模型

✓ AI 模型已设置: gpt-4o
```

#### ⚙️ 第三步：其他设置

```
⚙️ 第三步：其他设置 (可选)
──────────────────────────────────────────────────

? 是否配置默认域名？(可以简化后续命令) (y/N)

? 请输入默认域名 (例如: example.com) › example.com
✓ 默认域名已设置: example.com
```

### 3. **配置完成提示**

```
💾 保存配置...

╔══════════════════════════════════════════════════╗
║            ✅ 配置完成！                          ║
╚══════════════════════════════════════════════════╝

配置文件保存在: /Users/username/.config/cfai/config.toml

🚀 快速开始：
  cfai zone list - 列出所有域名
  cfai dns list <domain> - 查看 DNS 记录
  cfai ai analyze <domain> - AI 智能分析
  cfai --help - 查看帮助
```

## 🎨 视觉增强

### 新增的输出功能

#### 1. **欢迎横幅**
```
   ____  _____    _    ___
  / ___|  ___|  / \  |_ _|
 | |   | |_    / _ \  | |
 | |___|  _   / ___ \ | |
  \____|_|  /_/   \_\___|

  🚀 AI-Powered Cloudflare Management Tool
```

#### 2. **步骤指示器**
```
步骤 1: 配置 Cloudflare API
──────────────────────────────────────────────────
```

#### 3. **增强的消息类型**
- ✅ 成功消息（绿色）
- ❌ 错误消息（红色）
- ⚠️  警告消息（黄色）
- ℹ️  信息消息（蓝色）
- 💡 提示消息（亮黄色）
- ⏳ 加载中（青色）

#### 4. **进度显示**
```
▶ [1/3] 正在加载配置...
▶ [2/3] 验证认证信息...
▶ [3/3] 完成！
```

#### 5. **状态徽章**
```
状态  ACTIVE
类型  cloudflare
```

## 📖 使用示例

### 手动运行配置向导

```bash
# 随时可以重新配置
cfai config setup
```

### 查看当前配置

```bash
# 隐藏敏感信息
cfai config show

# 显示所有信息（包括 API Key）
cfai config show --show-secrets
```

### 验证配置

```bash
cfai config verify
```

### 查看配置文件路径

```bash
cfai config path
```

## 🔧 高级功能

### 环境变量覆盖

配置文件可以被环境变量覆盖：

```bash
# Cloudflare
export CLOUDFLARE_API_TOKEN="your-token"
export CLOUDFLARE_EMAIL="your-email"
export CLOUDFLARE_API_KEY="your-key"
export CLOUDFLARE_ACCOUNT_ID="your-account-id"

# AI
export AI_API_URL="https://api.openai.com/v1"
export AI_API_KEY="your-ai-key"
export AI_MODEL="gpt-4o"
export AI_MAX_TOKENS="4096"
export AI_TEMPERATURE="0.7"
```

### 交互模式

```bash
# 进入完全交互式模式
cfai interactive
```

## 💡 使用技巧

1. **首次使用**：直接运行任何命令，系统会自动引导配置
2. **快速配置**：使用 `cfai config setup` 一次性完成所有配置
3. **查看帮助**：任何时候使用 `--help` 查看命令帮助
4. **AI 可选**：如果不需要 AI 功能，可以跳过 AI 配置
5. **重新配置**：配置随时可以通过 `cfai config setup` 重新设置

## 🎯 配置示例

### 最小配置（仅 Cloudflare）

```toml
[cloudflare]
api_token = "your-cloudflare-api-token"

[ai]
api_url = "https://api.openai.com/v1"
model = "gpt-4o"
max_tokens = 4096
temperature = 0.7

[defaults]
```

### 完整配置

```toml
[cloudflare]
api_token = "your-cloudflare-api-token"
account_id = "your-account-id"

[ai]
api_url = "https://api.openai.com/v1"
api_key = "your-ai-api-key"
model = "gpt-4o"
max_tokens = 4096
temperature = 0.7

[defaults]
domain = "example.com"
output_format = "table"
color = true
```

## 🚀 开始使用

现在您已经了解了 CFAI 的交互式功能，让我们开始：

```bash
# 1. 初始化配置（首次使用会自动触发）
cfai config setup

# 2. 查看您的域名
cfai zone list

# 3. 管理 DNS 记录
cfai dns list example.com

# 4. 使用 AI 分析
cfai ai analyze example.com

# 5. 查看更多帮助
cfai --help
```

---

**享受全新的 CFAI 交互体验！** 🎉
