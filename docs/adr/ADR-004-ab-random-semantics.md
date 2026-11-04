# ADR-004: A/B 统一条件对照的随机语义

- **状态：** 已批准
- **日期：** 2026-11-04
- **关联：** A-03、D-06、SRS 3.3.5/6.1、ARS 2.5/2.8
- **决策人：** 项目所有者
- **生效版本：** SEED-v2

## 背景

ARS 2.5 原公式把 `A_question_hmac`/`B_question_hmac` 与 `AB_A`/`AB_B` 标签放进 A/B 的 `run_seed`，而 ARS 2.8 的固定模式六爻由 `run_seed` 派生。这样在“统一条件对照”下，仅因问事文本不同就可能得到不同六爻，违背 SRS 3.3.5“不得因问事文本不同而派生不同随机条件”。

## 决策

1. A/B 统一条件对照共用同一 `ab_base_seed`、`ab_draw_seed` 与 `shared_random_snapshot`。
2. 用于随机派生的公共快照摘要 **不包含** 问事 HMAC、生辰材料或 A/B 标签：
   ```text
   common_snapshot_digest = digest(input_snapshot 去掉 question_hmac/bazi 后)
   A_run_seed = B_run_seed = HMAC(ab_base_seed, "AB|" + common_snapshot_digest)
   ```
3. 问事 HMAC 只用于各自的历史关联与解释上下文；是否保存问事与是否启用 Q/B 扰动是两个独立开关。
4. 独立推演模式仍各自使用普通种子链，不适用本 ADR。
5. 固定模式下 A/B 六爻必须一致；摇卦模式下必须共用同一随机快照才能对照。

## 理由

- 与 SRS 3.3.5、6.1 及现有 `seed` 实现一致：代码已将 `question_hmac`/`bazi_present` 从公共快照中排除，并有回归测试。
- 保证“统一条件”可验证：A/B 的差异只能来自方案解释输入，不来自随机条件。
- 不牺牲隐私：HMAC 仍可保存在各自输入快照中用于历史关联。

## 影响

- **文档：** 修订 ARS 2.5；在 SRS 3.3.5/6.1 保持“共同随机快照”表述；更新决策登记册与文档分析状态。
- **代码：** 无需行为变更；现有 `question_does_not_change_common_random_conditions` 测试即验证本决策。
- **数据/金标准：** A/B 固定模式金标准应断言两侧六爻一致。
- **隐私/合规：** 问事原文仍不进入种子或日志；HMAC 仅用于关联。

## 验证

- `cargo test -p yitool-seed --test determinism` 通过。
- 同一 A/B 基础种子、不同问事 HMAC 时，`base_seed`、`draw_seed`、`snapshot_digest`、`run_seed` 全部相同。
- 固定模式下 A/B 六爻一致；摇卦模式下随机快照摘要一致。
