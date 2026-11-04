# Data Model: 阶段 0 — 可信基础与可复现内核

**Date**: 2026-11-03
**Status**: Draft（随实现更新）
**Storage**: SQLite（WAL）+ 只读资源文件（JSON + manifest）

---

## 1. 实体关系

```text
Profile 1 ─── * Run 1 ─── * StepLog
                    │
                    ├── 1 RandomSnapshot
                    ├── 1 InputSnapshot (逻辑对象，存于 Run/Snapshot)
                    └── * Note

DataManifest 1 ─── * ResourceEntry
RuleRecord * ─── 1 SourceRecord
Run * ─── 1 VersionSet
```

---

## 2. 实体定义

### 2.1 Profile（档案）

| 字段 | 类型 | 约束 | 说明 |
|---|---|---|---|
| `profile_id` | TEXT | PK，UUID/ULID | 稳定标识 |
| `display_name` | TEXT | NOT NULL，1–50 字符 | 家庭成员显示名 |
| `created_at` | TEXT | ISO 8601 | 创建时间 |
| `is_default` | INTEGER | 0/1，仅一个默认 | 默认档案 |

**校验**：档案删除后其 Run/Setting/Note 级联删除；生辰不跨档案共享。

### 2.2 Setting（档案设置）

| 字段 | 类型 | 默认 | 说明 |
|---|---|---|---|
| `profile_id` | TEXT | PK/FK | 档案 |
| `mode` | TEXT | `FIXED` | `FIXED|CAST` |
| `input_mode` | TEXT | `NINE_COLOR` | `NINE_COLOR|TRADITIONAL_NUMBER` |
| `aux_mode` | TEXT | `ZODIAC` | `ZODIAC|MONTH` |
| `day_bound` | TEXT | `ZICHU` | `ZICHU|ZIZHENG` |
| `timezone` | TEXT | `Asia/Shanghai` | IANA |
| `ab_mode` | TEXT | `INDEPENDENT` | `INDEPENDENT|UNIFIED` |
| `animation_enabled` | INTEGER | 1 | 可关闭动画 |
| `privacy_redact` | INTEGER | 1 | 导出脱敏 |

### 2.3 Run（推演记录）

| 字段 | 类型 | 约束 | 说明 |
|---|---|---|---|
| `run_id` | TEXT | PK | UUID |
| `profile_id` | TEXT | FK | 档案 |
| `created_at` | TEXT | ISO 8601 | 创建时间 |
| `mode` | TEXT | `FIXED|CAST` | 推演模式 |
| `engine_scope` | TEXT | `MEIHUA|MEIHUA+JIAOLIN|ALL` | 引擎范围 |
| `question_hmac` | TEXT | 可空 | 问事 HMAC；默认 `NONE` |
| `question_saved` | INTEGER | 0/1 | 是否加密保存原文 |
| `bazi_present` | INTEGER | 0/1 | 是否含生辰；不含任何生辰材料 |
| `hmac_key_id` | TEXT | 可空 | HMAC 密钥标识（A-09） |
| `random_snapshot_digest` | TEXT | 可空 | 随机快照摘要 |
| `ars_version` | TEXT | NOT NULL | 版本 |
| `ir_version` | TEXT | NOT NULL | 版本 |
| `cal_version` | TEXT | NOT NULL | 版本 |
| `rule_set` | TEXT | NOT NULL | `RULE-v0.2` |
| `data_version` | TEXT | NOT NULL | `DATA-v0.2` |
| `tzdata_version` | TEXT | 可空 | IANA tzdata 版本 |
| `disclaimer_present` | INTEGER | 默认 1 | 免责声明 |

**校验**：`bazi_present=1` 时不得有任何生辰字段；`question_saved=1` 时必须 `question_hmac` 非空且加密原文单独存储；`mode=CAST` 时必须有 `random_snapshot_digest`。

### 2.4 StepLog（步骤日志）

| 字段 | 类型 | 约束 | 说明 |
|---|---|---|---|
| `run_id` | TEXT | FK | 推演 |
| `step_index` | INTEGER | PK 组合 | 顺序 |
| `engine` | TEXT | `MEIHUA|JIAOLIN|LIUYAO|ZEJI|CAL|SEED|IR` | 引擎 |
| `step_id` | TEXT | NOT NULL | 如 `MEIHUA-QIGUA-001` |
| `rule_ids_json` | TEXT | JSON 数组 | 规则 ID |
| `data_refs_json` | TEXT | JSON 数组 | 数据表 ID |
| `source_ref` | TEXT | 可空 | 典籍出处 |
| `rule_level` | TEXT | `L1|L2|L3|PROVISIONAL` | 层级 |
| `formula_summary` | TEXT | 可空 | 公式摘要 |
| `output_json` | TEXT | JSON | 输出 |
| `status` | TEXT | `OK|PENDING|ERROR` | 状态 |

