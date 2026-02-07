<div align="center">

# 🚀 CFAI

### AI-Powered Cloudflare Management Tool

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Cloudflare](https://img.shields.io/badge/Cloudflare-API%20v4-F38020.svg)](https://developers.cloudflare.com/api/)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)

**CFAI** 是一个用 Rust 构建的 AI 驱动 Cloudflare 全功能管理工具。  
通过命令行即可完成域名、DNS、SSL、防火墙、缓存、Workers 等全部管理操作，  
并集成 AI 智能分析，提供安全建议、性能优化、故障诊断和自动配置方案。

[English](#english) · [功能特性](#-核心特性) · [快速开始](#-快速开始) · [命令参考](#-命令参考) · [Roadmap](#️-roadmap)

</div>

---

## ✨ 核心特性

| 特性 | 说明 |
|------|------|
| 🌐 **域名全功能管理** | Zone、DNS、SSL/TLS、防火墙、缓存、页面规则、Workers、流量分析 |
| 🤖 **AI 智能助手** | 配置分析、安全建议、性能优化、故障诊断、自动配置方案 |
| 🖥️ **CLI + GUI** | 命令行高效操作 + 图形界面直观管理（GUI 基于 Tauri，开发中） |
| 🎨 **美观输出** | 彩色表格、进度条、状态徽标、JSON/纯文本多格式输出 |
| 🔧 **灵活认证** | 支持 API Token 和 Email + Global API Key 两种认证方式 |
| 📦 **单文件分发** | Rust 编译为单个静态二进制文件，无需运行时依赖 |
| ⚙️ **灵活配置** | TOML 配置文件 + 环境变量覆盖，支持交互式配置向导 |
| 🔌 **AI 兼容** | 支持 OpenAI、DeepSeek 及任何兼容 OpenAI API 的服务 |

## 📦 安装

### 方式一：下载预编译二进制（推荐）

前往 [Releases](https://github.com/DoBestone/cfai/releases/latest) 页面下载对应平台的二进制文件：

```bash
# macOS / Linux
chmod +x cfai
sudo mv cfai /usr/local/bin/

# 验证安装
cfai --version
```

### 方式一（可选）：一键安装脚本

```bash
curl -fsSL https://raw.githubusercontent.com/DoBestone/cfai/main/scripts/install.sh | bash
```

### 方式二：使用 Cargo 安装

```bash
cargo install --git https://github.com/DoBestone/cfai.git
```

### 方式三：从源码编译

```bash
# 确保已安装 Rust 1.70+ (https://rustup.rs)
git clone https://github.com/DoBestone/cfai.git
cd cfai
cargo build --release

# 二进制文件位于 target/release/cfai
sudo cp target/release/cfai /usr/local/bin/
```

## 🔧 快速开始

### 1. 初始化配置

```bash
# 交互式配置向导（推荐首次使用）
cfai config setup
```

或手动配置：

```bash
# 设置 Cloudflare API Token
cfai config set cloudflare.api_token YOUR_CLOUDFLARE_API_TOKEN

# 设置 AI 服务（可选，启用 AI 功能）
cfai config set ai.api_key YOUR_AI_API_KEY
cfai config set ai.api_url https://api.openai.com/v1
cfai config set ai.model gpt-4o
```

也可通过环境变量配置：

```bash
export CLOUDFLARE_API_TOKEN="your-token"
export AI_API_KEY="your-ai-key"
export AI_API_URL="https://api.openai.com/v1"
```

### 2. 验证配置

```bash
cfai config verify
```

### 3. 开始使用

```bash
# 列出所有域名
cfai zone list

# 查看 DNS 记录
cfai dns list example.com

# AI 智能分析域名配置
cfai ai analyze example.com

# AI 自由问答
cfai ai ask "如何优化网站性能和安全性"
```

## 📖 命令参考

### 域名管理 (`zone` / `z`)

```bash
cfai zone list                      # 列出所有域名
cfai zone get example.com           # 查看域名详情
cfai zone add example.com           # 添加域名
cfai zone delete example.com        # 删除域名
cfai zone pause example.com         # 暂停域名
cfai zone resume example.com        # 恢复域名
cfai zone check example.com         # 检查激活状态
cfai zone settings example.com      # 查看所有设置
cfai zone set example.com key value # 修改设置
```

### DNS 管理 (`dns` / `d`)

```bash
cfai dns list example.com                           # 列出 DNS 记录
cfai dns list example.com -t A                      # 按类型过滤
cfai dns add example.com -t A -n www -c 1.2.3.4     # 添加记录
cfai dns add-a example.com www 1.2.3.4              # 快速添加 A 记录
cfai dns add-cname example.com blog target.com      # 快速添加 CNAME
cfai dns update example.com RECORD_ID -c 5.6.7.8   # 更新记录
cfai dns delete example.com RECORD_ID               # 删除记录
cfai dns find example.com www                       # 搜索记录
cfai dns export example.com                         # 导出记录
```

### SSL/TLS 管理 (`ssl`)

```bash
cfai ssl status example.com          # 查看 SSL 状态
cfai ssl mode example.com strict     # 设置 SSL 模式
cfai ssl https example.com on        # 开启 Always HTTPS
cfai ssl min-tls example.com 1.2     # 设置最小 TLS 版本
cfai ssl verify example.com          # 查看验证状态
cfai ssl list example.com            # 列出证书
cfai ssl origin-certs example.com    # 列出源服务器证书
cfai ssl auto-rewrite example.com on # 自动 HTTPS 重写
```

### 防火墙管理 (`firewall` / `fw`)

```bash
cfai firewall status example.com                     # 安全概览
cfai firewall list example.com                       # 列出防火墙规则
cfai firewall ip-rules example.com                   # 列出 IP 规则
cfai firewall block example.com 1.2.3.4              # 封禁 IP
cfai firewall whitelist example.com 5.6.7.8          # IP 白名单
cfai firewall unblock example.com RULE_ID            # 删除 IP 规则
cfai firewall level example.com high                 # 设置安全级别
cfai firewall ua-on example.com                      # 开启 Under Attack
cfai firewall ua-off example.com                     # 关闭 Under Attack
cfai firewall rate-limits example.com                # 列出速率限制
```

### 缓存管理 (`cache`)

```bash
cfai cache status example.com                        # 查看缓存设置
cfai cache purge-all example.com                     # 清除全部缓存
cfai cache purge-url example.com https://...         # 按 URL 清除
cfai cache purge-host example.com blog.example.com   # 按主机名清除
cfai cache level example.com aggressive              # 设置缓存级别
cfai cache browser-ttl example.com 14400             # 设置浏览器缓存
cfai cache dev-mode example.com on                   # 开启开发模式
```

### 页面规则 (`page-rules` / `pr`)

```bash
cfai page-rules list example.com                                           # 列出规则
cfai page-rules get example.com RULE_ID                                    # 规则详情
cfai page-rules redirect example.com "*example.com/old/*" "https://new/*"  # URL 跳转
cfai page-rules delete example.com RULE_ID                                 # 删除规则
```

### Workers 管理 (`workers` / `w`)

```bash
cfai workers list                        # 列出 Workers 脚本
cfai workers delete script-name          # 删除脚本
cfai workers routes example.com          # 列出路由
cfai workers kv                          # 列出 KV 命名空间
cfai workers domains                     # 列出自定义域名
```

### 流量分析 (`analytics` / `stats`)

```bash
cfai analytics overview example.com      # 24小时流量概览
cfai analytics detail example.com        # 详细分析
```

### 🤖 AI 智能助手 (`ai`)

```bash
cfai ai ask "如何防止 DDoS 攻击"                     # 自由问答
cfai ai analyze example.com                          # 全面分析
cfai ai analyze example.com -t dns                   # DNS 分析
cfai ai analyze example.com -t security              # 安全分析
cfai ai analyze example.com -t performance           # 性能分析
cfai ai troubleshoot "网站打不开" -d example.com     # 故障诊断
cfai ai auto-config "配置一个安全的博客网站"         # 自动配置建议
```

### 配置管理 (`config`)

```bash
cfai config setup                # 交互式配置
cfai config show                 # 查看配置
cfai config show --show-secrets  # 显示敏感信息
cfai config set KEY VALUE        # 设置配置项
cfai config path                 # 配置文件路径
cfai config verify               # 验证配置
```

### 安装 / 更新 / 交互模式

```bash
cfai install                     # 下载并安装最新二进制
cfai update                      # 更新到最新版本
cfai interactive                 # 进入交互模式
```

## 🎛️ 全局选项

```bash
--format table|json|plain    # 输出格式
-v, --verbose                # 详细输出
```

## 🏗️ 项目结构

```
cfai/
├── Cargo.toml              # Rust 项目配置
├── src/
│   ├── main.rs             # 程序入口
│   ├── api/                # Cloudflare API 客户端
│   │   ├── client.rs       # HTTP 客户端封装
│   │   ├── zone.rs         # 域名 API
│   │   ├── dns.rs          # DNS API
│   │   ├── ssl.rs          # SSL API
│   │   ├── firewall.rs     # 防火墙 API
│   │   ├── cache.rs        # 缓存 API
│   │   ├── page_rules.rs   # 页面规则 API
│   │   ├── workers.rs      # Workers API
│   │   └── analytics.rs    # 分析数据 API
│   ├── models/             # 数据模型
│   │   ├── common.rs       # 通用模型
│   │   ├── zone.rs         # 域名模型
│   │   ├── dns.rs          # DNS 模型
│   │   └── ...
│   ├── cli/                # CLI 界面
│   │   ├── commands/       # 命令实现
│   │   └── output.rs       # 输出格式化
│   ├── ai/                 # AI 模块
│   │   ├── analyzer.rs     # AI 分析引擎
│   │   └── prompts.rs      # 提示词模板
│   └── config/             # 配置管理
│       └── settings.rs     # 配置模型
```

## 🔑 认证方式

### 方式一: API Token (推荐)

1. 登录 [Cloudflare Dashboard](https://dash.cloudflare.com/profile/api-tokens)
2. 创建 API Token，建议权限：
   - Zone: Read, Edit
   - DNS: Read, Edit
   - SSL and Certificates: Read, Edit
   - Firewall Services: Read, Edit
   - Analytics: Read

### 方式二: Global API Key

1. 在 [API Tokens](https://dash.cloudflare.com/profile/api-tokens) 页面获取 Global API Key
2. 配置邮箱和 Key

## 🛣️ Roadmap

- [x] CLI 版本核心功能
- [ ] AI 建议自动执行
- [ ] DNS 记录批量导入
- [ ] Workers 脚本上传
- [ ] GUI 版本 (Tauri)
- [ ] 自定义规则引擎
- [ ] 配置模板/预设
- [ ] 多账户管理

---

## 🌍 English <a name="english"></a>

**CFAI** is an AI-powered Cloudflare management tool built with Rust. It provides full-featured domain management through CLI, including DNS, SSL/TLS, firewall, cache, Workers, and analytics — with integrated AI assistant for security analysis, performance optimization, troubleshooting, and auto-configuration.

### Key Features

- 🌐 **Full Domain Management** — Zone, DNS, SSL, Firewall, Cache, Page Rules, Workers, Analytics
- 🤖 **AI Assistant** — Intelligent analysis, security recommendations, performance optimization, auto-config
- 📦 **Single Binary** — Compiled to a single static binary with no runtime dependencies
- 🔌 **AI Compatible** — Works with OpenAI, DeepSeek, and any OpenAI-compatible API

### Quick Start

```bash
# Install
cargo install --git https://github.com/DoBestone/cfai.git

# Setup
cfai config setup

# Use
cfai zone list
cfai dns list example.com
cfai ai analyze example.com
cfai ai ask "How to protect against DDoS attacks?"
```

For full command reference, see the [Chinese documentation above](#-命令参考).

---

## 🤝 Contributing

欢迎贡献！无论是 Bug 报告、功能建议还是代码贡献，都非常感谢。

1. Fork 本仓库
2. 创建你的特性分支 (`git checkout -b feature/amazing-feature`)
3. 提交你的修改 (`git commit -m 'feat: add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 提交 Pull Request

### 提交规范

本项目使用 [Conventional Commits](https://www.conventionalcommits.org/) 规范：

- `feat:` 新功能
- `fix:` 修复 Bug
- `docs:` 文档更新
- `refactor:` 代码重构
- `perf:` 性能优化
- `test:` 测试相关
- `chore:` 构建/工具变动

## ⭐ Star History

如果这个项目对你有帮助，请给一个 ⭐ Star！

## 📄 License

[MIT License](LICENSE) © DoBest
