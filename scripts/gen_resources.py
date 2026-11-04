#!/usr/bin/env python3
"""从 ARS/SRS 文档生成只读 JSON 资源。

只使用 Python 标准库；输出稳定排序，便于 checksum 复现。
不修改文档，不把文档中的待冻结项升级为 FROZEN。
"""

from __future__ import annotations

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DOCS = ROOT / "docs"
ARS = DOCS / "易学算法与规则规范 ARS v0.2.txt"
RES = ROOT / "resources"

FIVE_ELEMENTS = ["木", "火", "土", "金", "水"]


def read_ars() -> str:
    return ARS.read_text(encoding="utf-8")


def write_json(rel: str, doc: dict) -> None:
    path = RES / rel
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        json.dumps(doc, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    print(f"wrote {path.relative_to(ROOT)}")


def gen_color_num() -> None:
    nine = [
        (1, "赤色", "火", 1),
        (2, "橙色", "土", 2),
        (3, "黄色", "土", 3),
        (4, "绿色", "木", 4),
        (5, "青色", "木", 5),
        (6, "蓝色", "水", 6),
        (7, "紫色", "金", 7),
        (8, "黑色", "水", 8),
        (9, "白色", "金", 9),
    ]
    write_json(
        "data/color_num.json",
        {
            "version": "COLOR-NUM-02",
            "status": "VERIFIED",
            "source": "ARS v0.2 1.1 / ADR-002",
            "description": "九色为系统自定义扩展象数体系，非传统梅花原典标准；九色为唯一标准输入。",
            "nine_color": [
                {"color_id": i, "name": n, "element": e, "xiangshu": x}
                for i, n, e, x in nine
            ],
        },
    )


def gen_zodiac() -> None:
    rows = [
        (1, "鼠", "子", 1, "水", 1),
        (2, "牛", "丑", 2, "土", 2),
        (3, "虎", "寅", 3, "木", 3),
        (4, "兔", "卯", 4, "木", 4),
        (5, "龙", "辰", 5, "土", 5),
        (6, "蛇", "巳", 6, "火", 6),
        (7, "马", "午", 7, "火", 7),
        (8, "羊", "未", 8, "土", 8),
        (9, "猴", "申", 9, "金", 9),
        (10, "鸡", "酉", 10, "金", 10),
        (11, "狗", "戌", 11, "土", 11),
        (12, "猪", "亥", 12, "水", 12),
    ]
    write_json(
        "data/zodiac.json",
        {
            "version": "Z-01",
            "status": "VERIFIED",
            "source": "ARS v0.2 1.3",
            "items": [
                {
                    "zodiac_id": i,
                    "name": n,
                    "branch": b,
                    "branch_id": bi,
                    "element": e,
                    "xiangshu": x,
                }
                for i, n, b, bi, e, x in rows
            ],
        },
    )


def gen_bagua() -> None:
    trigrams = [
        ("QIAN", 1, "乾", "☰", "金", [1, 1, 1], "天"),
        ("DUI", 2, "兑", "☱", "金", [1, 1, 0], "泽"),
        ("LI", 3, "离", "☲", "火", [1, 0, 1], "火"),
        ("ZHEN", 4, "震", "☳", "木", [1, 0, 0], "雷"),
        ("XUN", 5, "巽", "☴", "木", [0, 1, 1], "风"),
        ("KAN", 6, "坎", "☵", "水", [0, 1, 0], "水"),
        ("GEN", 7, "艮", "☶", "土", [0, 0, 1], "山"),
        ("KUN", 8, "坤", "☷", "土", [0, 0, 0], "地"),
    ]
    write_json(
        "data/bagua.json",
        {
            "version": "G-01",
            "status": "VERIFIED",
            "source": "ARS v0.2 附录 A / 先天八卦",
            "trigrams": [
                {
                    "key": k,
                    "number": n,
                    "name": name,
                    "symbol": sym,
                    "element": e,
                    "lines_bottom_up": lines,
                    "image": image,
                }
                for k, n, name, sym, e, lines, image in trigrams
            ],
        },
    )


def parse_hex64() -> list[dict]:
    text = read_ars()
    start = text.index("## 附录 A：六十四卦八宫表")
    end = text.index("## 附录 B：纳甲干支表")
    rows = []
    pattern = re.compile(
        r"^\|\s*(乾|兑|离|震|巽|坎|艮|坤)\s*\|\s*(\d)\s*\|\s*([^|]+?)\s*\|\s*"
        r"(乾|兑|离|震|巽|坎|艮|坤)\s*\|\s*(乾|兑|离|震|巽|坎|艮|坤)\s*\|"
        r"\s*([A-Z]+)\s*\|\s*(\d)\s*\|\s*(\d)\s*\|"
    )
    for line in text[start:end].splitlines():
        match = pattern.match(line.strip())
        if match:
            palace, seq, name, upper, lower, kind, shi, ying = match.groups()
            rows.append(
                {
                    "palace": palace,
                    "palace_seq": int(seq),
                    "name": name,
                    "upper": upper,
                    "lower": lower,
                    "palace_kind": kind,
                    "shi_yao": int(shi),
                    "ying_yao": int(ying),
                }
            )
    if len(rows) != 64:
        raise SystemExit(f"HEX-01 解析失败：得到 {len(rows)} 条，应为 64")
    return rows


def gen_hex64() -> None:
    rows = parse_hex64()
    write_json(
        "data/hex64.json",
        {
            "version": "HEX-01",
            "status": "PENDING",
            "source": "ARS v0.2 附录 A（待交叉校核）",
            "hexagrams": rows,
        },
    )


def parse_najia() -> list[dict]:
    text = read_ars()
    start = text.index("## 附录 B：纳甲干支表")
    end = text.index("## 附录 C：规则 ID 索引")
    rows = []
    pattern = re.compile(
        r"^\|\s*(乾|坤|震|巽|坎|离|艮|兑)\s*\|(.+?)\|$"
    )
    for line in text[start:end].splitlines():
        match = pattern.match(line.strip())
        if match:
            trigram, rest = match.groups()
            cells = [c.strip() for c in rest.split("|")]
            if len(cells) != 6:
                continue
            rows.append({"trigram": trigram, "lines": cells})
    if len(rows) != 8:
        raise SystemExit(f"NJ-01 解析失败：得到 {len(rows)} 条，应为 8")
    return rows


def gen_najia() -> None:
    write_json(
        "data/najia.json",
        {
            "version": "NJ-01",
            "status": "PENDING",
            "source": "ARS v0.2 附录 B（待校核）",
            "trigrams": parse_najia(),
        },
    )


def gen_xun() -> None:
    stems = ["甲", "乙", "丙", "丁", "戊", "己", "庚", "辛", "壬", "癸"]
    branches = ["子", "丑", "寅", "卯", "辰", "巳", "午", "未", "申", "酉", "戌", "亥"]
    items = []
    for start in range(0, 60, 10):
        head = f"{stems[0]}{branches[start % 12]}"
        kong = [branches[(start + 10) % 12], branches[(start + 11) % 12]]
        items.append(
            {
                "xun_head": head,
                "cycle_start": start,
                "kong_branches": kong,
            }
        )
    write_json(
        "data/xun.json",
        {
            "version": "XK-01",
            "status": "VERIFIED",
            "source": "ARS v0.2 / 六十甲子旬空",
            "items": items,
        },
    )


def gen_wuxing_wangshuai() -> None:
    branches = [
        ("子", 0, "水"),
        ("丑", 1, "土"),
        ("寅", 2, "木"),
        ("卯", 3, "木"),
        ("辰", 4, "土"),
        ("巳", 5, "火"),
        ("午", 6, "火"),
        ("未", 7, "土"),
        ("申", 8, "金"),
        ("酉", 9, "金"),
        ("戌", 10, "土"),
        ("亥", 11, "水"),
    ]
    write_json(
        "data/wuxing_wangshuai.json",
        {
            "version": "W-01",
            "status": "PENDING",
            "source": "ARS v0.2 3.7（待 CAL/ARS 校核冻结）",
            "relation_rule": "同我旺、生我相、我生休、我克囚、克我死",
            "branches": [
                {"branch": b, "branch_id": i, "element": e} for b, i, e in branches
            ],
            "relations": ["旺", "相", "休", "囚", "死"],
        },
    )


def gen_rules() -> None:
    sources = [
        ("SRC-MEIHUA-001", "梅花易数", "传世本", "起卦", "九色/传统数字起卦公式", "系统按 ARS v0.2 重述", "L2", "PENDING", "公有领域"),
        ("SRC-MEIHUA-002", "梅花易数", "传世本", "互卦", "取中间四爻", "系统按 ARS v0.2 重述", "L2", "PENDING", "公有领域"),
        ("SRC-MEIHUA-003", "梅花易数", "传世本", "变卦", "动爻阴阳互变", "系统按 ARS v0.2 重述", "L2", "PENDING", "公有领域"),
        ("SRC-MEIHUA-004", "梅花易数", "传世本", "体用", "动爻定体用", "系统按 ARS v0.2 重述", "L2", "PENDING", "公有领域"),
        ("SRC-MEIHUA-005", "梅花易数", "传世本", "旺衰", "月建五行生克", "W-01 待冻结", "L2", "PENDING", "公有领域"),
        ("SRC-MEIHUA-006", "本系统自定义", "v0.1", "忌神", "未定义算法", "阶段 0 阻断正式结果", "PROVISIONAL", "PENDING", "系统自定义"),
        ("SRC-CAL-001", "历法服务规范 CAL v0.1", "v0.1", "历法", "公农历/节气/四柱/日界/旬空", "CAL 数据待权威历书校核", "L1", "PENDING", "系统文档"),
        ("SRC-SEED-001", "技术设计说明书 TDS v0.1", "v0.1", "种子链", "SHA-256 派生与 HMAC", "阶段 0 实现", "L2", "PENDING", "系统文档"),
        ("SRC-IR-001", "推演解释规则规范 IR v0.1", "v0.1", "合规", "免责声明与禁用绝对断语", "阶段 0 最小实现", "L1", "PENDING", "系统文档"),
        ("SRC-LOG-001", "推演解释规则规范 IR v0.1", "v0.1", "日志", "步骤日志字段", "阶段 0 最小实现", "L1", "PENDING", "系统文档"),
    ]

    def rule(rule_id, engine, category, level, source, status, tests, result_type="INFORMATION", note=""):
        return {
            "rule_id": rule_id,
            "engine": engine,
            "category": category,
            "rule_level": level,
            "condition": {"note": note} if note else {},
            "result_type": result_type,
            "weight": None,
            "priority": None,
            "source_ref": source,
            "version": "RULE-v0.2",
            "status": status,
            "test_refs": tests,
        }

    rules = [
        rule("SEED-BASE-001", "SEED", "base", "L2", "SRC-SEED-001", "PENDING", ["TC-SEED-001"]),
        rule("SEED-DRAW-001", "SEED", "draw", "L2", "SRC-SEED-001", "PENDING", ["TC-SEED-001"]),
        rule("SEED-SNAPSHOT-001", "SEED", "snapshot", "L2", "SRC-SEED-001", "PENDING", ["TC-SEED-002"]),
        rule("SEED-RUN-001", "SEED", "run", "L2", "SRC-SEED-001", "PENDING", ["TC-SEED-001"]),
        rule("RNG-UNIFORM-001", "RNG", "uniform", "L2", "SRC-SEED-001", "PENDING", ["TC-RNG-001"]),
        rule("CAL-LUNAR-001", "CAL", "lunar", "L1", "SRC-CAL-001", "PENDING", ["TC-CAL-001", "TC-CAL-002"]),
        rule("CAL-JIEQI-001", "CAL", "jieqi", "L1", "SRC-CAL-001", "PENDING", ["TC-CAL-003"]),
        rule("CAL-DAYBOUND-001", "CAL", "day_bound", "L1", "SRC-CAL-001", "PENDING", ["TC-CAL-004"]),
        rule("CAL-XUN-001", "CAL", "xun", "L1", "SRC-CAL-001", "PENDING", ["TC-CAL-005"]),
        rule("MEIHUA-QIGUA-001", "MEIHUA", "qigua", "L3", "SRC-MEIHUA-001", "PENDING", ["TC-MEIHUA-001"]),
        rule("MEIHUA-QIGUA-002", "MEIHUA", "qigua", "L2", "SRC-MEIHUA-001", "PENDING", ["TC-MEIHUA-002"]),
        rule("MEIHUA-HUGUA-001", "MEIHUA", "hugua", "L2", "SRC-MEIHUA-002", "PENDING", ["TC-MEIHUA-001"]),
        rule("MEIHUA-BIANGUA-001", "MEIHUA", "biangua", "L2", "SRC-MEIHUA-003", "PENDING", ["TC-MEIHUA-001", "TC-MEIHUA-002"]),
        rule("MEIHUA-TIYONG-001", "MEIHUA", "tiyong", "L2", "SRC-MEIHUA-004", "PENDING", ["TC-MEIHUA-001"]),
        rule("MEIHUA-WANGSHUAI-001", "MEIHUA", "wangshuai", "L2", "SRC-MEIHUA-005", "PENDING", ["TC-MEIHUA-003"]),
        rule("MEIHUA-JISHEN-001", "MEIHUA", "jishen", "PROVISIONAL", "SRC-MEIHUA-006", "PENDING", ["TC-MEIHUA-004"]),
        rule("LOG-SCHEMA-001", "LOG", "schema", "L1", "SRC-LOG-001", "PENDING", ["TC-LOG-001"]),
        rule("IR-COMPLIANCE-001", "IR", "compliance", "L1", "SRC-IR-001", "PENDING", ["TC-IR-001"], result_type="ABSOLUTE_FILTER"),
    ]

    write_json(
        "data/rules.json",
        {
            "version": "RULE-v0.2",
            "status": "PENDING",
            "source": "ARS v0.2 附录 C/D + SRS v0.4",
            "rules": rules,
            "sources": [
                {
                    "source_ref": s[0],
                    "book": s[1],
                    "edition": s[2],
                    "chapter": s[3],
                    "original_excerpt": s[4],
                    "modern_note": s[5],
                    "rule_level": s[6],
                    "collation_status": s[7],
                    "copyright_status": s[8],
                }
                for s in sources
            ],
        },
    )


def gen_compliance() -> None:
    phrases = [
        ("必定", "绝对断语"),
        ("一定会", "绝对断语"),
        ("百分之百", "绝对断语"),
        ("绝对会", "绝对断语"),
        ("必然发生", "绝对断语"),
        ("注定", "宿命断语"),
        ("包治", "医疗承诺"),
        ("稳赚", "投资承诺"),
        ("改运", "效果承诺"),
    ]
    write_json(
        "ir/compliance/forbidden_phrases.json",
        {
            "version": "IR-COMPLIANCE-v0.1",
            "disclaimer": "本工具仅供个人学习、文化研究与娱乐，不构成任何决策、医疗、法律或投资建议；请勿据此替代专业意见。",
            "forbidden_phrases": [
                {"phrase": p, "reason": r} for p, r in phrases
            ],
        },
    )


def main() -> None:
    RES.mkdir(parents=True, exist_ok=True)
    gen_color_num()
    gen_zodiac()
    gen_bagua()
    gen_hex64()
    gen_najia()
    gen_xun()
    gen_wuxing_wangshuai()
    gen_rules()
    gen_compliance()


if __name__ == "__main__":
    main()
