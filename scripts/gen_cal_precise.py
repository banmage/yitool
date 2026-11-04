#!/usr/bin/env python3
"""高精度生成 CAL 农历月表与二十四节气时刻表。

天文历算使用 sxtwl（寿星万年历，VSOP87/ELP2000）。农历月首与节气时刻
均按中国标准时间 UTC+8 的民用日锚输出，与香港天文台《1901–2100 年
公曆與農曆日期對照表》一致，便于逐日交叉校核。

香港天文台说明：新月或节气时刻非常接近午夜零时（约数分钟内）时，
不同高精度算法可能出现一日之差，这属于权威源已声明的边界容差。

用法：
    python3 -m venv .venv-cal
    .venv-cal/bin/pip install -r scripts/requirements-cal.txt
    .venv-cal/bin/python scripts/gen_cal_precise.py
    .venv-cal/bin/python scripts/gen_cal_precise.py --status VERIFIED

生成 resources/cal/lunar_months.json 与 resources/cal/jieqi.json。
默认只写 PENDING；仅在香港天文台逐日交叉校核通过后使用 `--status VERIFIED`。
"""

from __future__ import annotations

import argparse
import json
from collections import Counter
from datetime import date, datetime, timedelta, timezone
from importlib.metadata import version
from pathlib import Path

try:
    import sxtwl
except ImportError:  # pragma: no cover - 给出可操作提示
    raise SystemExit(
        "缺少 sxtwl。请先创建虚拟环境：\n"
        "  python3 -m venv .venv-cal\n"
        "  .venv-cal/bin/pip install -r scripts/requirements-cal.txt"
    )

ROOT = Path(__file__).resolve().parent.parent
DEFAULT_OUT = ROOT / "resources" / "cal"
VERSION = "CAL-DATA-v0.2"
GENERATED_AT = "2026-11-04"

RANGE_START = date(1900, 1, 31)
RANGE_END = date(2100, 12, 31)
JIEQI_FIRST_YEAR = 1899
JIEQI_LAST_YEAR = 2101

CST = timezone(timedelta(hours=8))

# CAL 内部节气索引：0=小寒、2=立春、23=冬至（偶数索引为“节”）。
TERMS: list[str] = [
    "小寒",
    "大寒",
    "立春",
    "雨水",
    "惊蛰",
    "春分",
    "清明",
    "谷雨",
    "立夏",
    "小满",
    "芒种",
    "夏至",
    "小暑",
    "大暑",
    "立秋",
    "处暑",
    "白露",
    "秋分",
    "寒露",
    "霜降",
    "立冬",
    "小雪",
    "大雪",
    "冬至",
]
TERM_INDEX = {name: index for index, name in enumerate(TERMS)}

# sxtwl 的节气索引：0=冬至、1=小寒、……、23=大雪。
SXTWL_TERMS: list[str] = ["冬至", *TERMS[:23]]


def local_from_jd(jd: float) -> tuple[datetime, datetime]:
    """把 sxtwl 的 JD 转成 UTC+8 民用时刻与 UTC 时刻（秒级）。"""
    t = sxtwl.JD2DD(jd)
    base = datetime(t.Y, t.M, t.D, int(t.h), int(t.m), 0, tzinfo=CST)
    local = base + timedelta(seconds=round(float(t.s)))
    return local, local.astimezone(timezone.utc)


