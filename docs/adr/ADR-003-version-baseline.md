# ADR-003: 统一版本基线与数据字典唯一化

- **状态：** 已批准
- **日期：** 2026-11-04
- **关联：** A-02、A-05、D-05、D-10
- **决策人：** 项目所有者
- **生效版本：** RULE-v0.2 / DATA-v0.2 / CAL-DATA-v0.2 / DD-v0.2

> **后续版本：** CAL 规则集已由 ADR-016 于 2026-11-05 升为 `CAL-v0.2` / `CAL-RULE-v0.2`；本 ADR 中 `CAL-RULE-v0.1（规则集仍待冻结）` 为当时状态。

## 背景

SRS 附录 B 的日志示例写 `RULE-v0.4`/`DATA-v0.4`，而 ARS 0.3、Rust 内核与资源 manifest 已统一使用 `RULE-v0.2`/`DATA-v0.2`。CAL 文档的示例仍写 `CAL-DATA-v0.1`，但当前校核数据为 `CAL-DATA-v0.2`。同时 SRS 附录 A、ARS 附录 D 与 `docs/04-DATA-DICTIONARY.md` 的表清单、状态和版本不一致，存在实现与文档双轨风险。

## 决策

1. 版本基线统一为：
   - `RULE_SET_VERSION = RULE-v0.2`
   - `DATA_VERSION = DATA-v0.2`
   - `CAL_DATA_VERSION = CAL-DATA-v0.2`
   - `CAL_RULE_SET = CAL-RULE-v0.1`（规则集仍待冻结）
2. `docs/04-DATA-DICTIONARY.md` 是数据表 ID、版本、状态与归属的**唯一索引**。
3. SRS 附录 A、ARS 附录 D 与 CAL 附录 B 只保留各自视角的摘要，必须注明“以数据字典为准”，不得自行维护另一套状态。
4. 文档版本号不得反向覆盖代码与资源版本；历史记录按生成时版本渲染，不静默重算。

## 理由

- Rust `VersionSet`、资源 manifest 与 CLI 验收输出已经使用 v0.2；阶段 0 尚无正式历史数据，改文档比改实现成本低。
- 单一数据字典可以避免 SRS/ARS/CAL 三份附录继续漂移。
- `CAL-DATA-v0.2` 已完成 HKO 1901–2100 交叉校核，文档继续写 v0.1 会误导复现与审计。

## 影响

- **文档：** 修正 SRS 附录 B、SRS 附录 A、ARS 附录 D、CAL 2.2/8.1/附录 B；更新数据字典版本说明与决策登记册。
- **代码：** 无行为变更；`VersionSet::default()` 与 `yitool verify` 输出保持 v0.2。
- **数据/金标准：** 资源版本与 checksum 不变。
- **隐私/合规：** 无变化。

## 验证

- 全仓库检索不再出现 `RULE-v0.4`、`DATA-v0.4`、`CAL-DATA-v0.1`（历史报告除外）。
- `cargo run -p yitool-cli -- verify --resources resources` 输出版本为 `RULE-v0.2`、`DATA-v0.2`、`CAL-DATA-v0.2`。
- `docs/04-DATA-DICTIONARY.md` 的状态与 `resources/manifests/data.manifest.json` 一致。
