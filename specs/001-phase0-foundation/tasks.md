---

description: "Task list for 阶段 0 — 可信基础与可复现内核"
---

# Tasks: 阶段 0 — 可信基础与可复现内核

**Input**: Design documents from `/specs/001-phase0-foundation/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: 本特性以“可信基础 + 金标准”为目标，测试为必交付项（spec FR-019、SC-001～SC-010）。

**Organization**: 按用户故事分组；P1 故事构成 MVP，P2 故事在 P1 通过后交付。

## Format: `[ID] [P?] [Story] Description`

- **[P]**: 可并行（不同文件、无依赖）
- **[Story]**: 对应 spec.md 的 US1～US6

## Path Conventions

Rust workspace + 只读资源 + CLI：

```text
Cargo.toml
crates/{domain,cal,seed,rules,meihua,ir,store,app,cli}/
resources/{data,cal,ir,manifests}/
scripts/
tests/golden/
```

---

## Phase 1: Setup（共享基础设施）

**Purpose**: 建立可编译、可测试的工程骨架与资源生成链路。

- [x] T001 创建 workspace `Cargo.toml`、`rust-toolchain.toml`、`.cargo/config.toml`（离线友好、无遥测）
- [x] T002 [P] 创建 9 个 crate 目录与最小 `lib.rs`/`src/main.rs`，确保 `cargo check --workspace` 通过
- [x] T003 [P] 配置 `rustfmt.toml`、`clippy` lint 基线（`-D warnings` 于 CI 脚本）
- [x] T004 [P] 建立 `resources/` 目录结构与 `resources/manifests/data.manifest.json` 空模板
- [x] T005 编写 `scripts/gen_cal_data.py`：用天文算法生成 1900–2100 农历月表 + 节气时刻表
- [x] T006 [P] 编写 `scripts/gen_resources.py`：从 ARS 附录 A/B、九色表、生肖表生成 JSON 资源
- [x] T007 [P] 编写 `scripts/gen_manifest.py`：计算 SHA-256 并写入 manifest
- [x] T008 生成首批资源并校验：`lunar_months.json`、`jieqi.json`、`hex64.json`、`bagua.json`、`color_num.json`、`zodiac.json`、`xun.json`、`wuxing_wangshuai.json`
- [x] T009 编写 `scripts/check.sh`：`fmt --check` + `clippy` + `test --workspace` + `verify` + `privacy-scan`

**Checkpoint**: `cargo check --workspace` 与资源生成脚本可重复运行。

---

## Phase 2: Foundational（阻塞性前置）

**Purpose**: 所有用户故事共享的类型、版本、错误与资源加载底座。

- [x] T010 [P] `crates/domain`：`VersionSet`、`RuleLevel`、`Status`、`Engine`、错误码枚举
- [x] T011 [P] `crates/domain`：`GanZhi`、`WuXing`、`Trigram`、`Hexagram`、`StepLog` 类型与校验
- [x] T012 [P] `crates/domain`：`Hexagram` 依据上/下卦与六爻二进制互转，覆盖 64 卦唯一性测试
- [x] T013 `crates/rules`：manifest 解析、相对路径防穿越、SHA-256 checksum 校验、缺失/损坏错误
- [x] T014 [P] `crates/rules`：`RuleRecord`/`SourceRecord` 注册表与 `rule_id` 唯一性校验
- [x] T015 [P] `crates/ir`：免责声明常量、禁用绝对断语词表加载与扫描接口
- [x] T016 `crates/cli`：`clap` 骨架、统一 JSON 包装 `{ok,data|error,versions}`、退出码映射
- [x] T017 [P] 建立 `tests/golden/` 读取器与 fixture schema（JSON，带 `case_id`/`rule_id`/版本）

**Checkpoint**: 资源可校验加载；CLI 可输出统一错误结构；domain 类型测试通过。

---

## Phase 3: User Story 1 — 固定模式可复现 (P1) 🎯 MVP

**Goal**: 同输入同版本逐项一致；快照跨会话可重放且不含生辰。

**Independent Test**: 100 次固定模式运行 + 导出/导入快照，逐项比对。

### Tests

- [x] T018 [P] [US1] `crates/seed/tests/determinism.rs`：同输入 100 次种子链完全一致
- [x] T019 [P] [US1] `crates/seed/tests/replay.rs`：快照导出→新进程导入→run_seed 一致
- [x] T020 [P] [US1] `crates/app/tests/snapshot_privacy.rs`：快照 JSON 不含生辰/问事原文

### Implementation

- [x] T021 [US1] `crates/seed`：NFC 规范化 + 长度前缀编码 + SHA-256 基础种子（`SEED-BASE-001`）
- [x] T022 [US1] `crates/seed`：抽卡子种子与输入快照（`SEED-DRAW-001`、`SEED-SNAPSHOT-001`）
- [x] T023 [US1] `crates/seed`：推演种子与子种子派生，禁止循环依赖（`SEED-RUN-001`）
- [x] T024 [US1] `crates/seed`：A/B 统一条件共用 base/draw/snapshot；问事 HMAC 不参与共同随机（FR-004）
- [x] T025 [US1] `crates/seed`：快照规范化序列化 + SHA-256 摘要，`InputSnapshot` 不含生辰
- [x] T026 [US1] `crates/app`：固定模式编排与逐项结果比对器；导出快照（脱敏）
- [x] T027 [US1] `crates/cli seed`：按 `contracts/cli.md` 输出种子链与快照

**Checkpoint**: US1 独立测试通过；快照重放一致。

---

## Phase 4: User Story 2 — 历法结果可信可核 (P1)

**Goal**: 公农历、节气、四柱、日界、时区、DST、旬空可查询且有版本；逐日基准通过。

**Independent Test**: 2025/2026 逐日基准 + 立春/节气/闰月/23:00/DST 边界。

### Tests

- [x] T028 [P] [US2] `crates/cal/tests/anchors.rs`：1900-01-31 甲辰、2000-01-01 戊午、2026-11-03 农历 09-25/辛巳/甲戌旬空申酉
- [x] T029 [P] [US2] `crates/cal/tests/baseline_2025_2026.rs`：逐日比对 `resources/cal/baseline/*.json`
- [x] T030 [P] [US2] `crates/cal/tests/day_bound.rs`：22:59/23:00/23:59/00:00/00:59 × ZICHU/ZIZHENG
- [x] T031 [P] [US2] `crates/cal/tests/jieqi_boundary.rs`：立春前后年柱、节气交接月柱、闰月月序 vs 月建
- [x] T032 [P] [US2] `crates/cal/tests/timezone_dst.rs`：跨时区、DST 重复/缺失、越界 `CAL_OUT_OF_RANGE`

### Implementation

- [x] T033 [US2] `crates/cal`：加载并索引农历月表/节气表，版本与 `PENDING` 状态随结果输出
- [x] T034 [US2] `crates/cal`：公历→农历、闰月标记、`LUNAR-MONTH-INDEX` 与 `JIEQI-MONTH-BRANCH` 分离
- [x] T035 [US2] `crates/cal`：年柱（立春）/月柱（节）/日柱（JDN）/时柱（五鼠遁）
- [x] T036 [US2] `crates/cal`：`ZICHU`/`ZIZHENG` 日界、旬首与旬空
- [x] T037 [US2] `crates/cal`：IANA 时区解析、DST 歧义/缺失策略（较早偏移 / 向后平移并标记）
- [x] T038 [US2] `crates/cal`：范围检查 1900-01-31～2100-12-31 与错误码
- [x] T039 [US2] `crates/cal`：生成 2025/2026 逐日基准 JSON（来源标注 `PENDING`，独立锚点交叉校验）
- [x] T040 [US2] `crates/cli cal`：输出 `CalResult`（含 `cal`/`cal_data`/`tzdata` 版本）

**Checkpoint**: US2 全部基准通过；历年立春/闰月/日界无误。

---

## Phase 5: User Story 3 — 规则与数据可溯源 (P1)

**Goal**: 每步有规则、数据、出处、层级；PENDING 不得发布正式结果。

**Independent Test**: 抽 30 步检查字段完整性；移除资源后显示待校核且不生成正式结果。

### Tests

- [x] T041 [P] [US3] `crates/rules/tests/manifest.rs`：checksum 失败/缺失/路径穿越/版本不兼容
- [x] T042 [P] [US3] `crates/meihua/tests/traceability.rs`：每步 `rule_ids/data_refs/source_ref/rule_level` 非空
- [x] T043 [P] [US3] `crates/ir/tests/pending.rs`：PENDING 规则输出“待校核”，不得标记正式结论

### Implementation

- [x] T044 [US3] `resources/data`：九色、生肖、八卦、64 卦、旺衰、旬空表，含 `status`/`source`/`version`
- [x] T045 [US3] `crates/rules`：规则注册表加载 ARS 附录 C/D 与规则样例，提供按引擎查询
- [x] T046 [US3] `crates/domain`：`StepLogBuilder` 强制必填字段；PENDING 状态传播
- [x] T047 [US3] `crates/app`：无资源/校验失败时阻断正式结果并提供修复提示
- [x] T048 [US3] `crates/cli verify`：逐资源输出 id/status/checksum_ok，失败退出码 3

**Checkpoint**: 任意步骤可溯源；损坏资源不会静默降级。

---

## Phase 6: User Story 4 — 隐私边界可验证 (P1)

**Goal**: 生辰仅内存；问事默认不持久化；HMAC 仅存 key_id 与摘要；删除后主库不可见。

**Independent Test**: 输入敏感值→扫描日志/DB/导出/快照；删除记录→主库不可见。

### Tests

- [x] T049 [P] [US4] `crates/domain/tests/bazi_no_serialize.rs`：`BaziMaterial` 无 `Serialize`，`Debug` 为 `<redacted>`
- [x] T050 [P] [US4] `crates/app/tests/privacy_scan.rs`：构造含敏感值的日志/DB/导出，扫描必须命中
- [x] T051 [P] [US4] `crates/store/tests/delete.rs`：删除档案/记录后主库不可见，另一档案不受影响

### Implementation

- [x] T052 [US4] `crates/domain`：`BaziMaterial` 仅内存类型、`Zeroize` on drop、禁止 Clone 到持久化路径
- [x] T053 [US4] `crates/seed`：HMAC-SHA256 + 本地密钥抽象（OS keychain 接口先以 trait + mock 实现），仅输出 `question_hmac`/`key_id`
- [x] T054 [US4] `crates/app`：日志脱敏层（禁止打印敏感字段），导出默认脱敏
- [x] T055 [US4] `crates/app`：`privacy_scan` 遍历文件/DB/JSON，定位文件与偏移
- [x] T056 [US4] `crates/cli privacy-scan`：命中退出码 4，输出命中类型（bazi/question/key）

**Checkpoint**: 隐私扫描 0 命中；密钥缺失时只提示、不覆盖。

---

## Phase 7: User Story 5 — 金标准一键验收 (P2)

**Goal**: 一条命令跑全部金标准；失败定位到 rule_id/输入/期望/实际/版本。

**Independent Test**: `cargo test --workspace` + `yitool verify`；故意改错规则能报出 rule_id。

### Tests

- [x] T057 [P] [US5] `crates/meihua/tests/golden.rs`：TC-MEIHUA-001（C=1,A=3→同人/动4/家人/姤）与 TC-MEIHUA-002（9,16→否/初爻/无妄）
- [x] T058 [P] [US5] `crates/seed/tests/distribution.rs`：九色/生肖卡方 + 六爻 6/7/8/9 = 1/8,3/8,3/8,1/8
- [x] T059 [P] [US5] `crates/app/tests/golden_runner.rs`：失败报告含 rule_id、输入、期望、实际、版本
- [x] T060 [US5] `crates/cli verify`/`scripts/check.sh`：统一金标准入口与汇总输出

### Implementation

- [x] T061 [US5] `crates/meihua`：九色/传统数字起卦、互卦、变卦、体用、旺衰
- [x] T062 [US5] `crates/meihua`：`jishen` 未冻结时输出 `PENDING` 并阻断方位象意正式结果（FR-012）
- [x] T063 [US5] `crates/meihua`：逐步 `StepLog`（起卦/互/变/体用/旺衰）
- [x] T064 [US5] `crates/cli meihua`：两模式输入校验与 JSON 输出
- [x] T065 [US5] `crates/seed`：`getrandom` + 拒绝采样（unbiased 1..n），固定模式不调用 OS 熵
- [x] T066 [US5] `tests/golden`：固化种子/CAL/梅花/旬空/分布 fixture 与期望值

**Checkpoint**: 一条命令全绿；失败可定位。

---

## Phase 8: User Story 6 — 多档案与设置底座 (P2)

**Goal**: 档案隔离、设置、历史、删除、备份/恢复、导出脱敏。

**Independent Test**: 两档案互不可见；删除 A 不影响 B。

### Tests

- [x] T067 [P] [US6] `crates/store/tests/isolation.rs`：两档案设置/记录互不可见
- [x] T068 [P] [US6] `crates/store/tests/schema.rs`：schema 版本、外键、WAL、`hmac_key_id` 存在

### Implementation

- [x] T069 [US6] `crates/store`：SQLite schema（profiles/settings/runs/run_steps/random_snapshots/notes/meta）
- [x] T070 [US6] `crates/store`：档案 CRUD、级联删除、默认档案唯一约束
- [x] T071 [US6] `crates/store`：Run/StepLog/RandomSnapshot 写入与按原版本读取
- [x] T072 [US6] `crates/app`：备份/恢复（SQLite backup API）与导出脱敏策略
- [x] T073 [US6] `crates/cli db init|check`：初始化、完整性检查、schema 版本输出

**Checkpoint**: 档案隔离与删除语义通过。

---

## Phase 9: Polish（跨故事收尾）

- [x] T074 [P] 性能基准：种子 P95<50ms、单引擎 P95<200ms、一年历法遍历 P95<500ms（记录设备/OS/样本）
- [x] T075 [P] 九色唯一输入：清理旧色制数据/规则/兼容入口，选择器仅提供九色（SC-010）
- [x] T076 [P] 文档同步：更新 `README.md`、`AGENTS.md`、`docs/00-PROJECT-ROADMAP.md` 当前状态
- [x] T077 [P] 更新 `docs/02-DECISION-REGISTER.md`：D-03/D-04/D-12/D-13/D-20 落地结果
- [x] T078 运行 `scripts/check.sh` 全绿并提交（按 `feat:`/`test:` 拆分提交）
- [x] T079 复盘：列出未冻结项与延期阶段，写入 `specs/001-phase0-foundation/quickstart.md` 验收记录

---

## Dependencies & Execution Order

```text
Phase 1 Setup
   └→ Phase 2 Foundational
        ├→ US1（种子/复现）
        ├→ US2（CAL）──→ US5（梅花金标准，依赖 CAL 月建）
        ├→ US3（溯源，依赖资源）
        ├→ US4（隐私，依赖 seed HMAC + store）
        └→ US6（存储底座，可与 US1/US2 并行）
US3 + US4 + US5 + US6 → Phase 9 Polish
```

- US2 是 US5 的硬前置（旺衰需要月建）。
- US4 的删除测试依赖 US6 schema；可先写 trait/mock。
- US1 与 US2 可并行；US3 依赖 Phase 1 资源。

## Parallel Example

```bash
# 资源与历法并行
task T005 & task T006
# US1 与 US2 测试先行
task T018 & task T028
# 两套起卦金标准并行
task T057 & task T058
```

## Implementation Strategy

1. **MVP（P1）**：Phase 1→2→US1→US2→US3→US4。完成后即可承诺“可复现、历法可信、可溯源、隐私可验证”。
2. **增量（P2）**：US5 金标准一键化 → US6 存储底座 → Polish。
3. 每个 Phase 结束运行 `cargo test --workspace`，失败不得进入下一 Phase。
4. 规则/数据变更必须同步升版本、补测试样例、更新 manifest。

## Notes

- `[P]` 任务不得修改同一文件。
- 所有输出必须附免责声明；禁止绝对断语。
- 未冻结规则输出 `PENDING`，界面/日志标“待校核”。
- 禁止在日志、错误、调试输出中打印生辰、问事原文、HMAC 密钥。
