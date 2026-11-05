# ADR-018: HEX-01 六十四卦八宫表校核与冻结

- **状态：** 已批准
- **日期：** 2026-11-05
- **关联：** F-03、D-45、ARS 1.6/附录 A、SRS 附录 A、ADR-017
- **决策人：** 项目所有者
- **生效版本：** `HEX-01-v0.2`（资源）/ `RULE-v0.2`（规则）

## 背景

`HEX-01` 六十四卦八宫表此前状态为 `PENDING`，资源仅含卦名、上下卦、宫位、宫内序、卦类型与世应，缺少 ARS 1.6 要求的 `palace_element` 与 `palace_index` 字段，且没有与通行八宫表逐宫对账的自动化证据。该表是纳甲装卦、变卦宫位与焦氏索引的共同基础，不校核则相关引擎不能发布正式结果。

## 决策

1. **完整表：** `HEX-01-v0.2` 固定 8 宫 × 8 卦共 64 条，字段为：
   - `palace`、`palace_index`（乾1、兑2、离3、震4、巽5、坎6、艮7、坤8）；
   - `palace_element`（乾/兑金、离火、震/巽木、坎水、艮/坤土）；
   - `palace_seq`（宫内序 1..8）、`name`、`upper`、`lower`；
   - `palace_kind`（`PURE|FIRST|SECOND|THIRD|FOURTH|FIFTH|WANDERING|RETURNING`）、`shi_yao`、`ying_yao`。
2. **校核来源：** 以 ARS v0.2 附录 A 为基线，对照《京氏易传》八宫卦序、通行“八宫卦象次序歌”与世应规则逐宫核验：
   - 乾宫：乾、姤、遁、否、观、剥、晋、大有；
   - 坤宫：坤、复、临、泰、大壮、夬、需、比；
   - 震宫：震、豫、解、恒、升、井、大过、随；
   - 艮宫：艮、贲、大畜、损、睽、履、中孚、渐；
   - 离宫：离、旅、鼎、未济、蒙、涣、讼、同人；
   - 坎宫：坎、节、屯、既济、革、丰、明夷、师；
   - 巽宫：巽、小畜、家人、益、无妄、噬嗑、颐、蛊；
   - 兑宫：兑、困、萃、咸、蹇、谦、小过、归妹。
3. **世应规则：** 本宫纯卦世6应3；一至五世分别世1..5、应4..2；游魂世4应1；归魂世3应6。
4. **金标准：** 新增 `tests/golden/hex01_bagong.json`，保存八宫成员顺序、卦类型与世应模式；资源与 `domain` 卦象实现必须同时与金标准一致。
5. **独立生成校验：** 测试按八宫生成规则从本宫纯卦逐爻变出 64 卦，与金标准成员顺序逐项比对，避免只做“资源自证”。
6. **状态：** 资源 `HEX-01-v0.2` 记为 `VERIFIED`；`HEX-01` 作为数据表不再列为 `PENDING`。
7. **不升全局版本：** 数据基线仍为 `DATA-v0.2`，规则集仍为 `RULE-v0.2`；本次只补齐字段、校核内容与状态。
8. **整体仍为 `PENDING`：** `NJ-01`、`JL-01`、`IR-01` 等尚未校核，纳甲/焦氏/解释引擎不得发布正式结论。
9. **历史兼容：** 历史记录按原版本渲染，不因本次校核静默重算。

## 影响

- **数据：** `resources/data/hex64.json` 增加 `id`、`palace_index`、`palace_element`，版本升为 `HEX-01-v0.2`，状态升为 `VERIFIED`；重新生成 manifest/checksum。
- **测试：** 新增 `crates/rules/tests/hex_bagong_crosscheck.rs`，覆盖金标准完整性、资源对账、上下卦与 `domain` 卦名一致、八宫逐爻生成一致。
- **文档：** SRS/ARS/数据字典/决策登记册/路线图/README/AGENTS/阶段 0 验收记录同步。
- **合规：** 无变化；八宫表只提供结构索引，不改变吉凶判断与解释措辞。

## 验证

- `cargo test -p yitool-rules --test hex_bagong_crosscheck` 通过（4 项，覆盖 64 卦）。
- `cargo test --workspace` 通过。
- `cargo run -p yitool-cli -- verify --resources resources` 通过。
- `./scripts/check.sh` 全绿。
