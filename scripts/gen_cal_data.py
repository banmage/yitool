#!/usr/bin/env python3
"""生成 CAL 预计算表与逐日基准的统一入口。

农历月表与节气时刻表由 `scripts/gen_cal_precise.py`（sxtwl）生成；逐日基准
由 `tools/gen-cal` 从已校核的预计算表派生。校核流程见
`docs/reports/CAL-HKO-CROSSCHECK-v0.2.md`。

用法：
    .venv-cal/bin/python scripts/gen_cal_data.py cal
    .venv-cal/bin/python scripts/gen_cal_data.py cal --status VERIFIED
    .venv-cal/bin/python scripts/gen_cal_data.py baseline
    .venv-cal/bin/python scripts/gen_cal_data.py all

`cal` 模式必须使用安装了 sxtwl 的 Python 解释器；`--status VERIFIED` 只能在
香港天文台逐日交叉校核通过后使用。
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def main() -> None:
    mode = sys.argv[1] if len(sys.argv) > 1 else "all"
    extra = sys.argv[2:]
    if mode not in {"cal", "baseline", "all"}:
        raise SystemExit("模式必须是 cal|baseline|all")

    if mode in {"cal", "all"}:
        subprocess.run(
            [sys.executable, str(ROOT / "scripts" / "gen_cal_precise.py"), *extra],
            cwd=ROOT,
            check=True,
        )

    if mode in {"baseline", "all"}:
        subprocess.run(
            [
                "cargo",
                "run",
                "--quiet",
                "-p",
                "gen-cal",
                "--",
                "baseline",
                str(ROOT / "resources" / "cal"),
            ],
            cwd=ROOT,
            check=True,
        )


if __name__ == "__main__":
    main()
