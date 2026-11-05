# 统一数据字典（Data Dictionary）

**版本：** DD-v0.2
**更新日期：** 2026-11-04
**目的：** 合并 SRS 附录 A、ARS 附录 D、CAL 附录 B 与 IR/TDS 中的表清单，形成**唯一索引**，作为资源加载、manifest、金标准测试与文档修订的依据。

**冲突处理：** 本文档以并集为准；同一 ID 在不同文档中状态/版本不一致时，以“待冻结”优先，并在 `docs/01-DOC-ANALYSIS.md` 的 A-05/A-13/A-14 跟踪。

---

## 1. 标识规范

| 类型 | 格式 | 示例 | 说明 |
|---|---|---|---|
| 步骤 ID | `<模块>-<动作>-<NNN>` | `MEIHUA-QIGUA-001` | 一步骤可关联多条规则 |
| 规则 ID | 同步骤 ID 或 `<模块>-<类别>-<NNN>` | `ZEJI-JIANCHU-001` | 必须唯一，不与其他类型混用 |
| 数据表 ID | 大写 + 连字符 | `HEX-01`、`COLOR-NUM-02` | 只表示数据/映射表，不表示规则 |
| 典籍出处 ID | `SRC-<模块>-<NNN>` | `SRC-MEIHUA-001` | 关联书名、版本、篇目、原文 |
| 解释规则 ID | `IR-<引擎>-<类别>-<NNN>` | `IR-MEIHUA-TIYONG-001` | IR 规则 |
| 解释模板 ID | `TPL-<引擎>-<类别>-<NNN>` | `TPL-MEIHUA-TIYONG-003` | 模板集 |
| 资源清单 | `MANIFEST-<rules|data|texts>` | `MANIFEST-data` | 带 checksum |
| 版本 | `<域>-v<主>.<次>` | `RULE-v0.2` | 冻结后不原地修改 |

**禁止：** 把 `data_ref` 当 `rule_id`；把 `step_id` 与 `rule_id` 混用；用裸文件名代替表 ID。

---

## 2. 版本基线（D-05/ADR-003 已统一）

```text
SRS_VERSION        = SRS-v0.4        # 待升 v0.5 同步文档状态
ARS_VERSION        = ARS-v0.2
IR_VERSION         = IR-v0.1
CAL_VERSION        = CAL-v0.1
TDS_VERSION        = TDS-v0.1
RULE_SET_VERSION   = RULE-v0.2       # 统一为 ARS 基线
DATA_VERSION       = DATA-v0.2       # 统一为 ARS 基线
CAL_RULE_SET       = CAL-RULE-v0.1
CAL_DATA_VERSION   = CAL-DATA-v0.2
IR_RULE_SET        = IR-RULE-v0.1
IR_TEMPLATE_SET    = IR-TPL-v0.1
SEED_ALGO          = SEED-v2
RNG_VERSION        = RNG-v1
APP_VERSION        = APP-v0.1
DB_SCHEMA_VERSION  = DB-v0.2
CONFIG_SCHEMA      = CFG-v0.1
LOG_SCHEMA         = LOG-v0.1
```

> 说明：SRS 附录 B、ARS 0.3 与代码已统一为 `RULE-v0.2`/`DATA-v0.2`；SRS 附录 B 已同步完整 `VersionSet`。
> **版本记录：** CAL 文档已同步 `CAL-DATA-v0.2`；`DB-v0.2` 由 ADR-007 引入 `hmac_key_id` 强制约束；`G-01-v0.2` 由 ADR-011 增加后天方位字段；CAL 日界/日柱/范围/时区由 ADR-013 冻结，CAL 数据源/基准/边界容差由 ADR-014 冻结，tzdata 版本由 ADR-015 冻结；当前校核数据以本表为准。

---

## 3. 状态词汇

| 状态 | 含义 | 界面/日志 |
|---|---|---|
| `FROZEN` | 已冻结，可用于正式结果 | 正常显示版本 |
| `VERIFIED` | 已校核，但规则集可能未整体冻结 | 显示版本 + 已校核 |
| `PENDING` | 待校核/待校勘 | 显示“待校核/待校勘”，不得作正式结论 |
| `DEPRECATED` | 已废弃 | 不加载；历史按原版本渲染 |
| `MISSING` | 条目缺失 | 显示“数据不可用/待补”，不编造 |

