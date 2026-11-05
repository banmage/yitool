#!/usr/bin/env python3
"""计算 resources/ 下只读资源的 SHA-256 并写入 manifest。"""

from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
RES = ROOT / "resources"
MANIFEST = RES / "manifests" / "data.manifest.json"

IDS = {
    "data/color_num.json": "COLOR-NUM-02",
    "data/zodiac.json": "Z-01",
    "data/bagua.json": "G-01",
    "data/hex64.json": "HEX-01",
    "data/najia.json": "NJ-01",
    "data/xun.json": "XK-01",
    "data/wuxing_wangshuai.json": "W-01",
    "data/rules.json": "RULE-SET-01",
    "cal/lunar_months.json": "CAL-LUNAR-DATA",
    "cal/jieqi.json": "CAL-JIEQI-DATA",
    "cal/baseline/2025.json": "CAL-BASELINE-2025",
    "cal/baseline/2026.json": "CAL-BASELINE-2026",
    "ir/compliance/forbidden_phrases.json": "IR-COMPLIANCE-01",
}


def main() -> None:
    entries = []
    for rel, resource_id in sorted(IDS.items()):
        path = RES / rel
        if not path.exists():
            raise SystemExit(f"缺少资源: {path}")
        raw = path.read_bytes()
        meta = json.loads(raw)
        entries.append(
            {
                "id": resource_id,
                "path": rel,
                "version": meta.get("version", "DATA-v0.2"),
                "status": meta.get("status", "VERIFIED"),
                "source": meta.get("source", "unknown"),
                "sha256": hashlib.sha256(raw).hexdigest(),
            }
        )

    doc = {
        "manifest_id": "MANIFEST-data",
        "version": "DATA-v0.2",
        "generated_by": "scripts/gen_manifest.py",
        "not_frozen_notice": "所有 PENDING 条目在对应规则集冻结前不得作为正式结论；CAL v0.2 已冻结，ARS/IR 等仍待冻结。",
        "entries": entries,
    }
    MANIFEST.parent.mkdir(parents=True, exist_ok=True)
    MANIFEST.write_text(
        json.dumps(doc, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    print(f"wrote {MANIFEST.relative_to(ROOT)} ({len(entries)} entries)")


if __name__ == "__main__":
    main()