**校验**：`rule_ids` 非空；`rule_level` 合法；`PENDING` 步骤不得被标记为正式结论。

### 2.5 InputSnapshot（输入快照，逻辑对象）

| 字段 | 类型 | 说明 |
|---|---|---|
| `color_scheme` | TEXT | `NINE_COLOR|TRADITIONAL_NUMBER` |
| `color_id` | INTEGER? | 1–9（九色） |
| `aux_type` | TEXT | `ZODIAC|MONTH` |
| `aux_id` | INTEGER? | 1–12 |
| `traditional_numbers` | [u32;2]? | 传统数字模式 |
| `question_hmac` | TEXT | HMAC 或 `NONE` |
| `bazi_present` | BOOL | 仅布尔 |
| `liuyao_input_type` | TEXT | `COIN_CAST|MANUAL_YAO|FIXED_DERIVED` |
| `random_snapshot_digest` | TEXT? | 快照摘要 |

**校验**：九色模式 `color_id ∈ 1..=9`；传统数字模式 `traditional_numbers` 非空；不含生辰材料。

### 2.6 RandomSnapshot（随机快照）

| 字段 | 类型 | 说明 |
|---|---|---|
| `run_id` | TEXT | PK/FK |
| `snapshot_json` | TEXT | 原始比特/铜钱/抽卡结果 |
| `snapshot_digest` | TEXT | SHA-256 |
| `rng_version` | TEXT | `RNG-v1` |

**校验**：`snapshot_digest` 与内容一致；不含生辰/问事原文。

### 2.7 RuleRecord（规则记录）

| 字段 | 类型 | 说明 |
|---|---|---|
| `rule_id` | TEXT | PK |
| `engine` | TEXT | MEIHUA/JIAOLIN/LIUYAO/ZEJI/CAL/SEED/RNG/LOG/IR |
| `category` | TEXT | 类别 |
| `rule_level` | TEXT | L1/L2/L3/PROVISIONAL |
| `condition` | JSON | 匹配条件 |
| `result_type` | TEXT? | POSITIVE/NEGATIVE/ABSOLUTE_FILTER/INFORMATION/CONFLICT |
| `weight` | NUMBER? | 权重 |
| `priority` | INTEGER? | 优先级 |
| `source_ref` | TEXT | 来源 |
| `version` | TEXT | 规则版本 |
| `status` | TEXT | VERIFIED/PENDING |
| `test_refs` | [TEXT] | 测试样例 ID |

### 2.8 SourceRecord（典籍出处）

| 字段 | 类型 | 说明 |
|---|---|---|
| `source_ref` | TEXT | PK，如 `SRC-MEIHUA-001` |
| `book` | TEXT | 书名；L3 必须“本系统自定义” |
| `edition` | TEXT | 版本 |
| `chapter` | TEXT | 卷/篇 |
| `original_excerpt` | TEXT | 原文/规则摘要（L1 不得改写） |
| `modern_note` | TEXT | 现代释义 |
| `rule_level` | TEXT | L1/L2/L3/PROVISIONAL |
| `collation_status` | TEXT | VERIFIED/PENDING |
| `copyright_status` | TEXT | 版权状态 |

### 2.9 DataManifest / ResourceEntry

| 字段 | 类型 | 说明 |
|---|---|---|
| `manifest_id` | TEXT | `MANIFEST-data` 等 |
| `version` | TEXT | `DATA-v0.2` |
| `entries[].id` | TEXT | 表 ID |
| `entries[].path` | TEXT | 相对路径 |
| `entries[].version` | TEXT | 表版本 |
| `entries[].status` | TEXT | FROZEN/VERIFIED/PENDING/DEPRECATED |
| `entries[].source` | TEXT | 来源 |
| `entries[].sha256` | TEXT | 校验和 |
| `entries[].updated_at` | TEXT | 日期 |

### 2.10 VersionSet（版本戳）

