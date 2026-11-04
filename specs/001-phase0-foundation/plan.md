# Implementation Plan: 阶段 0 — 可信基础与可复现内核

**Branch**: `001-phase0-foundation` | **Date**: 2026-11-03 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/001-phase0-foundation/spec.md`

## Summary

建立阶段 0 的可信基础：Rust workspace + CLI 验收入口，先实现 CAL 历法服务、种子链、规则/溯源加载、隐私边界与金标准测试框架；Tauri 2 壳与 React UI 在阶段 1 接入。技术栈与九色映射已由 ADR-001/ADR-002 批准。

## Technical Context

**Language/Version**: Rust 1.99.1（stable，2026-10-13）；TypeScript 5.x + React 19（阶段 1 UI，本阶段不构建）

**Primary Dependencies**: `chrono` + `chrono-tz`（时间/时区）、`serde` + `serde_json`（资源与日志）、`sha2` + `hmac` + `hex`（种子/HMAC/checksum）、`getrandom`（OS CSPRNG）、`unicode-normalization`（NFC）、`thiserror`（错误）、`clap`（CLI）、`rusqlite`（bundled，SQLite）、`tempfile`（测试）

**Storage**: SQLite 单文件 + WAL；阶段 0 建立 schema、profiles/runs/run_steps/random_snapshots/meta；资源文件只读 + manifest/checksum

**Testing**: `cargo test`（单元 + 金标准 + 隐私扫描 + 资源校验）；JSON fixture；`cargo clippy`；未来 `npm test`（UI）

**Target Platform**: Windows 10+/macOS 12+/Linux 桌面；阶段 0 在 Linux 上以 CLI 运行，不依赖 GUI 系统库

**Project Type**: 桌面应用 + 可复用核心库（Rust workspace）

**Performance Goals**: 种子生成 P95 < 50ms；单引擎推演 P95 < 200ms；一年历法遍历 P95 < 500ms；1000 条历史查询 P95 < 1s

**Constraints**: 离线优先、无遥测；支持日期 1900-01-31～2100-12-31；默认 `Asia/Shanghai` + `ZICHU`；生辰仅内存、不序列化；三引擎不合成总分；未冻结规则标“待校核”

**Scale/Scope**: 单本地实例、多家庭成员档案（≤10）、历史 ≤1000 条；阶段 0 交付 CAL/种子/规则/梅花内核 + CLI + 测试，不含完整 UI、焦氏 4096、纳甲、择吉权重

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| 原则 | 门禁 | 计划如何满足 | 结论 |
|---|---|---|---|
| I 文档分层 | 不得跨层定义 | `cal`/`seed`/`rules`/`meihua` 分 crate；TDS 只做实现 | PASS |
| II 规则溯源 | 每步 `rule_id/data_ref/source_ref/level` | `domain::StepLog` + `rules::Registry` 强制字段 | PASS |
| III 可复现 | 固定模式逐项一致；种子链无循环 | `seed` crate + 固定模式 fixture + 快照重放测试 | PASS |
| IV 隐私 | 生辰不落盘；问事默认不持久化 | `BaziMaterial` 不实现 `Serialize`；`privacy_scan` 测试 | PASS |
| V 不合成总分 | 三引擎平行 | 阶段 0 只做梅花；无总分代码 | PASS |
| VI 合规 | 禁用词 + 免责声明 | `ir` 最小合规模块 + 词表扫描 | PASS |
| VII 版本化 | 历史只读；资源 checksum | `VersionSet` + manifest + 旧七色只读 | PASS |
| VIII 金标准 | 规则未冻结不发布 | `PENDING` 状态 + golden fixtures | PASS |
| IX 离线轻量 | 核心不联网 | 无网络依赖；无遥测 | PASS |

**Gate result: PASS（无违反项）**

## Project Structure

### Documentation (this feature)

```text
specs/001-phase0-foundation/
├── plan.md              # 本文件
├── research.md          # Phase 0 研究结论
├── data-model.md        # Phase 1 实体与关系
├── quickstart.md        # Phase 1 验收指南
├── contracts/           # Phase 1 接口契约
│   ├── cli.md
│   ├── cal.md
│   ├── seed.md
│   └── resources.md
├── checklists/
│   └── requirements.md
└── tasks.md             # Phase 2 任务（/speckit-tasks）
```

### Source Code (repository root)

```text
Cargo.toml                    # workspace
rust-toolchain.toml           # stable
crates/
├── domain/                   # ID、版本、规则层级、五行、干支、卦象、日志结构
├── cal/                      # 公农历、节气、干支、日界、时区、旬空
├── seed/                     # 种子链、HMAC、拒绝采样、快照
├── rules/                    # manifest、checksum、规则/溯源注册表
├── meihua/                   # 九色/传统数字、互卦、变卦、体用、旺衰
├── ir/                       # 最小合规扫描（禁用词/免责声明），阶段 1 扩展
├── store/                    # SQLite schema、profiles/runs/steps/snapshots
├── app/                      # 编排、隐私扫描、导出快照
└── cli/                      # `yitool` 验收入口
resources/
├── data/
│   ├── color_num.json        # COLOR-NUM-01/02
│   ├── zodiac.json           # Z-01
│   ├── bagua.json            # G-01
│   ├── hex64.json            # HEX-01
│   ├── wuxing_wangshuai.json # W-01
│   └── xun.json              # XK-01
├── cal/
│   ├── lunar_months.json     # 预计算农历月表（1900–2100）
│   ├── jieqi.json            # 预计算节气时刻（1900–2100）
│   └── baseline/
│       ├── 2025.json         # 逐日基准（PENDING）
│       └── 2026.json
├── ir/
│   └── compliance/forbidden_phrases.json
└── manifests/
    └── data.manifest.json
