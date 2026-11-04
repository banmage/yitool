# ADR-011: 忌神五行与弱势卦位算法

- **状态：** 已批准
- **日期：** 2026-11-04
- **关联：** A-15、D-11、SRS 3.2.1/3.7/4.6、ARS 3.8、IR 3.4
- **决策人：** 项目所有者
- **生效版本：** `G-01-v0.2` / `MEIHUA-JISHEN-001`（`RULE-v0.2`）

## 背景

SRS 3.2.1、3.7 与 4.6 要求梅花推演输出“忌神五行、弱势卦位”，IR 3.4 与 IR 7 已按该输入设计柔性方位提醒；但 ARS 第 3 章此前只定义起卦、互卦、变卦、体用与旺衰，没有忌神/弱势卦位的计算规则（A-15）。若继续缺失，方位象意无法落地；若各引擎自行解释，又会破坏复现与跨版本一致性。

## 决策

1. **忌神五行：** 取“克体之行”，即
   ```text
   E_ji = controlled_by(element(ti))
   ```
   其中 `ti` 为体卦。例：体卦离火 → 忌神水；体卦乾金 → 忌神火。
2. **弱势卦位：** 取所有五行等于忌神五行的八卦，按 `G-01` 后天方位输出：
   ```text
   weak_positions = { trigram | element(trigram) = E_ji }
   ```
   方位与五行均来自 `G-01`；同一五行对应多个卦时全部输出，不擅自合并或挑选。
3. **输出结构：** `jishen` 至少包含 `element`、`weak_positions[]`（每项含 `trigram`、`direction`、`element`）、`rule_level` 与说明。`weak_positions` 顺序固定为 `G-01` 先天数 1..8。
4. **规则层级：** 该算法为本系统自定义的简化模型，标 **L3**；不得表述为典籍唯一流派或传统定论。
5. **不改变卦象：** 忌神/弱势卦位只作为解释输入，不改变起卦、互卦、变卦、体用与旺衰结果，也不参与任何吉凶总分。
6. **与旺衰的关系：** 本算法不依赖 `W-01` 月令旺衰表；但 `W-01` 尚未校核时，梅花整体结果仍为 `PENDING`，方位象意不得作为正式结论发布。
7. **合规措辞：** 输出只用于柔性提醒，禁止“必须避开”“绝对不能去”“必有灾祸”等绝对断语；忌神不等于危险，弱势卦位不等于禁忌方位。

## 影响

- **文档：** ARS 新增 3.8 `MEIHUA-JISHEN-001`；IR 3.4 标注输入来自该规则；D-11 关闭，A-15 关闭。
- **数据：** `G-01` 升为 `G-01-v0.2`，`bagua.json` 增加 `direction` 字段；`MEIHUA-JISHEN-001` 规则层级由 `PROVISIONAL` 改为 `L3`。
- **代码：** `domain::WuXing::controlled_by`、`domain::Trigram::direction`；`meihua::JishenOut`/`WeakPosition`；步骤日志 `MEIHUA-JISHEN` 输出结构化结果。
- **测试：** 新增 `bagua_direction` 资源交叉校验；梅花金标准补充同人（水/北）、否（火/南）、坤体（木/东、东南）三组断言。
- **未改变：** 整体 `status` 仍为 `PENDING`、`formal_allowed=false`，直至 `W-01` 等规则完成校核。

## 验证

- `cargo test -p yitool-domain -p yitool-meihua -p yitool-rules`
- `cargo run -p yitool-cli -- verify --resources resources`
- 金标准：`C=1,A=3` → 体离火 → 忌神水 → 坎/北；`U0=9,D0=16` → 体乾金 → 忌神火 → 离/南；`C=8,A=8` → 体坤土 → 忌神木 → 震/东、巽/东南。
