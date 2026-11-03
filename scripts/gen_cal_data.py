#!/usr/bin/env python3
"""生成 CAL 预计算表与逐日基准。

实际天文计算在 Rust 工具 tools/gen-cal 中完成（依赖 mingli-astro），
此脚本仅提供统一入口，便于文档/CI 复现。

用法：
    python3 scripts/gen_cal_data.py [cal|baseline|all]
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def main() -> None:
    mode = sys.argv[1] if len(sys.argv) > 1 else "all"
    if mode not in {"cal", "baseline", "all"}:
        raise SystemExit("模式必须是 cal|baseline|all")
    cmd = [
        "cargo",
        "run",
        "--quiet",
        "-p",
        "gen-cal",
        "--",
        mode,
        str(ROOT / "resources" / "cal"),
    ]
    subprocess.run(cmd, cwd=ROOT, check=True)


if __name__ == "__main__":
    main()
