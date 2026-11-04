# Phase 0 Research: 阶段 0 — 可信基础与可复现内核

**Date**: 2026-11-03
**Status**: Complete（阶段 0 所需未知项已收敛）

---

## R-01 技术栈与工程形态

- **Decision**: Rust workspace + CLI 验收入口；阶段 1 再接 Tauri 2 + React/TS。
- **Rationale**: 与 ADR-001/TDS 推荐一致；阶段 0 不被 WebKitGTK 等 GUI 依赖阻塞；核心库可独立测试、可被 Tauri 命令层复用。
- **Alternatives considered**:
  - Tauri 2 + Python 引擎：原型快，但打包、隐私边界、性能与类型安全成本更高。
  - 纯 Python/PySide6：最快，但后续迁移 Tauri 需重写。

## R-02 CAL 实现方式

- **Decision**: 预计算表 + 纯 Rust 查询。`resources/cal/lunar_months.json` 存农历月首（JDN、农历年/月、是否闰月、天数）；`resources/cal/jieqi.json` 存 24 节气精确时刻（UTC）。
- **Rationale**: CAL v0.1 允许“预计算表”或“天文算法”，二者都须通过逐日基准；阶段 0 优先可验证性与确定性，天文算法留作后续升级（`CAL_EPHEMERIS`）。
- **Alternatives considered**:
  - 现成 Rust 农历 crate：依赖行为与版本不可控，难以满足 `CAL-v0.1` 的版本/校验要求。
  - 自研天文算法：精度与边界（历史时区、节气秒级）成本高，阶段 0 不合适。
- **Data source**: `sxtwl` 2.0.7（Python，VSOP87/ELP2000）生成 `CAL-DATA-v0.2`；已用香港天文台 1901–2100 对照表逐日交叉校核，状态 `VERIFIED`（节气秒级时刻仍属计算值，6 条午夜边界按 D-41 处理）。

## R-03 农历表结构

- **Decision**: 每个农历月一条记录：`{start_jdn, year, month, leap, days}`；按 `start_jdn` 升序二分查找。
- **Rationale**: 公历→农历只需定位所在月；农历→公历只需按 `(year, month, leap)` 查表；闰月与月长显式存储。
- **Boundary**: 支持 1900-01-31～2100-12-31；超出返回 `CAL_OUT_OF_RANGE`。

## R-04 节气与干支

- **Decision**:
  - 节气表存 24 个节气时刻（UTC）；月建只按 12 个“节”切换。
  - 年柱以立春时刻为界；月柱以节气时刻为界；日柱用 JDN + 基准；时柱用五鼠遁。
  - 旬空用 `xun_head_branch` 公式。
- **Rationale**: 与 CAL 5.1–5.5 一致；所有边界可按时刻而非整日判断。
- **Verification anchors**:
  - 1900-01-31 = 甲辰日（与 2000-01-01 戊午日相差 36494 天，`36494 mod 60 = 14`，`54-14=40=甲辰`）。
  - 2026-11-03 = 农历 2026-09-25，日柱 辛巳；2026-02-04 = 农历 2025-12-17，年柱 丙午、月柱 庚寅、日柱 己酉。

## R-05 日界

- **Decision**: 默认 `ZICHU`（23:00 换日）；`ZIZHENG` 00:00 换日；`effective_date` 用于日柱/旬空/日辰。
- **Rationale**: SRS 3.5 与 CAL 6.1 默认值；历史记录保留原 `day_bound`。
- **Edge**: 23:00–23:59 在 `ZICHU` 下归次日；00:00–00:59 属子时但日柱已按 `ZICHU` 进入次日。

## R-06 时区与夏令时

- **Decision**: 使用 `chrono-tz`；所有接口显式传 IANA 时区；DST 重复取较早偏移，缺失向后平移 1 小时（`02:30 → 03:30`），并标记 `dst_ambiguity/dst_gap`。
- **Rationale**: CAL 6.4 要求明确提示；IANA 数据版本由 `chrono-tz` 提供并记录。
- **Alternatives**: 固定 +08:00 简化实现——不满足跨时区与历史时区要求。

## R-07 种子与随机

- **Decision**:
  - `base_seed = SHA256(canonical_json)`；`draw_seed = HMAC-SHA256(base_seed, "DRAW|purpose")`；`run_seed = HMAC-SHA256(base_seed, "RUN|" + snapshot_digest)`；`sub_seed = HMAC-SHA256(run_seed, engine|step|purpose)`。
  - 固定模式不调用 OS 熵；摇卦模式用 `getrandom` + 拒绝采样；快照存原始比特。
  - 基础种子、快照、日志不含生辰；问事仅 HMAC。
