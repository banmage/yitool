# Implementation Plan: 阶段 2 — 焦氏易林索引骨架与查询联动

**Branch**: `004-phase2-jiaolin` | **Date**: 2026-11-11 | **Spec**: [spec.md](./spec.md)

## Summary

先交付可校验的 `JL-01` 索引骨架：4096 个有序卦对、64 条本卦本卦、稳定卦 ID、`PENDING`/`VERIFIED` 状态与查询联动。资源初始全部为 `PENDING`，不录入或编造原文；D-28/O-06 关闭后再按底本增量校勘。实现复用现有 `domain`/`app`/`store`/`ir`/桌面命令层，新增 `jiaolin` crate、`DB-v0.4` 迁移、CLI 与只读桌面面板。

## Technical Context

**Language/Version**: Rust stable; TypeScript/React  
**Primary Dependencies**: existing `domain`/`ir`/`app`/`store` crates, `serde`/`serde_json`/`thiserror`, Tauri 2, React, Vitest  
**Storage**: 本地 JSON 资源 `resources/data/jiaolin_4096.json`；SQLite `DB-v0.4` 为 `run_reports` 增加 `jl_version`/`jl_json`  
**Testing**: `cargo test --workspace`、新增 `crates/jiaolin` 集成测试、`crates/app`/`store` 历史测试、前端 Vitest、`./scripts/check.sh`  
**Target Platform**: Tauri 桌面 + `yitool` CLI；离线运行  
**Constraints**: 4096 键完整；不编造原文；`PENDING` 显示“待校勘”；不合成吉凶总分；历史按保存版本渲染  
**Scale/Scope**: 64×64 索引约 0.8–1.5 MB JSON；单次查询 O(1)；不进入纳甲/择吉/三引擎

## Constitution Check

- **隐私**: 焦氏资源、查询结果、日志与历史不包含生辰原始值/派生值、问事原文或 HMAC 密钥。
- **溯源**: 每条 `VERIFIED` 记录必须有底本/版本来源与 `source_ref`；`PENDING` 也保留来源状态。
- **版本**: `JL-01-v0.1` 为骨架版本；内容变化必须升版、重算 checksum；历史保存 `jl_version` 与 `jl_json`，不静默重算。
- **合规**: 原文逐字引用，系统白话标注“本系统白话提示，非原文”；不输出独立吉凶等级或绝对断语；所有结果附免责声明。
- **阶段范围**: 只做焦氏易林；不启用纳甲、择吉、黄历、三引擎综合评分。
- **数据治理**: 初始 4096 条全部 `PENDING`；D-28/O-06 关闭前不批量录入未授权文本，不关闭 `JL-01` 或阶段 0。

## Design

### 1. `JL-01` 资源与骨架生成

资源路径：`resources/data/jiaolin_4096.json`。顶层元数据：

```json
{
  "id": "JL-01",
  "version": "JL-01-v0.1",
  "status": "PENDING",
  "source": "结构骨架；底本/校勘/版权待 D-28/O-06 冻结",
  "entries": []
}
```

每条记录字段与 ARS 1.10 对齐：

```json
{
  "origin_hex_id": "QIAN/KUN",
  "target_hex_id": "QIAN/ZHEN",
  "origin_name": "天地否",
  "target_name": "天雷无妄",
  "text": null,
  "edition": null,
  "collation_status": "PENDING",
  "variants": [],
  "plain_note": null,
  "source_ref": null,
  "updated_at": null
}
```

- 稳定卦 ID 复用 `domain::Hexagram::key()` 的 `UPPER/LOWER` 约定；`origin_name`/`target_name` 来自 `HEX-01`，仅用于阅读与交叉校验。
- 新增 `scripts/gen_jiaolin_skeleton.py`：从 `resources/data/hex64.json` 生成 64×64 全 `PENDING` 骨架；默认拒绝覆盖已有文件，`--force` 才重建，避免覆盖后续校勘数据。
- `scripts/gen_manifest.py` 的 `IDS` 增加 `data/jiaolin_4096.json: JL-01`，并重算 `resources/manifests/data.manifest.json`。

