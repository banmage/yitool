#!/usr/bin/env python3
"""用香港天文台公历/农历对照表交叉校核 CAL 预计算数据。

来源：香港天文台（HKO）1901–2100 年公曆與農曆日期對照表
      https://www.hko.gov.hk/tc/gts/time/calendar/text/files/T{year}c.txt

用法：
    python3 scripts/crosscheck_cal_hko.py            # 下载缺失年份、校核、写报告
    python3 scripts/crosscheck_cal_hko.py --no-download  # 仅用本地缓存
    python3 scripts/crosscheck_cal_hko.py --years 2025 2026

默认缓存目录 target/cal-crosscheck/hko（不进入版本库）；默认报告
docs/reports/CAL-HKO-CROSSCHECK-v0.2.md。

HKO 在其对照表页脚声明：新月或节气时刻接近午夜零时（数分钟内）时，
不同高精度算法可能相差一日。脚本因此把“日期相差一日、且 CAL 交节时刻
距午夜不超过 --boundary-seconds（默认 900 秒）”的节气单独列为边界容差，
不计入严格不一致；其余差异一律判 FAIL。
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
import urllib.error
import urllib.request
from dataclasses import dataclass
from datetime import date, datetime, timedelta
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CAL_DIR = ROOT / "resources" / "cal"
DEFAULT_CACHE = ROOT / "target" / "cal-crosscheck" / "hko"
DEFAULT_REPORT = ROOT / "docs" / "reports" / "CAL-HKO-CROSSCHECK-v0.2.md"
URL = "https://www.hko.gov.hk/tc/gts/time/calendar/text/files/T{year}c.txt"
# HKO 声明的午夜边界容差：交节时刻距 00:00 不超过 15 分钟时允许日期差一日。
DEFAULT_BOUNDARY_SECONDS = 900

MONTH_NAMES = {
    "正": 1,
    "一": 1,
    "二": 2,
    "三": 3,
    "四": 4,
    "五": 5,
    "六": 6,
    "七": 7,
    "八": 8,
    "九": 9,
    "十": 10,
    "十一": 11,
    "十二": 12,
}
DAY_NAMES = {
    "初一": 1,
    "初二": 2,
    "初三": 3,
    "初四": 4,
    "初五": 5,
    "初六": 6,
    "初七": 7,
    "初八": 8,
    "初九": 9,
    "初十": 10,
    "十一": 11,
    "十二": 12,
    "十三": 13,
    "十四": 14,
    "十五": 15,
    "十六": 16,
    "十七": 17,
    "十八": 18,
    "十九": 19,
    "二十": 20,
    "廿一": 21,
    "廿二": 22,
    "廿三": 23,
    "廿四": 24,
    "廿五": 25,
    "廿六": 26,
    "廿七": 27,
    "廿八": 28,
    "廿九": 29,
    "三十": 30,
}
TERMS = [
    "小寒",
    "大寒",
    "立春",
    "雨水",
    "驚蟄",
    "春分",
    "清明",
    "穀雨",
    "立夏",
    "小滿",
    "芒種",
    "夏至",
    "小暑",
    "大暑",
    "立秋",
    "處暑",
    "白露",
    "秋分",
    "寒露",
    "霜降",
    "立冬",
    "小雪",
    "大雪",
    "冬至",
]
TERM_SET = set(TERMS)
TERM_TRAD_TO_SIMP = {
    "驚蟄": "惊蛰",
    "穀雨": "谷雨",
    "小滿": "小满",
    "芒種": "芒种",
    "處暑": "处暑",
    "白露": "白露",
    "寒露": "寒露",
    "霜降": "霜降",
    "小雪": "小雪",
    "大雪": "大雪",
    "冬至": "冬至",
    "小寒": "小寒",
    "大寒": "大寒",
    "立春": "立春",
    "雨水": "雨水",
    "春分": "春分",
    "清明": "清明",
    "立夏": "立夏",
    "夏至": "夏至",
    "小暑": "小暑",
    "大暑": "大暑",
    "立秋": "立秋",
    "秋分": "秋分",
    "立冬": "立冬",
}
LINE_RE = re.compile(
    r"^(\d{4})年(\d{1,2})月(\d{1,2})日\s+(\S+)\s+星期[一二三四五六日]\s*(\S*)\s*$"
)


@dataclass(frozen=True)
class HkoDay:
    gregorian: date
    lunar_year: int
    lunar_month: int
    lunar_day: int
    leap: bool
    term: str | None


def jdn_from_ymd(y: int, m: int, d: int) -> int:
    a = (14 - m) // 12
    y2 = y + 4800 - a
    m2 = m + 12 * a - 3
    return d + (153 * m2 + 2) // 5 + 365 * y2 + y2 // 4 - y2 // 100 + y2 // 400 - 32045


def ymd_from_jdn(jdn: int) -> date:
    a = jdn + 32044
    b = (4 * a + 3) // 146097
    c = a - 146097 * b // 4
    d = (4 * c + 3) // 1461
    e = c - 1461 * d // 4
    m = (5 * e + 2) // 153
    day = e - (153 * m + 2) // 5 + 1
    month = m + 3 - 12 * (m // 10)
    year = 100 * b + d - 4800 + m // 10
    return date(year, month, day)


def parse_lunar(token: str) -> tuple[int, bool] | None:
    """返回 (月份, 是否闰月)；非月首返回 None。"""
    if not token.endswith("月"):
        return None
    leap = token.startswith("閏")
    name = (token[1:] if leap else token).removesuffix("月")
    month = MONTH_NAMES.get(name)
    if month is None:
        return None
    return month, leap


def fetch_year(year: int, cache: Path, allow_download: bool) -> str:
    path = cache / f"T{year}c.txt"
    if path.exists() and path.stat().st_size > 0:
        return path.read_text(encoding="utf-8-sig")
    if not allow_download:
        raise SystemExit(f"缺少缓存文件：{path}（去掉 --no-download 或先联网下载）")
    cache.mkdir(parents=True, exist_ok=True)
    req = urllib.request.Request(
        URL.format(year=year),
        headers={"User-Agent": "yitool-cal-crosscheck/0.1 (+local verification)"},
    )
    try:
        with urllib.request.urlopen(req, timeout=30) as resp:
            raw = resp.read()
    except (urllib.error.URLError, TimeoutError) as exc:
        raise SystemExit(f"下载 {year} 年 HKO 表失败：{exc}") from exc
    if len(raw) < 1000:
        raise SystemExit(f"下载 {year} 年 HKO 表内容异常（{len(raw)} 字节）")
    path.write_bytes(raw)
    return raw.decode("utf-8-sig")


def parse_hko(text: str, year: int) -> list[HkoDay]:
    days: list[HkoDay] = []
    lunar_year: int | None = None
    current_month = 0
    current_leap = False
    for raw in text.splitlines():
        match = LINE_RE.match(raw.strip())
        if not match:
            continue
        gy, gm, gd, lunar_token, term = match.groups()
        g = date(int(gy), int(gm), int(gd))
        if g.year != year:
            continue
        parsed = parse_lunar(lunar_token)
        if parsed is not None:
            month, leap = parsed
            current_month, current_leap = month, leap
            if month == 1 and not leap:
                lunar_year = g.year
            elif lunar_year is None:
                lunar_year = g.year - 1
            day = 1
        else:
            if lunar_year is None:
                lunar_year = g.year - 1
            month, leap = current_month, current_leap
            if lunar_token not in DAY_NAMES:
                raise SystemExit(f"{g} 无法解析农历日：{lunar_token!r}")
            day = DAY_NAMES[lunar_token]
        term_name = TERM_TRAD_TO_SIMP.get(term) if term in TERM_SET else None
        days.append(HkoDay(g, lunar_year, month, day, leap, term_name))
    if len(days) < 300:
        raise SystemExit(f"{year} 年 HKO 表解析行数异常：{len(days)}")
    return days


def load_cal(cal_dir: Path) -> tuple[list[dict], list[dict], dict]:
    lunar = json.loads((cal_dir / "lunar_months.json").read_text(encoding="utf-8"))
    jieqi = json.loads((cal_dir / "jieqi.json").read_text(encoding="utf-8"))
    months = sorted(lunar["months"], key=lambda m: m["start"])
    for i, month in enumerate(months):
        month["_start_jdn"] = jdn_from_ymd(*map(int, month["start"].split("-")))
        end = date.fromisoformat(month["start"]) + timedelta(days=month["days"])
        month["_end"] = end
        if i + 1 < len(months) and months[i + 1]["start"] != end.isoformat():
            raise SystemExit(f"农历月表不连续：{month['start']} + {month['days']}")
    return months, jieqi["events"], lunar


def month_at(months: list[dict], day: date) -> dict | None:
    lo, hi = 0, len(months)
    while lo < hi:
        mid = (lo + hi) // 2
        month = months[mid]
        if date.fromisoformat(month["start"]) <= day < month["_end"]:
            return month
        if day < date.fromisoformat(month["start"]):
            hi = mid
        else:
            lo = mid + 1
    return None


def term_dates(events: list[dict]) -> dict[tuple[str, str], dict]:
    out = {}
    for event in events:
        local_date = event["local"][:10]
        out[(local_date, event["name"])] = event
    return out


def terms_by_name(events: list[dict]) -> dict[str, list[dict]]:
    out: dict[str, list[dict]] = {}
    for event in events:
        out.setdefault(event["name"], []).append(event)
    return out


def midnight_distance_seconds(local_iso: str) -> float:
    """交节时刻距最近午夜零时的秒数。"""
    t = datetime.fromisoformat(local_iso)
    seconds = t.hour * 3600 + t.minute * 60 + t.second + t.microsecond / 1_000_000
    return min(seconds, 86400.0 - seconds)


def boundary_match(
    hko_day: date,
    term: str,
    candidates: list[dict],
    boundary_seconds: int,
) -> dict | None:
    """查找 HKO 声明意义上的午夜边界对应（同日名、日期差一日、距午夜近）。"""
    if boundary_seconds <= 0:
        return None
    best: tuple[float, dict] | None = None
    for event in candidates:
        event_day = date.fromisoformat(event["local"][:10])
        if abs((event_day - hko_day).days) != 1:
            continue
        distance = midnight_distance_seconds(event["local"])
        if distance > boundary_seconds:
            continue
        if best is None or distance < best[0]:
            best = (distance, event)
    return best[1] if best else None


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--years", nargs="+", type=int, default=list(range(1901, 2101)))
    ap.add_argument("--cal-dir", type=Path, default=CAL_DIR)
    ap.add_argument("--cache", type=Path, default=DEFAULT_CACHE)
    ap.add_argument("--report", type=Path, default=DEFAULT_REPORT)
    ap.add_argument("--no-download", action="store_true")
    ap.add_argument("--max-details", type=int, default=80)
    ap.add_argument(
        "--boundary-seconds",
        type=int,
        default=DEFAULT_BOUNDARY_SECONDS,
        help="午夜边界容差秒数；0 表示禁用（严格逐日比对）",
    )
    args = ap.parse_args()

    years = sorted(set(args.years))
    months, events, lunar_meta = load_cal(args.cal_dir)
    our_terms = term_dates(events)
    cal_terms_by_name = terms_by_name(events)

    lunar_mismatches: list[tuple[HkoDay, dict]] = []
    term_mismatches: list[tuple[date, str, str]] = []
    hko_term_count = 0
    missing_terms: list[tuple[str, str]] = []
    boundary_terms: list[tuple[date, str, dict]] = []
    boundary_reverse: list[tuple[date, str, dict]] = []
    digest = hashlib.sha256()
    checked_days = 0
    hko_marks: set[tuple[str, str]] = set()
    hko_term_dates: dict[str, set[date]] = {}

    for index, year in enumerate(years, 1):
        text = fetch_year(year, args.cache, not args.no_download)
        digest.update(f"{year}:".encode())
        digest.update(text.encode())
        for hko in parse_hko(text, year):
            checked_days += 1
            month = month_at(months, hko.gregorian)
            if month is None:
                lunar_mismatches.append(
                    (
                        hko,
                        {
                            "year": "OUT_OF_RANGE",
                            "month": 0,
                            "leap": False,
                            "day": 0,
                            "start": "-",
                        },
                    )
                )
            else:
                actual = {
                    "year": month["year"],
                    "month": month["month"],
                    "leap": month["leap"],
                    "day": (hko.gregorian - date.fromisoformat(month["start"])).days + 1,
                    "start": month["start"],
                }
                if (
                    actual["year"] != hko.lunar_year
                    or (hko.lunar_month != 0 and actual["month"] != hko.lunar_month)
                    or (hko.lunar_month != 0 and actual["leap"] != hko.leap)
                    or actual["day"] != hko.lunar_day
                ):
                    lunar_mismatches.append((hko, actual))
            if hko.term:
                hko_term_count += 1
                key = (hko.gregorian.isoformat(), hko.term)
                hko_marks.add(key)
                hko_term_dates.setdefault(hko.term, set()).add(hko.gregorian)
                if key in our_terms:
                    continue
                neighbor = boundary_match(
                    hko.gregorian,
                    hko.term,
                    cal_terms_by_name.get(hko.term, []),
                    args.boundary_seconds,
                )
                if neighbor is not None:
                    boundary_terms.append((hko.gregorian, hko.term, neighbor))
                else:
                    term_mismatches.append(
                        (hko.gregorian, hko.term, "CAL 表中无此日期节气")
                    )
        if index % 20 == 0:
            print(f"  已校核 {index}/{len(years)} 年", file=sys.stderr)

    # 反向检查：CAL 在覆盖范围内标出的节气，HKO 是否也标出
    start, end = date(years[0], 1, 1), date(years[-1], 12, 31)
    for event in events:
        d = date.fromisoformat(event["local"][:10])
        if not (start <= d <= end):
            continue
        key = (d.isoformat(), event["name"])
        if key in hko_marks:
            continue
        near_midnight = (
            args.boundary_seconds > 0
            and midnight_distance_seconds(event["local"]) <= args.boundary_seconds
        )
        neighbor = next(
            (
                hko_day
                for hko_day in hko_term_dates.get(event["name"], set())
                if abs((hko_day - d).days) == 1
            ),
            None,
        )
        if near_midnight and neighbor is not None:
            boundary_reverse.append((d, event["name"], event))
        else:
            missing_terms.append((d.isoformat(), event["name"]))

    strict_pass = not lunar_mismatches and not term_mismatches and not missing_terms
    passed = strict_pass
    args.report.parent.mkdir(parents=True, exist_ok=True)
    lines = [
        "# CAL 交叉校核报告（香港天文台）",
        "",
        f"- **校核日期：** {date.today().isoformat()}",
        f"- **CAL 数据：** `{lunar_meta['version']}`（原状态 `{lunar_meta['status']}`）",
        f"- **权威来源：** 香港天文台《{years[0]}–{years[-1]} 年公曆與農曆日期對照表》",
        f"- **URL 模式：** `{URL.format(year='YYYY')}`",
        f"- **覆盖范围：** {years[0]}-01-01 ～ {years[-1]}-12-31，共 {len(years)} 年 / {checked_days} 天",
        f"- **HKO 缓存 SHA-256（按年份与内容）：** `{digest.hexdigest()}`",
        f"- **午夜边界容差：** {args.boundary_seconds} 秒",
        "",
        "## 结果",
        "",
        "| 项目 | HKO 样本 | 严格不一致 | 边界容差 | 结论 |",
        "|---|---:|---:|---:|---|",
        f"| 农历年/月/闰月/日 | {checked_days} | {len(lunar_mismatches)} | 0 | {'PASS' if not lunar_mismatches else 'FAIL'} |",
        f"| 节气日期（HKO → CAL） | {hko_term_count} | {len(term_mismatches)} | {len(boundary_terms)} | {'PASS' if not term_mismatches else 'FAIL'} |",
        f"| 节气日期（CAL → HKO） | {len(events)} | {len(missing_terms)} | {len(boundary_reverse)} | {'PASS' if not missing_terms else 'FAIL'} |",
        "",
        f"**总结论：** {'PASS' if passed else 'FAIL'}",
        "",
    ]
    if lunar_mismatches:
        lines += [
            "## 农历不一致明细（最多 %d 条）" % args.max_details,
            "",
            "| 公历 | HKO 农历 | CAL 农历 |",
            "|---|---|---|",
        ]
        for hko, actual in lunar_mismatches[: args.max_details]:
            expected_month = hko.lunar_month or actual["month"]
            expected_leap = hko.leap if hko.lunar_month else actual["leap"]
            expected = f"{hko.lunar_year}-{'闰' if expected_leap else ''}{expected_month:02d}-{hko.lunar_day:02d}"
            got = (
                f"{actual['year']}-{'闰' if actual['leap'] else ''}{actual['month']:02d}-{actual['day']:02d}"
                if actual["month"]
                else "越界"
            )
            lines.append(f"| {hko.gregorian} | {expected} | {got} |")
        lines.append("")
    if term_mismatches:
        lines += [
            "## 节气不一致明细（最多 %d 条）" % args.max_details,
            "",
            "| 日期 | 节气 | 说明 |",
            "|---|---|---|",
        ]
        for d, term, note in term_mismatches[: args.max_details]:
            lines.append(f"| {d} | {term} | {note} |")
        lines.append("")
    if missing_terms:
        lines += [
            "## CAL 有、HKO 未标记的节气（最多 %d 条）" % args.max_details,
            "",
            "| 日期 | 节气 |",
            "|---|---|",
        ]
        for d, term in missing_terms[: args.max_details]:
            lines.append(f"| {d} | {term} |")
        lines.append("")
    if boundary_terms:
        lines += [
            "## 午夜边界容差明细",
            "",
            "以下节气的 HKO 日期与 CAL 相差一日，但 CAL 交节时刻距午夜不超过 "
            f"{args.boundary_seconds} 秒，属 HKO 对照表页脚声明的算法边界情形，不计入严格不一致。",
            "",
            "| HKO 日期 | 节气 | CAL 交节时刻（+08:00） | 距午夜 |",
            "|---|---|---|---:|",
        ]
        for hko_day, term, event in boundary_terms[: args.max_details]:
            distance = midnight_distance_seconds(event["local"])
            lines.append(
                f"| {hko_day} | {term} | {event['local']} | {distance:.0f} 秒 |"
            )
        lines.append("")
    lines += [
        "## 说明",
        "",
        "- HKO 使用香港时间（UTC+8），与 CAL 的民用日锚 `+08:00` 一致，因此按日期直接比对。",
        "- HKO 表只标注节气所在日期，不标注交节时刻；时刻精度仍由 `CAL-EPHEMERIS` 与金标准保证。",
        "- 农历与节气数据由 sxtwl（VSOP87/ELP2000）独立计算，HKO 对照表为权威来源，二者构成“计算源 + 权威源”双源校核。",
        "- HKO 对照表页脚声明：新月或节气时刻很接近午夜零时（数分钟内）时，不同高精度算法可能相差一日。本报告把“日期差一日且 CAL 时刻距午夜 ≤ "
        f"{args.boundary_seconds} 秒”的条目单列为边界容差。",
        "- 本项目支持范围自 1900-01-31 起，1900 年不在 HKO 1901–2100 表内，未参与本次交叉校核。",
        "- 本报告由 `scripts/crosscheck_cal_hko.py` 生成，原始 HKO 表不进入版本库。",
        "",
    ]
    args.report.write_text("\n".join(lines), encoding="utf-8")
    print(
        f"覆盖 {checked_days} 天；农历不一致 {len(lunar_mismatches)}；"
        f"节气严格不一致 {len(term_mismatches)}；"
        f"边界容差 {len(boundary_terms)}；CAL 独有节气 {len(missing_terms)}"
    )
    try:
        shown = args.report.relative_to(ROOT)
    except ValueError:
        shown = args.report
    print(f"报告：{shown}")
    if not passed:
        sys.exit(1)


if __name__ == "__main__":
    main()