- **Rationale**: ARS 2 / TDS 6；消除循环依赖。
- **Open**: A-03（A/B 固定派生）与 A-04（Q/B 材料）需在实现前按 D-06/D-07 关闭；阶段 0 先实现普通种子链，A/B 与 Q/B 以 `PENDING` 接口占位。

## R-08 HMAC 密钥

- **Decision**: 阶段 0 定义 `KeyProvider` trait；Linux 用文件权限 0600 的密钥文件作开发实现；生产实现接 OS Keychain/Secret Service（阶段 1）；`runs` 表加 `hmac_key_id`。
- **Rationale**: 不把密钥写 DB/日志；阶段 0 可在 CI 注入临时密钥。
- **Alternatives**: 直接硬编码测试密钥——不可接受，仅测试 fixture 使用并明确标记。

## R-09 资源与 checksum

- **Decision**: `resources/manifests/data.manifest.json` 记录每个文件 `id/version/status/source/sha256`；加载时校验；`PENDING` 可加载但相关结果标“待校核”。
- **Rationale**: TDS 4.3；防止损坏/篡改。
- **Implementation**: `scripts/gen_manifest.py` 生成；测试比对。

## R-10 梅花内核

- **Decision**: 按 ARS 3 实现九色与传统数字两套入口；`Q_mod/B_mod` 默认 0；互卦默认取中间四爻；变卦翻动爻；体用按动爻 1–3/4–6；旺衰按 W-01。
- **Rationale**: 与 ARS 金标准 TC-MEIHUA-001/002 对齐。
- **Open**: A-15（忌神/弱势卦位）在 ARS 未定义；阶段 0 输出 `PENDING_JISHEN`，不生成方位正式结果。

## R-11 隐私扫描

- **Decision**: 在 `app::privacy` 实现扫描器：输入候选敏感值（生辰原始/派生、问事原文、HMAC key），扫描日志字符串、SQLite 文件字节、导出 JSON；命中即失败。
- **Rationale**: SRS 8.3 / ARS TC-PRIV-001；CI 可执行。
- **Limitation**: 无法证明“不存在未知派生值”；以已知输入 + 字段白名单 + 结构审查三重保证。

## R-12 合规扫描

- **Decision**: 阶段 0 实现禁用词表与免责声明检查；输出命中即拒绝。词表区分“绝对断语”与“规则命中”术语，后者走白名单。
- **Rationale**: IR 9.1/9.2；避免误杀。

## R-13 数据基线

- **Decision**: 阶段 0 生成 2025（闰六月）与 2026 两年逐日基准；状态 `PENDING`（回归快照，不作为权威基准），由 `CAL-DATA-v0.2` 派生并经 HKO 交叉校核。
- **Rationale**: A-07 指出 2026 无闰月；双年覆盖闰月与立春/节气边界。
- **Size**: 730 天 × 字段，JSON < 1MB。

## R-14 测试与 CI

- **Decision**: `cargo test --workspace` 为唯一入口；金标准 fixture 放 `tests/golden`；CI 运行 `fmt --check`、`clippy -D warnings`、`test`、`gen_manifest --check`、`privacy_scan`。
- **Rationale**: 一条命令可验收，失败定位 `rule_id`。
- **Performance**: 用 `criterion` 可选；阶段 0 先用简单计时测试，记录环境。

## R-15 未决项处理

| 项 | 处理 |
|---|---|
| A-03 A/B 固定派生 | 阶段 0 不实现 A/B；接口留 `AbMode` 占位，D-06 关闭后实现 |
| A-04 Q/B 材料 | 默认 0；启用接口标 `PENDING`，D-07 关闭后实现 |
| A-09 `hmac_key_id` | 阶段 0 schema 直接加列 |
| A-10 `bazi_material` 序列化 | 不实现 `Serialize`；CLI 不接受生辰 JSON |
| A-11 变卦六亲 | 阶段 3 前由 D-09 裁决；阶段 0 不实现纳甲 |
| A-15 忌神算法 | 输出 `PENDING_JISHEN`；D-11 关闭后补 |
| D-03/D-04/D-12/D-13/D-14 | 本计划按建议值实现，ADR 批准后改常量 |