### 2. `crates/jiaolin` 引擎

新增 workspace crate：

```text
crates/jiaolin/
├── Cargo.toml
└── src/
    ├── lib.rs        # 导出 table/query/error
    ├── table.rs      # 资源加载、完整性校验、索引
    └── error.rs      # JiaolinError
```

核心类型：

- `CollationStatus { Verified, Pending }`
- `JiaolinEntry`：资源记录，资源与 CLI/桌面嵌套结果使用 snake_case，与现有 `MeihuaResult` 一致；`CastingPayload` 外层仍为 camelCase。
- `JiaolinTable`：`entries: Vec<JiaolinEntry>` + `by_key: HashMap<(String, String), usize>` + `version` + `status`。
- `JiaolinLookup`：查询结果，包含本卦/之卦 ID 与名称、查询来源、动爻注记、命中条目、结果状态、取辞提示、版本。
- `LookupStatus { Verified, Pending, Missing }`（`Missing` 只作为查询结果，不写入资源状态）。

加载校验：

1. `id == "JL-01"`，版本非空，`status` 为 `PENDING`/`VERIFIED`。
2. 恰好 4096 条，键唯一，覆盖全部 64×64。
3. `origin==target` 恰好 64 条。
4. 每个 ID 属于 64 个 `Hexagram::key()` 集合；名称与 `HEX-01` 一致。
5. 状态只能为 `VERIFIED`/`PENDING`；`VERIFIED` 必须有非空 `text`、`edition`、`source_ref`；`PENDING` 允许 `text` 为空。
6. 任何校验失败返回结构化错误，不进入可用状态。

查询：

```text
lookup(origin, target):
  key = (origin.key(), target.key())
  entry = by_key.get(key)
  if entry is None -> Missing
  if entry.status == PENDING -> Pending
  else -> Verified
```

不做反向键、对角、相邻条目回退。

### 3. `domain` 卦 ID 辅助

在 `crates/domain/src/hexagram.rs` 增加：

- `Trigram::from_key(&str) -> Option<Trigram>`
- `Hexagram::from_key(&str) -> Option<Hexagram>`（解析 `UPPER/LOWER`）
- `Hexagram::from_name(&str) -> Option<Hexagram>`（遍历 `HEX_NAMES`）

用于 CLI 名称/键解析与资源交叉校验；补充单元测试，确保 64 个键与名称一一对应。

### 4. `app` 集成

- `crates/app/src/jiaolin.rs`：
  - `resolve_href(table, href)`：解析 `yitool://jiaolin/<origin-slug>/<target-slug>`；slug 将 `/` 替换为 `-`（如 `QIAN-KUN`）。格式非法返回错误；合法但缺失/未校勘返回带状态的结果。
  - `JiaolinLinkError`。
- `crates/app/src/context.rs`：
  - `AppContext` 增加 `jiaolin: JiaolinTable`，启动时加载并校验。
  - `AppContext::jiaolin_query(origin, target, source, yao_note)`。
  - `VersionSet` 增加 `jl_data`（默认 `JL-01-v0.1`），`AppContext::versions()` 从已加载表覆盖该字段。
- `crates/app/src/report.rs`：
  - 在结果摘要后增加“焦氏易林参考”小节：取辞索引、状态、原文（若有）、版本、异文、白话提示、`JL-01` 版本。
  - `PENDING` 显示“待校勘”；缺失显示“该条待校勘，暂不提供解释”。
  - 报告仍使用同一份 Markdown 供窗口/复制/下载。
- `crates/app/src/lib.rs` 导出新模块与类型。

### 5. `store` 与历史（`DB-v0.4`）

新增迁移：

```sql
ALTER TABLE run_reports ADD COLUMN jl_version TEXT;
ALTER TABLE run_reports ADD COLUMN jl_json TEXT;
```