```json
{
  "srs": "SRS-v0.4",
  "ars": "ARS-v0.2",
  "ir": "IR-v0.1",
  "cal": "CAL-v0.1",
  "rule_set": "RULE-v0.2",
  "data": "DATA-v0.2",
  "cal_data": "CAL-DATA-v0.2",
  "tzdata": "2025b",
  "seed_algo": "SEED-v2",
  "rng": "RNG-v1",
  "app": "APP-v0.1",
  "db_schema": "DB-v0.2"
}
```

### 2.11 BaziMaterial（内存对象）

| 字段 | 类型 | 说明 |
|---|---|---|
| `year/month/day/hour` | 内部 | 仅内存 |
| `timezone` | IANA | 换算用 |
| `day_bound` | ZICHU/ZIZHENG | 换算用 |

**约束**：不实现 `Serialize`；`Debug` 输出 `<redacted>`；生命周期限于单次推演调用；不写入任何日志/DB/快照/导出。

### 2.12 CalResult（历法结果）

```json
{
  "solar": "2026-11-03T12:00:00+08:00",
  "timezone": "Asia/Shanghai",
  "day_bound": "ZICHU",
  "effective_date": "2026-11-03",
  "lunar": {"year": 2026, "month": 9, "day": 25, "leap": false},
  "lunar_month_index": {"lunar_index": 9, "leap": false},
  "jieqi": {"current": "霜降", "next": "立冬"},
  "jieqi_month_branch": {"branch_id": 11, "branch": "戌", "element": "土", "jieqi": "寒露"},
  "ganzhi": {"year": "丙午", "month": "戊戌", "day": "辛巳", "hour": "甲午"},
  "xun": {"xun_head": "甲戌", "kong_branches": ["申", "酉"]},
  "dst": {"ambiguous": false, "gap": false},
  "versions": {"cal": "CAL-v0.1", "cal_data": "CAL-DATA-v0.2", "tzdata": "..."},
  "status": "VERIFIED"
}
```

### 2.13 BaselineRecord（逐日基准）

| 字段 | 类型 | 说明 |
|---|---|---|
| `solar_datetime` | TEXT | ISO 8601 + 时区 |
| `timezone` | TEXT | IANA |
| `day_bound` | TEXT | ZICHU/ZIZHENG |
| `lunar` | object | 年/月/日/闰 |
| `lunar_month_index` | integer | 1–12 |
| `jieqi` | TEXT | 当前节气 |
| `jieqi_month_branch` | TEXT | 月建 |
| `ganzhi` | object | 年/月/日/时 |
| `xun` | object | 旬首/空亡 |
| `source` | TEXT | 来源 |
| `status` | TEXT | VERIFIED/PENDING |

---

## 3. 状态机

### 3.1 Run

```text
CREATED → RUNNING → COMPLETED
                 └→ FAILED（错误码，不落敏感数据）
COMPLETED → DELETED（应用内删除）
```

### 3.2 Resource

```text
PENDING → VERIFIED → FROZEN
    └──────────────→ DEPRECATED
```

### 3.3 版本兼容

- 打开旧记录：按 `runs` 中的版本加载对应资源；资源缺失显示“原版本资源不可用”。
- 用户手动“按当前版本重新解释”：生成新记录并标注“重新解释，非原始记录”。

---

## 4. 索引与查询

| 表 | 索引 | 用途 |
|---|---|---|
| `runs` | `(profile_id, created_at DESC)` | 历史列表 |
| `runs` | `(mode, created_at)` | 统计 |
| `run_steps` | `(run_id, step_index)` | 详情 |
| `random_snapshots` | `(run_id)` | 重放 |
| `resources` | `(id)` | 加载校验 |

---

## 5. 隐私字段白名单

**允许持久化**：`profile_id`、`run_id`、时间、模式、版本、`bazi_present`、`question_hmac`、`hmac_key_id`、`random_snapshot_digest`、步骤结构化输出、规则/数据/来源 ID。

**禁止持久化**：生辰原始值、生辰派生值（四柱、藏干、纳音等）、问事原文、HMAC 密钥、密钥明文、含上述内容的调试日志。

---

## 6. 待实现/待冻结

- `settings` 表字段需与 UI 阶段 1 对齐。
- `notes`（复盘笔记）阶段 1 实现。
- `zeji_*`、`interpretations` 表阶段 4 实现。
- `hmac_key_id` 已在 schema 预留（A-09）。
- `random_snapshots` 当前一 Run 一快照；A/B 阶段 4 可能需一对多，需迁移。
