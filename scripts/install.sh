#!/usr/bin/env bash
set -euo pipefail

# CFAI 安装/更新脚本
# 用法:
#   安装: curl -fsSL https://raw.githubusercontent.com/DoBestone/cfai/main/scripts/install.sh | bash
#   更新: curl -fsSL https://raw.githubusercontent.com/DoBestone/cfai/main/scripts/install.sh | bash -s -- --update

REPO="${CFAI_REPO:-DoBestone/cfai}"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
UPDATE_MODE=false

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

info() { echo -e "${GREEN}==>${NC} $1"; }
error() { echo -e "${RED}错误:${NC} $1" >&2; exit 1; }
warn() { echo -e "${YELLOW}警告:${NC} $1" >&2; }
step() { echo -e "${CYAN}[$1]${NC} $2"; }

# 解析参数
parse_args() {
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --update|-u) UPDATE_MODE=true; shift ;;
            --dir=*) INSTALL_DIR="${1#*=}"; shift ;;
            --help|-h)
                echo "CFAI 安装/更新脚本"
                echo ""
                echo "用法:"
                echo "  bash install.sh [选项]"
                echo ""
                echo "选项:"
                echo "  --update, -u    更新已安装的 cfai"
                echo "  --dir=PATH      指定安装目录 (默认: /usr/local/bin)"
                echo "  --help, -h      显示帮助"
                exit 0
                ;;
            *) shift ;;
        esac
    done
}

# 检测操作系统和架构
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

# 检查安装目录权限
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

        if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
            warn "请将 $INSTALL_DIR 添加到 PATH:"
            echo "    export PATH=\"\$HOME/.local/bin:\$PATH\""
        fi
    fi

    step "安装目录" "$INSTALL_DIR"
}

# 获取当前版本
get_current_version() {
    local cfai_path="$INSTALL_DIR/cfai"
    if [[ -f "$cfai_path" ]]; then
        CURRENT_VERSION=$("$cfai_path" --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1 || echo "")
        if [[ -n "$CURRENT_VERSION" ]]; then
            step "当前版本" "v$CURRENT_VERSION"
        fi
    else
        CURRENT_VERSION=""
    fi
}

# 获取最新版本信息
get_latest_release() {
    step "检查" "正在获取最新版本..."

    local api_url="https://api.github.com/repos/$REPO/releases/latest"
    local response

    if ! response=$(curl -fsSL "$api_url" 2>&1); then
        error "无法获取 Release 信息: $response"
    fi

    VERSION=$(echo "$response" | grep -o '"tag_name": *"[^"]*"' | head -1 | sed 's/"tag_name": *"\([^"]*\)"/\1/')

    # 根据平台选择正确的资源
    local pattern="cfai.*${OS}.*${ARCH}"
    DOWNLOAD_URL=$(echo "$response" | grep -o '"browser_download_url": *"[^"]*'"$pattern"'[^"]*"' | head -1 | sed 's/"browser_download_url": *"\([^"]*\)"/\1/')
    ASSET_NAME=$(basename "$DOWNLOAD_URL" 2>/dev/null || echo "")

    if [[ -z "$DOWNLOAD_URL" ]]; then
        # 回退：尝试获取任何可用的资源
        DOWNLOAD_URL=$(echo "$response" | grep -o '"browser_download_url": *"[^"]*cfai[^"]*"' | head -1 | sed 's/"browser_download_url": *"\([^"]*\)"/\1/')
        ASSET_NAME=$(basename "$DOWNLOAD_URL" 2>/dev/null || echo "")
    fi

    if [[ -z "$DOWNLOAD_URL" ]]; then
        error "未找到适合 $OS-$ARCH 的下载资源"
    fi

    step "最新版本" "$VERSION"

    # 检查是否需要更新
    local latest_ver="${VERSION#v}"
    if [[ -n "$CURRENT_VERSION" ]] && [[ "$CURRENT_VERSION" == "$latest_ver" ]]; then
        info "已是最新版本 ($VERSION)，无需更新"
        exit 0
    fi
}

# 下载并安装
install_cfai() {
    local tmp_dir=$(mktemp -d)
    trap 'rm -rf "$tmp_dir"' EXIT

    local download_file="$tmp_dir/$ASSET_NAME"

    step "下载" "$ASSET_NAME"
    if ! curl -fsSL "$DOWNLOAD_URL" -o "$download_file" --progress-bar; then
        error "下载失败"
    fi

    step "安装" "正在解压..."

    local bin_file=""

    if [[ "$ASSET_NAME" == *.tar.gz ]] || [[ "$ASSET_NAME" == *.tgz ]]; then
        tar -xzf "$download_file" -C "$tmp_dir"
        # 查找 cfai 可执行文件（支持 cfai, cfai-darwin-arm64 等格式）
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

    # 安装到目标目录
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
    if $UPDATE_MODE; then
        echo -e "${GREEN}  ✅ CFAI 更新成功！${NC}"
    else
        echo -e "${GREEN}  ✅ CFAI 安装成功！${NC}"
    fi
    echo -e "${GREEN}════════════════════════════════════════${NC}"
    echo ""
    echo "  版本: $installed_version"
    echo "  路径: $cfai_path"
    echo ""

    if ! $UPDATE_MODE; then
        echo "快速开始:"
        echo "  cfai config setup    配置 API"
        echo "  cfai zone list       列出域名"
        echo "  cfai --help          查看帮助"
        echo ""
    fi

    if ! command -v cfai >/dev/null 2>&1; then
        warn "cfai 不在 PATH 中，请添加:"
        echo "    export PATH=\"$INSTALL_DIR:\$PATH\""
        echo ""
    fi
}

# 主流程
main() {
    parse_args "$@"

    if ! $UPDATE_MODE; then
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
    else
        echo ""
        info "CFAI 更新检查"
        echo ""
    fi

    detect_platform
    check_install_dir
    get_current_version
    get_latest_release
    install_cfai
    verify_installation
}

main "$@"