---

## 4. 数据表总表

### 4.1 输入与象数

| ID | 名称 | 归属 | 用途 | 状态 | 冻结阶段 | 来源 |
|---|---|---|---|---|---|---|
| `COLOR-NUM-02` | 九色象数表（唯一标准） | ARS/SRS | 唯一颜色输入标准（L3） | `VERIFIED` | 已冻结（D-01/ADR-002） | ARS 1.1 |
| `Z-01` | 十二生肖表 | ARS/SRS | 生肖象数/地支五行 | `VERIFIED` | 阶段 1 | ARS 1.3 |
| `LUNAR-MONTH-INDEX` | 农历月序表 | CAL/ARS | 梅花月序起象 | `PENDING` | 阶段 0（CF-01） | CAL 4.1、ARS 1.4.1 |
| `JIEQI-MONTH-BRANCH` | 节气月建表 | CAL/ARS | 旺衰、纳甲、择吉、黄历 | `PENDING` | 阶段 0（CF-02） | CAL 4.2、ARS 1.4.2 |
| `W-01` | 四时旺衰表 | ARS/SRS | 五行逐月旺相休囚死 | `PENDING` | 阶段 0（F-02） | ARS 3.7 |
| `G-01` | 先天八卦表（`G-01-v0.2`） | ARS/SRS | 数字→八卦、五行、后天方位 | `VERIFIED` | 阶段 0 | ARS 1.5、3.8 |
| `L-01` | 洛书九宫方位表 | SRS | 方位象意展示 | `VERIFIED` | 阶段 4 | SRS 附录 A、IR 7.2 |

### 4.2 卦象与纳甲

| ID | 名称 | 归属 | 用途 | 状态 | 冻结阶段 | 来源 |
|---|---|---|---|---|---|---|
| `HEX-01` | 六十四卦八宫表 | ARS/SRS | 卦名、上下卦、宫、世应、类型 | `PENDING` | 阶段 0/3（F-03） | ARS 1.6、附录 A |
| `NJ-01` | 纳甲干支表 | ARS/SRS | 八卦六爻干支 | `PENDING` | 阶段 3（F-04） | ARS 1.7、附录 B |
| `XK-01` | 旬空表 | ARS | 六旬空亡 | `VERIFIED` | 阶段 0 | ARS 1.8 |
| `JL-01` | 焦氏易林辞表 | ARS/SRS | 64×64 索引、原文、校勘 | `PENDING` | 阶段 2（F-05） | ARS 1.10、4.3 |
| `LIUYAO-YONGSHEN-01` | 纳甲用神映射 | ARS/SRS | 事项类型→六亲用神 | `PENDING` | 阶段 3（F-06） | ARS 5.7 |
| `MEIHUA-INTERPRET-01` | 梅花解释映射 | SRS | 梅花→IR 的映射 | `PENDING` | 阶段 1/4（IF-01） | SRS 附录 A |

### 4.3 择吉与黄历

| ID | 名称 | 归属 | 用途 | 状态 | 冻结阶段 | 来源 |
|---|---|---|---|---|---|---|
| `ZEJI-EVENT-01` | 择吉事项规则 | ARS/SRS | 事项→规则集 | `PENDING` | 阶段 4（F-07） | ARS 6.1 |
| `ZEJI-RULE-01` | 择吉统一规则表 | ARS | 建除/神煞/过滤/权重统一抽象 | `PENDING` | 阶段 4（F-07） | ARS 1.11 |
| `W-02` | 择吉权重表 | SRS | 权重（与 ZEJI-RULE-01 关系待定） | `PENDING` | 阶段 4（F-07） | SRS 3.4 |
| `HL-01` | 黄历宜忌表 | ARS/SRS | 建除、星宿、宜忌、出处 | `PENDING` | 阶段 4（F-10） | ARS 6.5 |
| `JC-01` | 建除十二神表 | ARS | 建除顺序 | `VERIFIED` | 阶段 4 | ARS 1.9 |