- `SCHEMA_VERSION` 升为 `DB-v0.4`。
- `RunReport` 增加 `jl_version: Option<String>`、`jl_json: Option<String>`。
- `save_run_report`/`get_run_report` 读写新列；旧记录为 `NULL`。
- 迁移测试：`DB-v0.3` 升级保留原报告，新列为 `NULL`；`DB-v0.4` 可重复迁移。
- `crates/app/src/history.rs` 的 `HistoryDetail` 不需要新表；`run.report.jl_json` 已包含保存时的焦氏查询。

### 6. `ir` 文案

新增 `crates/ir/src/jiaolin.rs`（或 `lib.rs` 子模块）：

- `PLAIN_NOTE_LABEL = "本系统白话提示，非原文"`
- `pending_message() = "该条待校勘，暂不提供解释"`
- `index_hint(origin_eq_target: bool)`：本卦本卦 / 有序取辞。
- `display_status(status)`：`VERIFIED` / `PENDING` / `MISSING` 的中文标签。
- 单元测试覆盖文案与“不出现吉凶等级”。

### 7. CLI

`crates/cli/src/main.rs` 增加：

```bash
yitool jiaolin --origin "天地否" --target "天雷无妄"
yitool jiaolin --origin-key "QIAN/KUN" --target-key "QIAN/ZHEN"
yitool jiaolin --href "yitool://jiaolin/QIAN-KUN/QIAN-ZHEN"
yitool jiaolin --origin "乾为天"   # 无 target 时取本卦本卦
```

- 名称或键均可；未知名称/键返回 `ARGUMENT` 错误。
- JSON 输出包含 `origin_hex_id`、`target_hex_id`、`origin_name`、`target_name`、`source`、`yao_note`、`status`、`entry`、`index_hint`、`version`。
- `--format text` 输出简短可读文本；`PENDING`/`MISSING` 显示“待校勘”。

### 8. 桌面后端与 UI

后端 `apps/desktop/src-tauri/src/lib.rs`：

- `JiaolinOptions`：64 卦的 `hexId`、`name`、`upper`、`lower`。
- `get_jiaolin_options(context)`。
- `get_jiaolin(context, origin_hex_id, target_hex_id, source, yao_note)`。
- `resolve_jiaolin(context, href)`。
- `perform_cast_with_store` 在梅花成功后自动查询焦氏，写入 `CastingPayload.jiaolin` 与 `RunReport.jl_*`；查询失败不阻断梅花保存。

`apps/desktop/src-tauri/src/main.rs` 注册新命令。

前端：

- `apps/desktop/src/types.ts`：`JiaolinEntry`、`JiaolinLookup`、`JiaolinOptions`、`VersionSet.jl_data`。
- `apps/desktop/src/api.ts`：`getJiaolinOptions`、`getJiaolin`、`resolveJiaolin`。
- `apps/desktop/src/JiaolinPanel.tsx`：只读面板 + 手动查询两个下拉框；展示状态、取辞索引、原文、版本、异文、白话提示；`PENDING`/缺失显示“待校勘”。
- `apps/desktop/src/App.tsx`：结果区下方接入面板；历史详情从 `jl_json` 恢复，不重新查询。
- 前端测试覆盖：自动联动、手动查询、`PENDING` 展示、`VERIFIED` 夹具、缺失/断链、历史恢复。

### 9. 测试与金标准

- `tests/golden/jl01_queries.json`：记录若干查询样例与预期键/状态，至少包含“天地否 → 天雷无妄”、`origin==target`、反向键不被命中。
- `crates/jiaolin/tests/integrity.rs`：4096、64 对角、唯一性、状态、`HEX-01` 交叉。
- `crates/jiaolin/tests/query.rs`：取辞规则、无回退、缺失状态。
- `crates/jiaolin/tests/tamper.rs`：重复/缺失/未知 ID/非法状态/空 `VERIFIED` 字段被拒绝。
- `crates/app/tests/jiaolin.rs`：报告小节、历史保存/重载、隐私扫描、断链。
- `crates/store` 内联测试：`DB-v0.4` 迁移与 `jl_json` 往返。
- 前端 `JiaolinPanel` 测试。

## Project Structure