def build_lunar() -> list[dict]:
    """逐日读取 sxtwl 农历，提取 1900-01-31～2100-12-31 的月首记录。"""
    months: list[dict] = []
    current: dict | None = None
    current_start: date | None = None

    day = RANGE_START
    while day <= RANGE_END:
        lunar = sxtwl.fromSolar(day.year, day.month, day.day)
        if lunar.getLunarDay() == 1:
            if current is not None and current_start is not None:
                current["days"] = (day - current_start).days
                months.append(current)
            current_start = day
            current = {
                "start": day.isoformat(),
                "year": lunar.getLunarYear(),
                "month": lunar.getLunarMonth(),
                "leap": bool(lunar.isLunarLeap()),
                "days": 0,
            }
        day += timedelta(days=1)

    if current is None or current_start is None:
        raise RuntimeError(f"{RANGE_START} 不是农历月首，无法生成月表")

    # 最后一个月的月长需要看 2100-12-31 之后的下一个月首。
    next_start = RANGE_END + timedelta(days=1)
    while True:
        lunar = sxtwl.fromSolar(next_start.year, next_start.month, next_start.day)
        if lunar.getLunarDay() == 1:
            break
        next_start += timedelta(days=1)
    current["days"] = (next_start - current_start).days
    months.append(current)

    if months[0]["start"] != RANGE_START.isoformat():
        raise RuntimeError(f"月表首条不是 {RANGE_START}: {months[0]['start']}")
    for month in months:
        if month["days"] not in (29, 30):
            raise RuntimeError(f"农历月 {month['start']} 天数异常: {month['days']}")
    for previous, following in zip(months, months[1:]):
        expected = date.fromisoformat(previous["start"]) + timedelta(
            days=previous["days"]
        )
        if expected.isoformat() != following["start"]:
            raise RuntimeError(
                f"月表不连续: {previous['start']} + {previous['days']} != {following['start']}"
            )
    return months


def build_jieqi() -> list[dict]:
    """生成 1899–2101 年二十四节气时刻（UTC+8 民用时与 UTC）。"""
    events: dict[float, dict] = {}
    for year in range(JIEQI_FIRST_YEAR - 1, JIEQI_LAST_YEAR + 2):
        for item in sxtwl.getJieQiByYear(year):
            if item.jqIndex < 0 or item.jqIndex >= len(SXTWL_TERMS):
                raise RuntimeError(f"{year} 年节气索引异常: {item.jqIndex}")
            local, utc = local_from_jd(item.jd)
            if not (JIEQI_FIRST_YEAR <= local.year <= JIEQI_LAST_YEAR):
                continue
            name = SXTWL_TERMS[item.jqIndex]
            events[round(float(item.jd), 6)] = {
                "term": TERM_INDEX[name],
                "name": name,
                "utc": utc.strftime("%Y-%m-%dT%H:%M:%SZ"),
                "local": local.isoformat(timespec="seconds"),
                "jd_ut": round(float(item.jd), 9),
            }

    counts = Counter(event["local"][:4] for event in events.values())
    bad_years = [
        year
        for year in range(JIEQI_FIRST_YEAR, JIEQI_LAST_YEAR + 1)
        if counts[str(year)] != 24
    ]
    if bad_years:
        raise RuntimeError(f"以下年份节气数量不是 24: {bad_years[:10]}")

    return sorted(events.values(), key=lambda event: (event["utc"], event["term"]))


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--out", type=Path, default=DEFAULT_OUT)
    ap.add_argument(
        "--status",
        choices=("PENDING", "VERIFIED"),
        default="PENDING",
        help="资源状态；仅在校核通过后使用 VERIFIED",
    )
    args = ap.parse_args()

    months = build_lunar()
    events = build_jieqi()

    common = {
        "version": VERSION,
        "status": args.status,
        "reference_tz": "+08:00",
        "source": (
            f"sxtwl {version('sxtwl')} (寿星万年历); "
            "generated by scripts/gen_cal_precise.py"
        ),
        "generated_at": GENERATED_AT,
    }

    args.out.mkdir(parents=True, exist_ok=True)
    (args.out / "lunar_months.json").write_text(
        json.dumps({**common, "months": months}, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )
    (args.out / "jieqi.json").write_text(
        json.dumps({**common, "events": events}, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )
    print(f"农历月：{len(months)}，节气：{len(events)}")
    print(f"输出：{args.out}")


if __name__ == "__main__":
    main()
