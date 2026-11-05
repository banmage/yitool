# Quickstart: 阶段 0 验收指南

**Date**: 2026-11-03
**Purpose**: 用最小命令验证阶段 0 的核心能力；本文件不包含实现代码。

---

## 1. 前置条件

```bash
# Rust stable（本机已安装 1.99.1）
rustc --version
cargo --version

# 项目依赖（由 Cargo 自动拉取）
# 阶段 0 不需要 WebKitGTK / Node（GUI 属阶段 1）
```

## 2. 一键测试

```bash
cargo test --workspace
```

预期：

- 全部单元测试、金标准测试、资源校验、隐私扫描通过。
- 任一失败输出 `rule_id`、输入、期望值、实际值。
- 测试运行时间 < 2 分钟（不含首次编译）。

## 3. 历法验证

```bash
cargo run -p yitool-cli -- cal \
  --datetime "2026-11-03T12:00:00+08:00" \
  --timezone "Asia/Shanghai" \
  --day-bound ZICHU
```

预期关键字段：

```json
{
  "lunar": {"year": 2026, "month": 9, "day": 25, "leap": false},
  "ganzhi": {"year": "丙午", "month": "戊戌", "day": "辛巳", "hour": "甲午"},
  "xun": {"xun_head": "甲戌", "kong_branches": ["申", "酉"]},
  "jieqi_month_branch": {"branch": "戌", "element": "土", "jieqi": "寒露"},
  "status": "PENDING"
}
```

立春边界：

```bash
cargo run -p yitool-cli -- cal \
  --datetime "2026-02-04T10:00:00+08:00" \
  --timezone "Asia/Shanghai" --day-bound ZICHU
```

预期：农历 2025-12-17；年柱 丙午（立春后）；月柱 庚寅；日柱 己酉。

闰月：

```bash
cargo run -p yitool-cli -- cal \
  --datetime "2025-08-15T12:00:00+08:00" \
  --timezone "Asia/Shanghai" --day-bound ZICHU
```

预期：农历 2025 闰六月（`leap=true`），`lunar_index=6`；节气月建仍按节气输出。

## 4. 梅花金标准

九色（TC-MEIHUA-001）：

```bash
cargo run -p yitool-cli -- meihua \
  --color 1 --aux 3 \
  --datetime "2026-11-03T12:00:00+08:00" \
  --timezone "Asia/Shanghai" --day-bound ZICHU
```

预期：

- 本卦：上乾下离 = 天火同人
- 动爻：4
- 变卦：上巽下离 = 风火家人
- 互卦：上乾下巽 = 天风姤

传统数字（TC-MEIHUA-002）：

```bash
cargo run -p yitool-cli -- meihua --traditional 9 16 \
  --datetime "2026-11-03T12:00:00+08:00" \
  --timezone "Asia/Shanghai" --day-bound ZICHU
```

预期：

- 本卦：上乾下坤 = 天地否
- 动爻：初爻（1）
- 变卦：天雷无妄

## 5. 种子链

```bash
cargo run -p yitool-cli -- seed \
  --profile family-a \
  --run-time "2026-11-03T12:00:00+08:00" \
  --timezone "Asia/Shanghai" --day-bound ZICHU \
  --mode FIXED --color 1 --aux 3
```

验证：

- 同参数运行两次，`base_seed/draw_seed/run_seed` 完全一致。
- 修改 `--color`，`base_seed/draw_seed` 不变，输入快照与 `run_seed` 变化。
- 输出不含生辰与问事原文。

## 6. 资源校验

```bash
cargo run -p yitool-cli -- verify --resources resources
```

预期：所有资源 `checksum_ok=true`；`PENDING` 资源显示状态但不阻断。

故意修改一个资源文件后再运行，预期退出码 3，`all_ok=false` 且对应条目 `checksum_ok=false`。

## 7. 隐私扫描

```bash
cargo run -p yitool-cli -- privacy-scan \
  --db /tmp/yitool-test.db \
  --logs /tmp/yitool-logs \
  --exports /tmp/yitool-exports \
  --needle "1990-01-01T12:00:00+08:00"
```

预期：`hits=[]`；命中退出码 4。

## 8. 数据库

```bash
cargo run -p yitool-cli -- db init --path /tmp/yitool-test.db
cargo run -p yitool-cli -- db check --path /tmp/yitool-test.db
```

预期：创建 `profiles/settings/runs/run_steps/random_snapshots/notes/meta`；`runs.hmac_key_id` 存在；WAL 与外键开启；`schema_version=DB-v0.2`，首次建库 `migration.applied=[DB-v0.1, DB-v0.2]`。对旧库升级时输出 `migration.backup` 备份路径。

## 9. 性能冒烟