```text
crates/jiaolin/                 # 新引擎：加载、校验、查询
crates/domain/src/hexagram.rs   # 卦 ID/名称辅助
crates/ir/src/jiaolin.rs        # 焦氏文案与状态标签
crates/app/src/jiaolin.rs       # 应用编排、链接解析
crates/app/src/context.rs       # 加载 JL-01、查询入口
crates/app/src/report.rs        # 报告焦氏小节
crates/store/src/lib.rs         # DB-v0.4 迁移与 jl_* 列
crates/cli/src/main.rs          # jiaolin 子命令
apps/desktop/src-tauri/src/     # 桌面命令
apps/desktop/src/               # 焦氏面板、API、类型、测试
resources/data/jiaolin_4096.json
resources/manifests/data.manifest.json
scripts/gen_jiaolin_skeleton.py
scripts/gen_manifest.py
tests/golden/jl01_queries.json
docs/adr/ADR-025-jiaolin-index-structure.md
specs/004-phase2-jiaolin/
```

## Data Model

### `JL-01` 记录

| 字段 | 类型 | 约束 |
|---|---|---|
| `origin_hex_id` | string | `UPPER/LOWER`，属于 64 卦 |
| `target_hex_id` | string | 同上 |
| `origin_name` / `target_name` | string | 与 `HEX-01` 一致 |
| `text` | string/null | `VERIFIED` 必填；`PENDING` 可为空 |
| `edition` | string/null | `VERIFIED` 必填 |
| `collation_status` | enum | `VERIFIED` / `PENDING` |
| `variants` | string[] | 可为空 |
| `plain_note` | string/null | 系统自写，标注非原文 |
| `source_ref` | string/null | `VERIFIED` 必填 |
| `updated_at` | string/null | 校勘时间 |

### `RunReport` 扩展

| 列 | 类型 | 说明 |
|---|---|---|
| `jl_version` | TEXT NULL | 保存时的 `JL-01` 版本 |
| `jl_json` | TEXT NULL | 保存时的焦氏查询结果 JSON |

## Contracts

### CLI

```json
{
  "origin_hex_id": "QIAN/KUN",
  "target_hex_id": "QIAN/ZHEN",
  "origin_name": "天地否",
  "target_name": "天雷无妄",
  "source": "MANUAL",
  "yao_note": "初爻",
  "status": "PENDING",
  "entry": null,
  "index_hint": "取辞索引 (QIAN/KUN, QIAN/ZHEN)",
  "version": "JL-01-v0.1"
}
```

### Tauri commands

- `get_jiaolin_options() -> JiaolinOptions`
- `get_jiaolin(originHexId, targetHexId, source, yaoNote) -> JiaolinLookup`
- `resolve_jiaolin(href) -> JiaolinLookup`

## Risks

- 4096 条 JSON 体积与加载时间：用 `HashMap` 索引、启动时一次加载；NFR 目标 200 ms。
- `DB-v0.4` 迁移必须保留旧报告；`ALTER TABLE ADD COLUMN` 为向后兼容操作，需测试旧库升级。
- 前端 API mock 需同步扩展；否则新增命令会导致现有测试误报。
- `yitool://jiaolin/` slug 与 `UPPER/LOWER` 的转换必须唯一且可逆；非法 slug 拒绝。
- D-28/O-06 未关闭：不得把任何未校勘条目标为 `VERIFIED`；发布资源默认全 `PENDING`。
- `VERIFIED` 测试夹具不得混入发布资源。
- 焦氏查询失败不得阻断梅花保存；自动查询失败时 payload 记录错误/空结果。

## Verification

1. `cargo test -p yitool-jiaolin`
2. `cargo test --workspace`
3. `cargo run -p yitool-cli -- jiaolin --origin "天地否" --target "天雷无妄"`
4. `cargo run -p yitool-cli -- verify --resources resources`
5. `cd apps/desktop && npm test && npm run build`
6. `./scripts/check.sh`
7. 在具备 GTK/WebKit 的环境运行 `npm run tauri dev`，检查焦氏面板与历史恢复。
