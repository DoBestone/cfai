#!/usr/bin/env bash
set -euo pipefail

# CFAI 一键更新脚本
# 用法: curl -fsSL https://raw.githubusercontent.com/DoBestone/cfai/main/scripts/update.sh | bash

REPO="${CFAI_REPO:-DoBestone/cfai}"
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
    # 测试直连 GitHub 速度
    local test_time=$(curl -o /dev/null -s -w '%{time_total}' --connect-timeout 3 "https://github.com" 2>/dev/null || echo "999")
    if (( $(echo "$test_time > 2" | bc -l 2>/dev/null || echo 1) )); then
        return 0  # 需要镜像
    fi
    return 1  # 不需要镜像
}

# 查找已安装的 cfai
find_cfai() {
    if command -v cfai >/dev/null 2>&1; then
        CFAI_PATH=$(command -v cfai)
        INSTALL_DIR=$(dirname "$CFAI_PATH")
    elif [[ -f "/usr/local/bin/cfai" ]]; then
        CFAI_PATH="/usr/local/bin/cfai"
        INSTALL_DIR="/usr/local/bin"
    elif [[ -f "$HOME/.local/bin/cfai" ]]; then
        CFAI_PATH="$HOME/.local/bin/cfai"
        INSTALL_DIR="$HOME/.local/bin"
    else
        error "未找到已安装的 cfai，请先安装: curl -fsSL https://raw.githubusercontent.com/DoBestone/cfai/main/scripts/install.sh | bash"
    fi

    step "已安装位置" "$CFAI_PATH"
}

# 获取当前版本
get_current_version() {
    CURRENT_VERSION=$("$CFAI_PATH" --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1 || echo "unknown")
    step "当前版本" "v$CURRENT_VERSION"
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
    local latest_ver="${VERSION#v}"

    step "最新版本" "$VERSION"

    # 检查是否需要更新
    if [[ "$CURRENT_VERSION" == "$latest_ver" ]]; then
        echo ""
        info "✅ 已是最新版本，无需更新"
        exit 0
    fi

    # 获取下载 URL
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
}

# 下载并更新
update_cfai() {
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
        # 镜像失败，尝试直连
        if [[ "$final_url" != "$DOWNLOAD_URL" ]]; then
            warn "镜像下载失败，尝试直连..."
            if ! curl -fSL "$DOWNLOAD_URL" -o "$download_file" --progress-bar; then
                error "下载失败"
            fi
        else
            error "下载失败"
        fi
    fi

    step "更新" "正在安装..."

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

    # 替换二进制
    if [[ -w "$INSTALL_DIR" ]]; then
        cp "$bin_file" "$CFAI_PATH"
        chmod +x "$CFAI_PATH"
    else
        sudo cp "$bin_file" "$CFAI_PATH"
        sudo chmod +x "$CFAI_PATH"
    fi
}

# 验证更新
verify_update() {
    local new_version=$("$CFAI_PATH" --version 2>&1 | head -1)

    echo ""
    echo -e "${GREEN}════════════════════════════════════════${NC}"
    echo -e "${GREEN}  ✅ CFAI 更新成功！${NC}"
    echo -e "${GREEN}════════════════════════════════════════${NC}"
    echo ""
    echo "  v$CURRENT_VERSION → $VERSION"
    echo "  路径: $CFAI_PATH"
    echo ""
}

# 主流程
main() {
    echo ""
    info "CFAI 更新检查"
    echo ""

    find_cfai
    get_current_version
    detect_platform
    get_latest_release
    update_cfai
    verify_update
}

main "$@"
