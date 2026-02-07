#!/usr/bin/env bash
set -euo pipefail

# CFAI 一键安装脚本
# 用法: curl -fsSL https://raw.githubusercontent.com/DoBestone/cfai/main/scripts/install.sh | bash

REPO="${CFAI_REPO:-DoBestone/cfai}"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
# GitHub 镜像加速 (国内用户)
MIRROR="${CFAI_MIRROR:-https://mirror.ghproxy.com/}"

# 颜色
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

info() { echo -e "${GREEN}==>${NC} $1"; }
error() { echo -e "${RED}错误:${NC} $1" >&2; exit 1; }
warn() { echo -e "${YELLOW}警告:${NC} $1" >&2; }
step() { echo -e "${CYAN}[$1]${NC} $2"; }

# 检测是否需要镜像加速
need_mirror() {
    local test_time=$(curl -o /dev/null -s -w '%{time_total}' --connect-timeout 3 "https://github.com" 2>/dev/null || echo "999")
    if (( $(echo "$test_time > 2" | bc -l 2>/dev/null || echo 1) )); then
        return 0
    fi
    return 1
}

# 检测平台
detect_platform() {
    local os=$(uname -s | tr '[:upper:]' '[:lower:]')
    local arch=$(uname -m)

    case "$os" in
        darwin) OS="darwin" ;;
        linux) OS="linux" ;;
        msys*|mingw*|cygwin*|windows*) OS="windows" ;;
        *) error "不支持的操作系统: $os" ;;
    esac

    case "$arch" in
        x86_64|amd64) ARCH="x86_64" ;;
        arm64|aarch64) ARCH="arm64" ;;
        *) error "不支持的架构: $arch" ;;
    esac

    step "平台" "$OS-$ARCH"
}

# 检查安装目录
check_install_dir() {
    if [[ ! -d "$INSTALL_DIR" ]]; then
        if ! mkdir -p "$INSTALL_DIR" 2>/dev/null; then
            INSTALL_DIR="$HOME/.local/bin"
            mkdir -p "$INSTALL_DIR"
        fi
    fi

    if [[ ! -w "$INSTALL_DIR" ]]; then
        warn "$INSTALL_DIR 无写入权限，将安装到 $HOME/.local/bin"
        INSTALL_DIR="$HOME/.local/bin"
        mkdir -p "$INSTALL_DIR"
    fi

    step "安装目录" "$INSTALL_DIR"
}

# 获取最新版本
get_latest_release() {
    step "检查" "正在获取最新版本..."

    local api_url="https://api.github.com/repos/$REPO/releases/latest"
    local response

    if ! response=$(curl -fsSL "$api_url" 2>&1); then
        error "无法获取 Release 信息: $response"
    fi

    VERSION=$(echo "$response" | grep -o '"tag_name": *"[^"]*"' | head -1 | sed 's/"tag_name": *"\([^"]*\)"/\1/')

    # 根据平台选择资源
    local pattern="cfai.*${OS}.*${ARCH}"
    DOWNLOAD_URL=$(echo "$response" | grep -o '"browser_download_url": *"[^"]*'"$pattern"'[^"]*"' | head -1 | sed 's/"browser_download_url": *"\([^"]*\)"/\1/')
    ASSET_NAME=$(basename "$DOWNLOAD_URL" 2>/dev/null || echo "")

    if [[ -z "$DOWNLOAD_URL" ]]; then
        DOWNLOAD_URL=$(echo "$response" | grep -o '"browser_download_url": *"[^"]*cfai[^"]*"' | head -1 | sed 's/"browser_download_url": *"\([^"]*\)"/\1/')
        ASSET_NAME=$(basename "$DOWNLOAD_URL" 2>/dev/null || echo "")
    fi

    if [[ -z "$DOWNLOAD_URL" ]]; then
        error "未找到适合 $OS-$ARCH 的下载资源"
    fi

    step "最新版本" "$VERSION"
}

# 下载并安装
install_cfai() {
    local tmp_dir=$(mktemp -d)
    trap 'rm -rf "$tmp_dir"' EXIT

    local download_file="$tmp_dir/$ASSET_NAME"
    local final_url="$DOWNLOAD_URL"

    # 检测是否使用镜像
    if need_mirror 2>/dev/null; then
        step "加速" "使用镜像下载..."
        final_url="${MIRROR}${DOWNLOAD_URL}"
    fi

    step "下载" "$ASSET_NAME"
    if ! curl -fSL "$final_url" -o "$download_file" --progress-bar --connect-timeout 10; then
        if [[ "$final_url" != "$DOWNLOAD_URL" ]]; then
            warn "镜像下载失败，尝试直连..."
            if ! curl -fSL "$DOWNLOAD_URL" -o "$download_file" --progress-bar; then
                error "下载失败"
            fi
        else
            error "下载失败"
        fi
    fi

    step "安装" "正在解压..."

    local bin_file=""

    if [[ "$ASSET_NAME" == *.tar.gz ]] || [[ "$ASSET_NAME" == *.tgz ]]; then
        tar -xzf "$download_file" -C "$tmp_dir"
        bin_file=$(find "$tmp_dir" -type f \( -name "cfai" -o -name "cfai-*" -o -name "cfai.exe" \) ! -name "*.md" ! -name "*.txt" -print -quit)
    elif [[ "$ASSET_NAME" == *.zip ]]; then
        unzip -q "$download_file" -d "$tmp_dir"
        bin_file=$(find "$tmp_dir" -type f \( -name "cfai" -o -name "cfai-*" -o -name "cfai.exe" \) ! -name "*.md" ! -name "*.txt" -print -quit)
    else
        bin_file="$download_file"
    fi

    if [[ -z "$bin_file" ]] || [[ ! -f "$bin_file" ]]; then
        error "未找到可执行文件"
    fi

    local target="$INSTALL_DIR/cfai"
    if [[ -w "$INSTALL_DIR" ]]; then
        cp "$bin_file" "$target"
        chmod +x "$target"
    else
        sudo cp "$bin_file" "$target"
        sudo chmod +x "$target"
    fi

    step "完成" "安装成功！"
}

# 验证安装
verify_installation() {
    local cfai_path="$INSTALL_DIR/cfai"

    if [[ ! -f "$cfai_path" ]]; then
        error "安装文件不存在: $cfai_path"
    fi

    local installed_version=$("$cfai_path" --version 2>&1 | head -1)

    echo ""
    echo -e "${GREEN}════════════════════════════════════════${NC}"
    echo -e "${GREEN}  ✅ CFAI 安装成功！${NC}"
    echo -e "${GREEN}════════════════════════════════════════${NC}"
    echo ""
    echo "  版本: $installed_version"
    echo "  路径: $cfai_path"
    echo ""
    echo "快速开始:"
    echo "  cfai config setup    配置 API"
    echo "  cfai zone list       列出域名"
    echo "  cfai --help          查看帮助"
    echo ""

    if ! command -v cfai >/dev/null 2>&1; then
        warn "cfai 不在 PATH 中，请添加:"
        echo "    export PATH=\"$INSTALL_DIR:\$PATH\""
        echo ""
    fi
}

# 主流程
main() {
    echo -e "${GREEN}"
    cat << "EOF"
   ____ _____  _    ___
  / ___|  ___/ \  |_ _|
 | |   | |_ / _ \  | |
 | |___|  _/ ___ \ | |
  \____|_|/_/   \_\___|

  AI-Powered Cloudflare CLI
EOF
    echo -e "${NC}"

    detect_platform
    check_install_dir
    get_latest_release
    install_cfai
    verify_installation
}

main "$@"
