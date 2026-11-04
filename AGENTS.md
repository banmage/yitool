# AGENTS.md

本文件是 AI 编码代理与开发者在仓库内的运行时指引。项目治理的最高文件是 `.specify/memory/constitution.md`；本文件冲突时以章程为准。

## 1. 项目是什么

易学学习与推演记录工具（个人及家庭自用、本地离线、学习娱乐）。核心能力：梅花易数、焦氏易林、纳甲六爻、黄历/择吉、推演解释（IR）、历史复盘。

**合规底线：** 不对外发布、不商用、不提供付费测算；所有输出必须附免责声明；禁止绝对断语；三引擎不合成吉凶总分。

## 2. 文档权威顺序

1. `.specify/memory/constitution.md` — 项目章程（最高）
2. `docs/易学推演辅助决策系统 SRS v0.4.txt` — 行为、边界、验收
3. `docs/易学算法与规则规范 ARS v0.2.txt` — 算法、映射、金标准
4. `docs/推演解释规则规范 IR v0.1.txt` — 解释模板、措辞、合规
5. `docs/历法服务规范 CAL v0.1 .txt` — 历法、日界、时区
6. `docs/技术设计说明书  TDS v0.1.txt` — 架构、接口、存储
7. `docs/00-PROJECT-ROADMAP.md` — 阶段、里程碑、风险
8. `docs/01-DOC-ANALYSIS.md` — 文档冲突与缺口（A-01～A-22）
9. `docs/02-DECISION-REGISTER.md` — 决策登记（D-01～D-40）
10. `docs/04-DATA-DICTIONARY.md` — 统一数据字典与版本基线
11. `specs/001-phase0-foundation/spec.md` — 当前特性规格

遇到规则/算法争议，以 ARS/IR/CAL 为准；遇到行为/验收争议，以 SRS 为准；任何实现与它们冲突时，回退修订实现，不得反向改规范。

## 3. 不可违反的规则

- 生辰：仅内存、不序列化、不落盘、不写日志、不入快照、不参与长期复现种子。
- 问事文本：默认不持久化；关联历史用本地密钥 HMAC；原文需显式确认并加密。
- 三引擎：独立输入、平行展示、不合成总分、不串引擎。
- 规则层级：L1/L2/L3/PROVISIONAL 必须标注；L3 不得冒充 L1；缺失显示“待校勘/待冻结”。
- 版本：每次推演记录全部版本；历史按原版本渲染，不静默重算。
- 合规：禁用绝对断语；所有结果页/导出/历史头部有免责声明。
- 未冻结规则：不得发布正式结果；界面与日志标“待校核”。

## 4. 工作流程（Spec Kit）

本项目已初始化 Spec Kit（`specify-cli 1.2.5`，集成 `dsh`）。技能位于 `.dsh/skills/`，通过 `/speckit-*` 调用。

```text
/speckit-constitution   更新章程
/speckit-specify        创建特性规格
/speckit-clarify        澄清歧义（按需）
/speckit-plan           生成实现计划
/speckit-tasks          生成任务
/speckit-analyze        跨文档一致性检查（按需）
/speckit-implement      执行实现
/speckit-checklist      生成质量检查清单（按需）
```

规则：

- 一次只创建一个特性规格，目录 `specs/NNN-<short-name>/`。
- 规格未通过 `checklists/requirements.md`，不得进入 plan。
- 先有金标准/失败测试，再有实现；禁止用实现输出反填金标准。
- 阶段门未通过不得进入下一阶段（SRS 9.1）。

## 5. 目录约定

```text
docs/                 五份规范 + 项目管理文档 + ADR
specs/                Spec Kit 特性规格
.specify/             章程、模板、脚本
.dsh/skills/          Spec Kit 技能（本地生成，不入库）
Cargo.toml            Rust workspace
rust-toolchain.toml   stable
crates/
  domain/             ID、版本、干支、五行、卦象、步骤日志、生辰内存对象
  cal/                公农历、节气、四柱、日界、时区、旬空
  seed/               种子链、HMAC、无偏随机
  rules/              manifest、checksum、规则/出处注册表
  meihua/             九色/传统数字、互卦、变卦、体用、旺衰
  ir/                 免责声明与禁用词扫描
  store/              SQLite schema、档案/推演/步骤/快照
  app/                编排、隐私扫描、快照导出、日志白名单
  cli/                `yitool` 验收入口
tools/gen-cal/        逐日回归基准生成器（读取已校核的 CAL 预计算表）
resources/            只读规则/数据/历法/合规词表（带 manifest/checksum）
scripts/              资源生成、manifest、质量门
tests/golden/         跨 crate 金标准 JSON
```