```bash
cargo test -p yitool-app --test perf -- --nocapture
```

预期：种子 P95 < 50ms、单引擎 P95 < 200ms、一年逐日遍历 P95 < 500ms、1000 条历史列表 P95 < 100ms、单条历史可见性 P95 < 10ms（本机 debug 基线，见 ADR-010；输出含 OS/架构与数据规模）。

## 10. 阶段 0 验收清单

- [x] `cargo test --workspace` 全绿
- [x] 九色与传统数字金标准通过
- [x] 2026-11-03 / 2026-02-04 / 2025 闰月样例通过
- [x] 种子链两次运行一致；问事 HMAC 不改变共同随机条件
- [x] 资源 checksum 校验通过
- [x] 隐私扫描 0 命中；SQLite 明文可被扫描命中
- [x] DB schema 含 `hmac_key_id`；备份/恢复与档案隔离通过
- [x] 夏令时重复/缺失按 ADR-006 解析并输出 `selected_offset`
- [x] Q/B 扰动按 ADR-005 显式启用、范围校验、非法材料报错
- [x] 2025/2026 逐日基准对照通过（730 天）
- [x] HKO 1901–2100 逐日交叉校核：农历 0 差异；节气 6 条午夜边界容差（`CAL-DATA-v0.2`）
- [x] 文档版本/数据字典/决策登记同步

## 11. 验收记录

**执行日期**：2026-11-03
**环境**：Linux x86_64；Intel Core i5-5200U / 4GB RAM；Rust 1.99.1；SQLite bundled；`chrono-tz` tzdata 2025b。
**命令**：`./scripts/check.sh`

| 项目 | 结果 |
|---|---|
| `cargo fmt --check` | 通过 |
| `cargo clippy -D warnings` | 通过 |
| `cargo test --workspace` | 全部通过（含 730 天基准、金标准、隐私、分布、性能） |
| `yitool verify` | 13 个资源 checksum 全部通过 |
| `yitool db init` | schema `DB-v0.2`，WAL，外键开启 |
| `yitool privacy-scan` | 0 命中 |
| CAL 锚点 | 2026-11-03 → 农历 09-25 / 丙午 戊戌 辛巳 甲午 / 甲戌旬空申酉 |
| 性能基线 | 种子 0.46ms、梅花 0.19ms、CAL 一年 0.39ms、1000 历史列表 2.23ms、单条查询 0.06ms（P95，debug） |
| 梅花金标准 | TC-MEIHUA-001、TC-MEIHUA-002 通过 |

**遗留**：HEX/NJ/W 等规则资源仍为 `PENDING`，需易学评审交叉校核；CAL 规则集已于 2026-11-05 冻结为 `CAL-v0.2` / `CAL-RULE-v0.2`（ADR-016）；Q/B 材料与 HMAC 密钥语义已由 ADR-005/007 冻结，系统 Keychain 接入延期至阶段 1。

### 11.1 CAL 数据交叉校核（2026-11-04）

**数据**：`CAL-DATA-v0.2`（sxtwl 2.0.7 / VSOP87 + ELP2000）。
**来源**：香港天文台《1901–2100 年公曆與農曆日期對照表》。
**命令**：`python3 scripts/crosscheck_cal_hko.py --no-download`
**报告**：`docs/reports/CAL-HKO-CROSSCHECK-v0.2.md`

| 项目 | 样本 | 严格不一致 | 边界容差 | 结果 |
|---|---:|---:|---:|---|
| 农历年/月/闰月/日 | 73049 | 0 | 0 | PASS |
| 节气日期（HKO → CAL） | 4800 | 0 | 6 | PASS |
| 节气日期（CAL → HKO） | 4872 | 0 | 6 | PASS |

**说明**：6 条差异均为交节时刻距午夜 ≤ 712 秒（最接近 4 秒），符合 HKO 页脚声明的算法边界情形；按 D-41 单列容差，不计严格失败。节气秒级时刻仍由 sxtwl 计算，尚无独立的秒级权威基准。

### 11.2 资源 manifest 启动校验（ADR-012，2026-11-05）

**命令**：`cargo test -p yitool-app --test manifest_guard`、`cargo run -p yitool-cli -- verify --resources resources`

| 项目 | 结果 |
|---|---|
| `verify` | 13 个资源 checksum 全部通过，退出码 0 |
| 篡改资源启动加载 | 返回 `AppError::ResourcesIntegrity`，错误信息含失败 `id` 与数量 |
| 清单版本不兼容 | 清单 `version != DATA-v0.2` 时拒绝加载，信息含期望/实际版本 |
| 路径穿越 (`../`) | `ManifestError::UnsafePath`，拒绝加载 |
| 资源缺失 | 报告 `checksum_ok=false` 且 `error` 非空 |