> **待澄清（A-13）：** `W-02` 与 `ZEJI-RULE-01.weight` 是同一张表、视图，还是“事项权重层 + 规则层”两层？建议：`ZEJI-RULE-01` 存规则与基础权重，`W-02` 存事项级覆盖权重，二者通过 `event_type` 关联。

### 4.4 解释、溯源与文案

| ID | 名称 | 归属 | 用途 | 状态 | 冻结阶段 | 来源 |
|---|---|---|---|---|---|---|
| `IR-01` | 推演解释规则表 | SRS/IR | IR 规则与模板 | `PENDING` | 阶段 0/1（IF-01/IF-07） | SRS 附录 A、IR 0.5 |
| `SRC-01` | 典籍溯源表 | ARS/SRS | rule_id/data_ref/source_ref 关联 | `VERIFIED` | 阶段 0 | ARS 8.2 |
| `COPY-01` | 治愈文案与生活提醒池 | SRS | 幸运数、情绪治愈 | `VERIFIED` | 阶段 5（IF-09） | SRS 3.8 |

> **待澄清（A-14）：** `IR-01` 是“数据表 ID”，而 IR 文档使用 `IR-RULE-v0.1`/`IR-TPL-v0.1` 规则集/模板集。建议：`IR-01` 作为 manifest 中的逻辑表 ID，指向 `resources/ir/rules/` 与 `resources/ir/templates/` 两个版本化目录。

### 4.5 历法与基准

| ID | 名称 | 归属 | 用途 | 状态 | 冻结阶段 | 来源 |
|---|---|---|---|---|---|---|
| `CAL-LUNAR-TABLE` | 公农历转换表 | CAL | 预计算月表 | `VERIFIED` | 阶段 0（CF-01） | CAL 2.2；HKO 1901–2100 逐日校核 |
| `CAL-JIEQI-TABLE` | 节气时刻表 | CAL | 预计算节气表 | `VERIFIED` | 阶段 0（CF-02） | CAL 3.2；HKO 日期级校核 + 午夜边界容差 |
| `CAL-BASE-2025` | 2025 逐日基准（含闰六月） | CAL | 闰月/基准验收 | `PENDING` | 阶段 0（CF-08） | CAL 8.1 + A-07；由 CAL-DATA-v0.2 派生的回归快照，非独立权威基准 |
| `CAL-BASE-2026` | 2026 逐日基准 | CAL | 基准验收 | `PENDING` | 阶段 0（CF-08） | CAL 8.1；由 CAL-DATA-v0.2 派生的回归快照，非独立权威基准 |
| `CAL-TZDATA` | IANA tzdata | CAL | 时区/夏令时 | `VERIFIED` | 阶段 0（CF-09） | CAL 6.5、ADR-015；chrono-tz 0.10.4 内置 tzdata 2025b |

> **A-07：** 2026 年无闰月；基准必须扩展 2025（闰六月）或 2028（闰五月），否则“闰月整月覆盖”无法满足。

### 4.6 Schema、日志与资源

| ID | 名称 | 归属 | 用途 | 状态 | 冻结阶段 | 来源 |
|---|---|---|---|---|---|---|
| `LOG-SCHEMA-001` | 推演日志结构 | ARS/TDS | 步骤日志 JSON Schema | `VERIFIED` | 阶段 0 | ARS 8.1、SRS 附录 B |
| `DB-SCHEMA-v0.1` | 本地数据库 schema | TDS | profiles/runs/steps/snapshots 等 | `PENDING` | 阶段 0（TF-08） | TDS 4.2 |
| `MANIFEST-rules` | 规则资源清单 | TDS | 版本/checksum/status/source | `MERGED` | 阶段 0（TF-04） | TDS 4.3；阶段 0 合并入 `MANIFEST-data`（ADR-012） |
| `MANIFEST-data` | 数据资源清单 | TDS | 同上 | `VERIFIED` | 阶段 0（TF-04，ADR-012） | TDS 4.3 |
| `MANIFEST-texts` | 古籍文本清单 | TDS | 同上 | `PENDING` | 阶段 5（TF-11） | TDS 4.3 |

