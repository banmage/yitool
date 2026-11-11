#!/usr/bin/env bash
# 构建桌面安装包并生成 SHA-256 校验和。
# 阶段 1 安装包未签名，仅用于本地/家庭部署验证；见 docs/adr/ADR-024-package-signing-release.md。
set -euo pipefail

cd "$(dirname "$0")/.."

bundle="${1:-deb}"
case "$bundle" in
  deb | nsis | msi | appimage | dmg) ;;
  *)
    echo "不支持的 bundle 类型：$bundle（可选：deb、nsis、msi、appimage、dmg）" >&2
    exit 2
    ;;
esac

npm --prefix apps/desktop run tauri -- build --bundles "$bundle"

artifact_dir="apps/desktop/src-tauri/target/release/bundle/$bundle"
if [[ ! -d "$artifact_dir" ]]; then
  echo "未找到产物目录：$artifact_dir" >&2
  exit 1
fi

shopt -s nullglob
files=("$artifact_dir"/*.deb "$artifact_dir"/*.exe "$artifact_dir"/*.msi "$artifact_dir"/*.AppImage "$artifact_dir"/*.dmg)
if (( ${#files[@]} == 0 )); then
  echo "产物目录中没有可校验的安装包：$artifact_dir" >&2
  exit 1
fi

(
  cd "$artifact_dir"
  sha256sum "${files[@]##*/}" > SHA256SUMS
  cat SHA256SUMS
)

echo "注意：SHA-256 只证明文件完整性，不证明来源真实性；阶段 1 安装包未签名。"