## 6. 当前状态与下一步

- 五份规范：草案已齐，均未冻结；未校核资源显式标 `PENDING`。
- 代码：Rust workspace + `yitool` CLI 已落地；阶段 0 P1 内核（CAL/种子/溯源/隐私/梅花金标准）已通过测试。
- 历法数据：`CAL-DATA-v0.2` 已按香港天文台 1901–2100 对照表逐日校核（农历 0 差异；节气 6 条午夜边界容差，见 `docs/reports/CAL-HKO-CROSSCHECK-v0.2.md`）。
- 当前阶段：**阶段 0（进行中）**。
- 已批准：D-01 九色、D-02 技术栈、D-05 版本基线、D-06 A/B 随机语义、D-07 Q/B 扰动、D-08 HMAC 密钥、D-10 数据字典、D-14 DST 策略、D-19 CAL 算法/库、D-41 午夜边界容差；D-03/D-04/D-12/D-13/D-20 已按建议值实现，待正式写入 ADR。
- 阻塞点：HEX-01/NJ-01/W-01 校勘、忌神算法、CAL 规则冻结。
- 下一步：关闭剩余阶段 0 决策 → 校勘规则数据 → 阶段 1 Tauri/React 接入。

## 7. 代码与提交约定

- 提交前缀：`docs:`、`spec:`、`feat:`、`fix:`、`test:`、`chore:`。
- 一个提交只做一件事；规则/数据变更必须同步版本与测试。
- 禁止硬编码规则表、权重表、解释模板；从 `resources/` 加载并校验 checksum（`hex64` 名称表在 `domain` 中保留镜像，并由资源测试交叉校验）。
- 禁止在日志/错误信息/调试输出中打印生辰、问事原文、HMAC 密钥。
- 新增规则必须补 `rule_id`、层级、来源、版本、测试样例。
- 未冻结规则输出必须带 `PENDING`/“待校核”，不得作为正式结论。

## 8. 常用命令

```bash
source ~/.cargo/env                # 本机 Rust 安装路径
cargo build --workspace            # 构建
cargo test --workspace             # 单元 + 金标准 + 基准 + 隐私 + 分布 + 性能
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
./scripts/check.sh                 # 全量质量门

cargo run -p yitool-cli -- cal --datetime "2026-11-03T12:00:00+08:00"
cargo run -p yitool-cli -- meihua --color 1 --aux 3 --datetime "2026-11-03T12:00:00+08:00"
cargo run -p yitool-cli -- meihua --traditional 9 16 --datetime "2026-11-03T12:00:00+08:00"
cargo run -p yitool-cli -- seed --profile family-a --run-time "2026-11-03T12:00:00+08:00" --color 1 --aux 3
cargo run -p yitool-cli -- verify --resources resources
cargo run -p yitool-cli -- db init --path /tmp/yitool.db

python3 scripts/gen_resources.py   # 从 ARS 生成 JSON 资源
.venv-cal/bin/python scripts/gen_cal_precise.py            # sxtwl 生成 CAL 表（PENDING）
.venv-cal/bin/python scripts/gen_cal_precise.py --status VERIFIED  # 校核通过后
cargo run -p gen-cal -- baseline resources/cal             # 从 CAL 表派生逐日基准
python3 scripts/crosscheck_cal_hko.py                      # HKO 1901–2100 交叉校核
python3 scripts/gen_manifest.py    # 重算 checksum

specify check                      # 检查工具与集成
specify version                    # 版本信息
specify integration list           # 已装集成
cat .specify/feature.json          # 当前特性目录
```