**说明**：启动即校验；checksum 通过仅表示未被篡改，`PENDING` 条目仍须标“待校核”。

### 11.3 CAL 规则决策冻结（ADR-013～015，2026-11-05）

| ADR | 决策 | 结果 |
|---|---|---|
| ADR-013 | 日界、日柱基准、支持范围、默认时区 | `ZICHU` 默认；JDN 连续纪日 + 1900-01-31 甲辰 / 2000-01-01 戊午双锚；`1900-01-31`～`2100-12-31`；默认 `Asia/Shanghai` |
| ADR-014 | CAL 数据源、双年基准、午夜边界容差 | sxtwl 2.0.7 预计算表 + Rust 纯查询；HKO 双源校核；2025/2026 回归快照；边界容差 900 秒 |
| ADR-015 | tzdata 版本与升级 | chrono-tz 0.10.4 内置 tzdata `2025b`；离线、历史保留原版本、升级需回归 |

**说明**：`CAL-BASE-2025/2026` 是由 `CAL-DATA-v0.2` 派生的回归快照，文件内仍标 `PENDING`，不得作为独立权威基准；CF-08 的权威证据是 HKO 交叉校核报告。CAL 文档升版冻结已完成（见 11.4）；发布构建 tzdata 复核仍为后续动作。

### 11.4 CAL 规则集冻结（ADR-016，2026-11-05）

| 项目 | 结果 |
|---|---|
| 文档版本 | `docs/历法服务规范 CAL v0.2.txt`，`CAL v0.2（规则冻结版）` |
| 规则集 | `CAL-RULE-v0.2` |
| 数据版本 | `CAL-DATA-v0.2`（不变） |
| 代码版本 | `VersionSet.cal = "CAL-v0.2"` |
| 冻结项 | CF-01～CF-09 关闭；CF-10 延期阶段 4 |
| 数据表 | `LUNAR-MONTH-INDEX`、`JIEQI-MONTH-BRANCH`、`CAL-TZDATA` → `VERIFIED`；`CAL-BASE-2025/2026` 仍为 `PENDING`（回归快照） |
| 历史兼容 | 旧记录按原 `CAL-v0.1` 渲染，不静默重算 |

**说明**：CAL 冻结只覆盖历法基础规则；`NJ-01`、`JL-01` 等未校核时，对应引擎仍须标“待校核”。

### 11.5 W-01 四时旺衰表校核（ADR-017，2026-11-05）

| 项目 | 结果 |
|---|---|
| 资源版本 | `W-01-v0.2`，状态 `VERIFIED` |
| 覆盖范围 | 12 月建 × 5 行，共 60 个旺相休囚死条目 |
| 来源 | ARS 3.7；《梅花易数》传世本「月建五行生克」；当令者旺、令生者相、生令者休、克令者囚、令克者死 |
| 金标准 | `tests/golden/w01_wangshuai.json` |
| 对账 | `crates/rules/tests/wangshuai_crosscheck.rs`：金标准 ↔ 资源 ↔ `domain::WuXing::relation_to` |
| 规则状态 | `MEIHUA-WANGSHUAI-001`、`MEIHUA-JISHEN-001` → `VERIFIED` |
| 整体结果 | 仍为 `PENDING`、`formal_allowed=false`（规则集整体未冻结） |

**说明**：`W-01-v0.2` 只校核旺衰表本身，不改变起卦、互卦、变卦、体用算法，也不解除其他未校核规则对正式结果的阻断。

### 11.6 HEX-01 六十四卦八宫表校核（ADR-018，2026-11-05）

| 项目 | 结果 |
|---|---|
| 资源版本 | `HEX-01-v0.2`，状态 `VERIFIED` |
| 覆盖范围 | 8 宫 × 8 卦，共 64 条；含宫序号、宫五行、宫内序、类型、世应 |
| 来源 | ARS 1.6/附录 A；《京氏易传》八宫卦序；通行八宫卦象次序歌与世应规则 |
| 金标准 | `tests/golden/hex01_bagong.json` |
| 对账 | `crates/rules/tests/hex_bagong_crosscheck.rs`：金标准 ↔ 资源 ↔ `domain` 卦名 ↔ 八宫逐爻生成 |
| 字段补齐 | `palace_index`（乾1…坤8）、`palace_element`（乾/兑金、离火、震/巽木、坎水、艮/坤土） |
| 整体结果 | 纳甲/焦氏仍受 `NJ-01`、`JL-01`、`IR-01` 等未校核规则约束 |

**说明**：`HEX-01-v0.2` 只校核八宫结构与世应，不改变起卦、互卦、变卦、体用算法，也不解除纳甲/焦氏/解释引擎的“待校核”状态。
