# ADR-014: CAL 预计算数据、双年逐日基准与午夜边界容差

- **状态：** 已批准
- **日期：** 2026-11-05
- **关联：** D-19、D-20、D-41、A-07、CF-01/CF-02/CF-08、CAL 2.2/3.2/8.1/8.2/8.3
- **决策人：** 项目所有者
- **生效版本：** `CAL-DATA-v0.2` / `CAL-BASELINE-v0.2`

## 背景

CAL 2.2 允许“预计算表”或“天文算法”两种实现，但要求冻结前必须确定一种并记录 `CAL_EPHEMERIS`。阶段 0 已用 sxtwl 2.0.7 生成 `CAL-DATA-v0.2`，并以香港天文台 1901–2100 对照表完成逐日交叉校核；同时 `resources/cal/baseline/2025.json`、`2026.json` 是由同一份数据派生的回归快照。需要明确：数据源、校核证据、基准快照的权威边界，以及 6 条午夜边界差异的判定口径，否则“基准通过”无法审计。

## 决策

1. **实现方式：** CAL 采用**预计算表 + Rust 纯查询**。农历月表与节气时刻表由 `scripts/gen_cal_precise.py` 调用 sxtwl 2.0.7（VSOP87/ELP2000）生成；运行时不计算天文历表、不联网、不下载历法数据。
2. **数据版本：** 当前冻结数据为 `CAL-DATA-v0.2`，资源文件为 `resources/cal/lunar_months.json` 与 `resources/cal/jieqi.json`，状态 `VERIFIED`。版本与 checksum 由 `resources/manifests/data.manifest.json` 固化。
3. **权威交叉校核：** 以香港天文台《1901–2100 年公曆與農曆日期對照表》为权威来源，sxtwl 为独立计算源，构成“权威源 + 计算源”双源校核。校核结果以 `docs/reports/CAL-HKO-CROSSCHECK-v0.2.md` 为准：
   - 农历年/月/闰月/日：73,049 天，严格不一致 0；
   - 节气日期（HKO → CAL）：4,800 条，严格不一致 0，边界容差 6；
   - 节气日期（CAL → HKO）：4,872 条，严格不一致 0，边界容差 6。
4. **逐日回归快照：** `resources/cal/baseline/2025.json` 与 `2026.json` 各 365 天，是**由 `CAL-DATA-v0.2` 派生的回归快照**，用于防止实现回归；它们不是独立权威基准，因此文件内 `status` 保持 `PENDING`，不得作为“外部历书来源”引用。CF-08 的验收证据是第 3 条的 HKO 交叉校核报告，而不是快照自身。
5. **基准年选择：** 2025 年含闰六月，用于覆盖闰月整月与月序；2026 年为当前使用年，用于覆盖当前节气/四柱。两年共 730 天，满足“至少一年逐日基准”的阶段 0 要求。
6. **午夜边界容差：** 仅当同时满足以下条件时，才把 HKO 与 CAL 的节气日期差异记为“边界容差”，不计入严格失败：
   - 同一节气名称；
   - 日期相差恰好 1 天；
   - CAL 交节时刻距最近午夜不超过 **900 秒（15 分钟）**。
   任一条件不满足即判 FAIL。容差条目必须在报告中逐条列出，不得只给总数。
7. **变更控制：** 修改农历月表或节气表必须：
   1. 升 `CAL-DATA` 版本；
   2. 重新生成 2025/2026 回归快照；
   3. 重跑 HKO 交叉校核并更新报告；
   4. 重新生成 manifest/checksum；
   5. 保留旧版本资源以按原版本渲染历史，不静默重算。
8. **已知限制：** HKO 表只给节气日期、不给秒级交节时刻；秒级时刻仍是 sxtwl 计算值，尚无独立秒级权威基准。该限制必须在报告与验收记录中保留，不得宣称“秒级已双源校核”。

## 影响

- **文档：** CAL 2.2 记录 `CAL_EPHEMERIS = sxtwl 2.0.7 / CAL-DATA-v0.2`；CAL 8.2 明确“回归快照不等于权威基准”；决策登记册 D-19/D-20/D-41 关闭。
- **代码：** `crates/cal` 只读取预计算表；`tools/gen-cal` 仅从已校核表派生回归快照，不得用低精度算法覆盖数据表。
- **数据/金标准：** 新增/修改 CAL 数据必须同步 `resources/cal/baseline/*.json`、`data.manifest.json` 与 `hko_crosscheck.rs` 固定断言；6 条边界容差条目固定为回归样例。
- **隐私/合规：** 历法数据不含个人信息；离线运行，无网络依赖。

## 验证

- `python3 scripts/crosscheck_cal_hko.py --no-download`：农历 0 差异、节气严格 0 差异、6 条边界容差，总结论 PASS。
- `cargo test -p yitool-cal --test hko_crosscheck`：数据版本、月表/节气表数量、3 条月首修正与 6 条边界容差固定通过。
- `cargo test -p yitool-cal --test baseline`：2025/2026 共 730 天逐日农历、月建、四柱、旬空一致。
- `cargo run -p yitool-cli -- verify --resources resources`：CAL 资源 checksum 通过。