scripts/
├── gen_cal_data.py           # 统一入口：sxtwl 预计算表 + 基准派生
├── gen_cal_precise.py        # sxtwl 生成 CAL 预计算表（可复现）
├── crosscheck_cal_hko.py     # HKO 1901–2100 逐日交叉校核
├── gen_resources.py          # 从 ARS 附录生成 hex64/bagua 等 JSON
├── gen_manifest.py           # 计算 checksum 并写 manifest
└── privacy_scan.sh           # 扫描日志/DB/导出（CI 辅助）
tests/
└── golden/                   # 跨 crate 金标准 JSON
```

**Structure Decision**: 采用 Rust workspace 单仓库；核心逻辑与 UI 解耦，阶段 0 用 `cli` 验收，阶段 1 再加 `src-tauri` 与 `ui/`。CAL 采用“预计算表 + 纯 Rust 查询”实现，避免天文算法在阶段 0 阻塞；`CAL-DATA-v0.2` 已用 HKO 1901–2100 对照表逐日校核。

## Complexity Tracking

> 无章程违反项，无需例外说明。

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| 无 | — | — |

## Phase 0: Research（摘要）

详见 [research.md](./research.md)。关键结论：

1. CAL 采用预计算表（农历月表 + 节气时刻表），来源 `sxtwl` 2.0.7，状态 `VERIFIED`；用 1900-01-31 甲辰、2000-01-01 戊午、2026-11-03 辛巳等独立锚点交叉校验，并以 HKO 1901–2100 逐日校核。
2. 日历范围 1900-01-31～2100-12-31；表约 2400 个月 + 4800 个节气，JSON 可控。
3. 时区用 `chrono-tz`；DST 重复取较早偏移，缺失向后平移 1 小时并标记。
4. 种子用 SHA-256/HMAC-SHA256；随机用 `getrandom` + 拒绝采样；固定模式不调用 OS 熵。
5. 九色按 ADR-002；旧七色只读。
6. 资源加载用 manifest + SHA-256 checksum；缺失/损坏拒绝加载。
7. 隐私：`BaziMaterial` 不实现 `Serialize`，`Debug` 脱敏；日志/DB/导出扫描。

## Phase 1: Design（摘要）

- 实体与关系见 [data-model.md](./data-model.md)。
- 接口契约见 [contracts/](./contracts/)。
- 验收入口见 [quickstart.md](./quickstart.md)。
- 数据库 schema：`profiles`、`settings`、`runs`、`run_steps`、`random_snapshots`、`notes`、`meta`（含 `hmac_key_id`）。
- CLI 命令：`cal`、`seed`、`meihua`、`verify`、`privacy-scan`、`db`。

**Post-design Constitution Check: PASS**（设计未引入新的跨层依赖；CAL/SEED/RULES/MEIHUA 边界保持不变）。
