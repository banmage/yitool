# ADR-021: 梅花核心规则校核与冻结

- **状态：** 已批准
- **日期：** 2026-11-06
- **关联：** M0.6、D-48、ARS 3.1–3.10、SRS 9.2、ADR-002、ADR-005、ADR-017、ADR-018
- **决策人：** 项目所有者
- **生效版本：** `MEIHUA-CORE-v0.2`（金标准）/ `RULE-v0.2`（规则）

## 背景

梅花五条核心规则 `MEIHUA-QIGUA-001`、`MEIHUA-QIGUA-002`、`MEIHUA-HUGUA-001`、
`MEIHUA-BIANGUA-001`、`MEIHUA-TIYONG-001` 此前状态均为 `PENDING`，出处
`SRC-MEIHUA-001`～`004` 的校勘状态也未升级。原有金标准只有 `TC-MEIHUA-001/002`
两个案例，无法证明全 64 卦、六种动爻、九色与传统数字双模式、Q/B 扰动路径都符合
ARS v0.2；在阶段 1 UI 与解释层接入前，需要先补齐可复核的校核证据。

## 决策

1. **校核对象：** 起卦（九色 `MEIHUA-QIGUA-001`、传统数字 `MEIHUA-QIGUA-002`）、
   互卦 `MEIHUA-HUGUA-001`、变卦 `MEIHUA-BIANGUA-001`、体用 `MEIHUA-TIYONG-001`。
2. **校核来源：** 以 ARS v0.2 3.2–3.6 的公式与《梅花易数》传世本起卦/互卦/变卦/体用
   语义为基线；规则层级保持原判定：基础取模映射为 L2，九色参与起卦与 Q/B 扰动为 L3。
   `VERIFIED` 只表示实现与 ARS 定义、金标准一致，不表示九色映射或 Q/B 扰动属于
   典籍原义，不得把 L3 冒充 L1。
3. **金标准：** 新增 `tests/golden/meihua_core.json`（`MEIHUA-CORE-v0.2`），保存
   17 个案例，覆盖：
   - 九色与传统数字两种模式；
   - 初爻至上爻全部六种动爻；
   - 上下卦同卦、8/8 边界取模、动爻在体卦/用卦两侧；
   - Q 扰动、B 扰动与二者为 0 的默认路径。
4. **独立复算：** 新增 `crates/meihua/tests/core_crosscheck.rs`，测试内按八经卦三爻位
   与 `HEX-01-v0.2` 卦名独立推导本卦、互卦、变卦、体用，再与金标准和 `meihua::cast`
   三方比对；不得复用 `cast` 的中间结果，也不以运行时输出反填金标准。
5. **全覆盖属性测试：** 对 8×8 上下卦 × 6 动爻共 384 种组合逐项比对；并验证 Q/B=0 时
   传统数字模式与九色模式结果等价（8×8 组合）。
6. **状态：** 五条核心规则升为 `VERIFIED`，`SRC-MEIHUA-001`～`004` 校勘状态升为
   `VERIFIED`；对应步骤日志状态由 `OK` 升为 `VERIFIED`。
7. **不升全局版本：** 数据基线仍为 `DATA-v0.2`，规则集仍为 `RULE-v0.2`；本次只校核
   规则内容与状态，不改变公式、资源字段或引擎输入输出结构。
8. **整体仍为 `PENDING`：** `IR-01`、`MEIHUA-INTERPRET-01`、`JL-01` 等尚未完成，
   `RULE-v0.2` 也未整体冻结，因此梅花结果 `status` 仍为 `PENDING`、
   `formal_allowed=false`，解释文字不得作为正式结论。核心卦象步骤为 `VERIFIED` 与
   整体结果为 `PENDING` 是两个层级，不得混同。
9. **历史兼容：** 历史记录按原版本渲染，不因本次校核静默重算。

## 理由

- 只有独立 oracle 与全组合覆盖，才能发现取模边界、同卦互卦、动爻换体用等手工样例
  容易遗漏的错误。
- 把 Q/B 扰动纳入金标准，可避免“默认路径通过、启用增强后静默偏移”的回归。
- 保留 L3 标注并区分 `VERIFIED`/`FROZEN`，符合项目“未冻结规则不得发布正式结果”的
  底线。

## 影响

- **数据：** `resources/data/rules.json` 中五条核心规则与四个出处升为 `VERIFIED`，
  `test_refs` 覆盖 `TC-MEIHUA-001`～`017`；重新生成 manifest/checksum。
- **代码：** `crates/meihua` 的起卦、互卦、变卦、体用步骤显式标记 `VERIFIED`，
  整体结果仍为 `PENDING`、`formal_allowed=false`。
- **测试：** 新增 `crates/meihua/tests/core_crosscheck.rs` 与
  `tests/golden/meihua_core.json`；`traceability.rs` 断言六步均 `VERIFIED` 但整体
  不得转为正式结果。
- **文档：** SRS/ARS/数据字典/决策登记册/路线图/README/AGENTS/阶段 0 验收记录同步。
- **合规：** 无变化；本次不生成吉凶总分，不改变免责声明与禁用词扫描。

## 验证

- `cargo test -p yitool-meihua` 通过（core_crosscheck 3 项、traceability 4 项、
  单元测试 5 项；含 384 种全组合与 64 种双模式等价）。
- `cargo test --workspace` 通过。
- `cargo run -p yitool-cli -- verify --resources resources` 通过。
- `./scripts/check.sh` 全绿。
