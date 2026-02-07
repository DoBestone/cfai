#!/usr/bin/env bash
set -euo pipefail

# CFAI 一键安装脚本
# 自动下载并安装最新版本的 cfai

REPO="${CFAI_REPO:-DoBestone/cfai}"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

info() {
    echo -e "${GREEN}==>${NC} $1"
}

error() {
    echo -e "${RED}错误:${NC} $1" >&2
    exit 1
}

warn() {
    echo -e "${YELLOW}警告:${NC} $1" >&2
}

# 检测操作系统和架构
detect_platform() {
    local os=$(uname -s | tr '[:upper:]' '[:lower:]')
    local arch=$(uname -m)

    case "$os" in
        darwin) OS="macos" ;;
        linux) OS="linux" ;;
        msys*|mingw*|cygwin*|windows*) OS="windows" ;;
        *) error "不支持的操作系统: $os" ;;
    esac

    case "$arch" in
        x86_64|amd64) ARCH="x86_64" ;;
        arm64|aarch64) ARCH="aarch64" ;;
        *) error "不支持的架构: $arch" ;;
    esac

    info "检测到平台: $OS $ARCH"
}

# 检查安装目录权限
check_install_dir() {
    # 创建目录（如果不存在）
    if [[ ! -d "$INSTALL_DIR" ]]; then
        if mkdir -p "$INSTALL_DIR" 2>/dev/null; then
            return 0
        fi
    fi

    # 检查写入权限
    if [[ ! -w "$INSTALL_DIR" ]]; then
        warn "$INSTALL_DIR 无写入权限，将安装到 $HOME/.local/bin"
        INSTALL_DIR="$HOME/.local/bin"
        mkdir -p "$INSTALL_DIR"

        # 检查是否在 PATH 中
        if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
            warn "请将 $INSTALL_DIR 添加到 PATH 环境变量中："
            echo "    export PATH=\"\$HOME/.local/bin:\$PATH\""
        fi
    fi
}

# 获取最新版本信息
get_latest_release() {
    info "获取最新版本信息..."

    local api_url="https://api.github.com/repos/$REPO/releases/latest"
    local response

    if ! response=$(curl -fsSL "$api_url" 2>&1); then
        error "无法获取 Release 信息: $response"
    fi

    # 提取下载 URL
    DOWNLOAD_URL=$(echo "$response" | grep -o '"browser_download_url": *"[^"]*"' | head -1 | sed 's/"browser_download_url": *"\([^"]*\)"/\1/')
    VERSION=$(echo "$response" | grep -o '"tag_name": *"[^"]*"' | head -1 | sed 's/"tag_name": *"\([^"]*\)"/\1/')
    ASSET_NAME=$(echo "$response" | grep -o '"name": *"cfai[^"]*"' | head -1 | sed 's/"name": *"\([^"]*\)"/\1/')

    if [[ -z "$DOWNLOAD_URL" ]]; then
        error "无法找到可下载的 Release 资源"
    fi

    info "找到版本: $VERSION"
}

# 下载并安装
install_cfai() {
    local tmp_dir=$(mktemp -d)
    trap 'rm -rf "$tmp_dir"' EXIT

    local download_file="$tmp_dir/$ASSET_NAME"

    info "正在下载 cfai $VERSION..."
    if ! curl -fsSL "$DOWNLOAD_URL" -o "$download_file" --progress-bar; then
        error "下载失败"
    fi

    info "正在安装到 $INSTALL_DIR..."

    # 根据文件类型解压或直接复制
    local bin_file=""

    if [[ "$ASSET_NAME" == *.tar.gz ]] || [[ "$ASSET_NAME" == *.tgz ]]; then
        tar -xzf "$download_file" -C "$tmp_dir"
        bin_file=$(find "$tmp_dir" -type f \( -name "cfai" -o -name "cfai.exe" \) -print -quit)
    elif [[ "$ASSET_NAME" == *.zip ]]; then
        unzip -q "$download_file" -d "$tmp_dir"
        bin_file=$(find "$tmp_dir" -type f \( -name "cfai" -o -name "cfai.exe" \) -print -quit)
    else
        # 直接是二进制文件
        bin_file="$download_file"
    fi

    if [[ ! -f "$bin_file" ]]; then
        error "未找到可执行文件"
    fi

    # 安装
    if [[ -w "$INSTALL_DIR" ]]; then
        cp "$bin_file" "$INSTALL_DIR/cfai"
        chmod +x "$INSTALL_DIR/cfai"
    else
        sudo cp "$bin_file" "$INSTALL_DIR/cfai"
        sudo chmod +x "$INSTALL_DIR/cfai"
    fi

    info "安装成功！"
}

# 验证安装
verify_installation() {
    local cfai_path="$INSTALL_DIR/cfai"

    if [[ ! -f "$cfai_path" ]]; then
        error "安装文件不存在: $cfai_path"
    fi

    local installed_version=$("$cfai_path" --version 2>&1 | head -1)
    info "已安装版本: $installed_version"

    echo ""
    echo -e "${GREEN}🎉 CFAI 安装成功！${NC}"
    echo ""
    echo "安装位置: $cfai_path"
    echo ""
    echo "快速开始："
    echo "  1. 配置 API: cfai config setup"
    echo "  2. 验证配置: cfai config verify"
    echo "  3. 查看帮助: cfai --help"
    echo ""

    if ! command -v cfai >/dev/null 2>&1; then
        warn "cfai 不在 PATH 中，请手动添加："
        echo "    export PATH=\"$INSTALL_DIR:\$PATH\""
        echo ""
    fi

    echo "完整文档: https://github.com/$REPO"
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

  AI-Powered Cloudflare Management Tool
EOF
    echo -e "${NC}"

    detect_platform
    check_install_dir
    get_latest_release
    install_cfai
    verify_installation
}

main "$@"
