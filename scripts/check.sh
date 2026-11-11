#!/usr/bin/env bash
# 阶段 0 质量门：格式、lint、测试、资源校验、隐私扫描。
set -euo pipefail

cd "$(dirname "$0")/.."

source "$HOME/.cargo/env" 2>/dev/null || true

cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

# 桌面命令层是独立 workspace，不随 --workspace 覆盖。
cargo fmt --manifest-path apps/desktop/src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path apps/desktop/src-tauri/Cargo.toml --no-default-features --all-targets -- -D warnings
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml --no-default-features

if [[ -d apps/desktop/node_modules ]]; then
  npm --prefix apps/desktop test
  npm --prefix apps/desktop run build
else
  echo "check.sh: 跳过前端检查（apps/desktop/node_modules 不存在，先运行 npm ci）"
fi

cargo run --quiet -p yitool-cli -- verify --resources resources
rm -f /tmp/yitool-check.db /tmp/yitool-check.db-wal /tmp/yitool-check.db-shm
cargo run --quiet -p yitool-cli -- db init --path /tmp/yitool-check.db
cargo run --quiet -p yitool-cli -- privacy-scan --db /tmp/yitool-check.db --needle "1966-06-06 06:06" --needle "测试问事原文"

echo "check.sh: OK"
