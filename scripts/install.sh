#!/usr/bin/env bash
set -euo pipefail

REPO="${CFAI_REPO:-DoBestone/cfai}"
PREFIX="${PREFIX:-/usr/local}"
BIN_DIR="${BIN_DIR:-$PREFIX/bin}"

OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$OS" in
  darwin) OS="macos";;
  linux) OS="linux";;
  msys*|mingw*|cygwin*) OS="windows";;
esac

case "$ARCH" in
  x86_64|amd64) ARCH="x86_64";;
  arm64|aarch64) ARCH="aarch64";;
esac

if [[ ! -w "$BIN_DIR" ]]; then
  BIN_DIR="$HOME/.local/bin"
fi

mkdir -p "$BIN_DIR"

release_json="$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest")"

asset_url=""
asset_name=""

if command -v python3 >/dev/null 2>&1; then
  read -r asset_name asset_url < <(TARGET_OS="$OS" TARGET_ARCH="$ARCH" python3 - <<PY
import json, os, re, sys

data = json.loads(sys.stdin.read())
os_pattern = os.environ.get("TARGET_OS", "").lower()
arch_pattern = os.environ.get("TARGET_ARCH", "").lower()

assets = data.get("assets", [])

def match(a):
    name = a.get("name", "").lower()
    if "cfai" not in name:
        return False
    if os_pattern and os_pattern not in name:
        return False
    if arch_pattern and arch_pattern not in name:
        return False
    return True

candidates = [a for a in assets if match(a)] or [a for a in assets if "cfai" in a.get("name", "").lower()]
if not candidates:
    sys.exit(1)

best = max(candidates, key=lambda a: a.get("size", 0))
print(best.get("name", ""))
print(best.get("browser_download_url", ""))
PY
  )
else
  asset_url="$(echo "$release_json" | grep -Eo 'https://[^\"]+' | grep -i 'cfai' | grep -i "$OS" | grep -i "$ARCH" | head -n 1)"
  asset_name="$(basename "$asset_url")"
fi

if [[ -z "$asset_url" ]]; then
  echo "无法找到适配的 Release 资源，请手动下载。"
  exit 1
fi

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

curl -fsSL "$asset_url" -o "$tmp_dir/$asset_name"

bin_path=""
if [[ "$asset_name" == *.tar.gz || "$asset_name" == *.tgz ]]; then
  tar -xzf "$tmp_dir/$asset_name" -C "$tmp_dir"
  bin_path="$(find "$tmp_dir" -type f \( -name cfai -o -name cfai.exe \) | head -n 1)"
elif [[ "$asset_name" == *.zip ]]; then
  unzip -q "$tmp_dir/$asset_name" -d "$tmp_dir"
  bin_path="$(find "$tmp_dir" -type f \( -name cfai -o -name cfai.exe \) | head -n 1)"
else
  bin_path="$tmp_dir/$asset_name"
fi

if [[ ! -f "$bin_path" ]]; then
  echo "未找到可执行文件。"
  exit 1
fi

install -m 755 "$bin_path" "$BIN_DIR/cfai"

echo "✅ 已安装到 $BIN_DIR/cfai"
