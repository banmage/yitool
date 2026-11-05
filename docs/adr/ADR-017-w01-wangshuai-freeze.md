# ADR-017: W-01 四时旺衰表校核与冻结

- **状态：** 已批准
- **日期：** 2026-11-05
- **关联：** F-02、O-05、D-44、ARS 3.7/附录 D、SRS 附录 A、ADR-011
- **决策人：** 项目所有者
- **生效版本：** `W-01-v0.2`（资源）/ `RULE-v0.2`（规则）

> **后续状态：** `HEX-01` 已于 2026-11-05 校核为 `HEX-01-v0.2`（ADR-018）；本 ADR 中“HEX-01 尚未校核”的表述为当时状态。梅花整体结果仍因 `NJ-01`、`JL-01`、`IR-01` 等未校核而保持 `PENDING`。

## 背景

`W-01` 四时旺衰表此前仅有“月建地支 → 五行”的 12 条映射与一条生克关系说明，状态为 `PENDING`，没有完整的“逐月 × 逐行 → 旺相休囚死”表。ARS 3.7 要求完整表校核后冻结；在冻结前，`MEIHUA-WANGSHUAI-001`、`MEIHUA-JISHEN-001` 步骤均标 `PENDING`，梅花整体结果不得作为正式结论。

旺相休囚死属于五行旺衰的通行规则，可用月建五行与目标五行的生克关系逐项确定：

- 当令者旺（同我）；
- 令生者相（月建生目标）；
- 生令者休（目标生月建）；
- 克令者囚（目标克月建）；
- 令克者死（月建克目标）。

## 决策

1. **完整表：** `W-01-v0.2` 定义 12 个月建 × 5 行的逐项状态，共 60 个条目；月建五行固定为：
   - 寅、卯 → 木（春）；
   - 巳、午 → 火（夏）；
   - 申、酉 → 金（秋）；
   - 亥、子 → 水（冬）；
   - 辰、戌、丑、未 → 土（四季）。
2. **校核来源：** 以《梅花易数》传世本“月建五行生克”为语义来源，按旺相休囚死通行规则人工转录；不以运行时实现输出反填。
3. **金标准：** 新增 `tests/golden/w01_wangshuai.json`，保存 12 月建 × 5 行的期望状态；资源与 `domain::WuXing::relation_to` 必须同时与金标准一致。
4. **状态：** 资源 `W-01-v0.2` 记为 `VERIFIED`；规则 `MEIHUA-WANGSHUAI-001` 与 `MEIHUA-JISHEN-001` 升为 `VERIFIED`；`SRC-MEIHUA-005` 校勘状态升为 `VERIFIED`。
5. **不升全局版本：** 数据基线仍为 `DATA-v0.2`，规则集仍为 `RULE-v0.2`；本次是表内容校核与状态升级，不改变梅花起卦、互卦、变卦、体用算法。
6. **整体仍为 `PENDING`：** `HEX-01`、`NJ-01`、`JL-01`、`IR-01` 等尚未校核，梅花结果 `status` 仍为 `PENDING`、`formal_allowed=false`，不得输出正式结论。
7. **历史兼容：** 历史记录按原版本渲染，不因本次校核静默重算。

## 影响

- **数据：** `resources/data/wuxing_wangshuai.json` 增加 `elements`、`season`、`table` 与校核来源；`resources/data/rules.json` 同步规则与来源状态；重新生成 manifest/checksum。
- **代码：** `crates/meihua` 的旺衰步骤与忌神步骤状态改为 `VERIFIED`，整体结果仍为 `PENDING`。
- **测试：** 新增 `crates/rules/tests/wangshuai_crosscheck.rs`，校验金标准、资源、`domain` 生克实现三者一致。
- **文档：** SRS/ARS/数据字典/决策登记册/路线图同步 `W-01` 状态与来源。
- **合规：** 无变化；旺衰只作为 IR 解释权重，不改变卦象数字，不合成吉凶总分。

## 验证

- `cargo test -p yitool-rules --test wangshuai_crosscheck` 通过（60 个状态全部对账）。
- `cargo test --workspace` 通过。
- `cargo run -p yitool-cli -- verify --resources resources` 通过。
- `./scripts/check.sh` 全绿。