---

## 5. 资源目录映射（TDS 4.3，阶段 0 实际落地）

```text
resources/
├── ir/
│   └── compliance/
│       └── forbidden_phrases.json   # IR-COMPLIANCE-01
├── data/
│   ├── hex64.json              # HEX-01
│   ├── najia.json              # NJ-01（阶段 0 仅占位，未用于装卦）
│   ├── color_num.json          # COLOR-NUM-02（九色唯一标准）
│   ├── zodiac.json             # Z-01
│   ├── bagua.json              # G-01
│   ├── xun.json                # XK-01
│   ├── wuxing_wangshuai.json   # W-01
│   └── rules.json              # RULE-SET-01（规则 + 出处注册表）
├── cal/
│   ├── lunar_months.json       # CAL-LUNAR-DATA
│   ├── jieqi.json              # CAL-JIEQI-DATA
│   └── baseline/
│       ├── 2025.json           # CAL-BASELINE-2025
│       └── 2026.json           # CAL-BASELINE-2026
└── manifests/
    └── data.manifest.json      # MANIFEST-data
```

阶段 1–5 再补充 `rules/`、`texts/`、`jiaolin_4096.json`、`zeji_*`、`huangli.json` 等；未实现目录不得提前声明为已冻结。

每个资源文件必须带：

```json
{
  "id": "HEX-01",
  "version": "DATA-v0.2",
  "status": "PENDING",
  "source": "ARS v0.2 附录 A + 《卜筮正宗》交叉校核",
  "checksum": "sha256:...",
  "updated_at": "2026-11-03"
}
```

---

## 6. 关系与约束

1. `rule_id` → `SRC-01`：每条规则必须有至少一条来源记录；L3 的 `book` 必须为“本系统自定义”。
2. `rule_id` → `data_refs[]`：每条规则声明依赖的数据表；数据表缺失时规则不得运行。
3. `data_ref` → `MANIFEST-data`：每张表必须在清单中登记版本、checksum、状态。
4. `IR-01` → `MEIHUA-INTERPRET-01` / `LIUYAO-YONGSHEN-01` / `ZEJI-*`：IR 只消费结构化结果，不直接查数据表。
5. `HEX-01` → `NJ-01` / `JL-01`：纳甲与焦氏都以 64 卦 ID 为键；卦 ID 必须稳定。
6. 颜色输入只有 `COLOR-NUM-02` 一张表；映射变更须升版，历史记录不重算。
7. `W-02` 与 `ZEJI-RULE-01`：待 A-13 澄清后确定主外键。

---

## 7. 阶段 0 必须就绪的最小集合

| 优先级 | 表/资源 | 原因 |
|---:|---|---|
| 1 | `G-01`、`COLOR-NUM-02`、`Z-01` | 梅花起卦 |
| 2 | `HEX-01` | 卦象识别、互变 |
| 3 | `LUNAR-MONTH-INDEX`、`JIEQI-MONTH-BRANCH` | 月令/旺衰 |
| 4 | `W-01` | 旺衰 |
| 5 | `XK-01` | 旬空 |
| 6 | `CAL-BASE-2025/2026`、`CAL-TZDATA` | 历法验收 |
| 7 | `MANIFEST-*`、`LOG-SCHEMA-001`、`DB-SCHEMA-v0.1` | 资源/日志/存储 |
| 8 | `SRC-01` | 溯源 |
| 9 | `IR-01`（仅禁用词/免责声明子集） | 合规扫描 |

**不在阶段 0 就绪：** `NJ-01`、`JL-01`、`LIUYAO-YONGSHEN-01`、`ZEJI-*`、`HL-01`、`COPY-01`、`L-01`。

---

## 8. 维护规则

1. 新增/修改数据表必须同时更新本文件、`MANIFEST-*` 与相关规范。
2. 表 ID 一旦发布不得复用；废弃用 `DEPRECATED`。
3. 状态从 `PENDING` 升 `VERIFIED/FROZEN` 必须附校核来源与测试样例。
4. 版本升级不得修改历史记录；历史按原版本资源渲染。
