# 易学学习与推演记录工具（个人及家庭自用）

本地离线、学习娱乐型的易学推演记录工具。以传统易学经典为学习依据，提供可视化、步骤化、可溯源的推演过程展示、黄历文化查询、择日文化参考、生活提醒与情绪治愈内容。

> **合规声明：** 本系统不对外发布、不提供公网服务、不商用、不提供付费测算，不替代医疗、法律、投资、安全、婚姻等专业判断。所有输出均为传统易学文化学习与娱乐参考，不构成生活决策唯一依据。

---

## 当前状态

| 项目 | 状态 |
|---|---|
| Spec Kit | ✅ 已初始化（`specify-cli 1.2.5`，DSH 集成） |
| 规范文档 | ✅ SRS v0.4、ARS v0.2、IR v0.1、CAL v0.1、TDS v0.1 草案已齐 |
| 规范冻结 | ⏳ 未冻结；规则/数据未校核项以 `PENDING` 显式标注 |
| 历法数据 | ✅ `CAL-DATA-v0.2` 经 HKO 1901–2100 逐日校核（6 条午夜边界容差） |
| 代码 | ✅ Rust workspace + `yitool` CLI（9 个 crate，阶段 0 内核） |
| 自动测试 | ✅ 单元/金标准/CAL 逐日基准/隐私扫描/分布/性能（`cargo test --workspace`） |
| 当前阶段 | **阶段 0 — 可信基础与可复现内核（进行中，P1 内核已验收）** |
| 当前规格 | [`specs/001-phase0-foundation/spec.md`](specs/001-phase0-foundation/spec.md) |

---

## 快速开始

```bash
# 1. 检查 Spec Kit 环境
specify check

# 2. 构建并运行阶段 0 验收入口
source ~/.cargo/env
cargo build --workspace
cargo run -p yitool-cli -- cal --datetime "2026-11-03T12:00:00+08:00"
cargo run -p yitool-cli -- meihua --color 1 --aux 3 --datetime "2026-11-03T12:00:00+08:00"
cargo run -p yitool-cli -- verify --resources resources

# 3. 全量质量门（fmt + clippy + test + verify + privacy-scan）
./scripts/check.sh

# CI: push/PR 由 .github/workflows/check.yml 运行同一脚本

# 4. 重新生成资源（可选）
python3 scripts/gen_resources.py
.venv-cal/bin/python scripts/gen_cal_precise.py --status VERIFIED
cargo run -p gen-cal -- baseline resources/cal
python3 scripts/gen_manifest.py
```

CAL 交叉校核报告见 [`docs/reports/CAL-HKO-CROSSCHECK-v0.2.md`](docs/reports/CAL-HKO-CROSSCHECK-v0.2.md)。

阶段 0 CLI 命令：`cal`、`seed`、`meihua`、`verify`、`privacy-scan`、`db init|check`。
详细验收见 [`specs/001-phase0-foundation/quickstart.md`](specs/001-phase0-foundation/quickstart.md)。

若在其他机器克隆后没有 `.dsh/skills/`，执行：

```bash
specify init . --here --force --non-interactive --integration dsh
```

---

## 文档导航

| 文档 | 内容 |
|---|---|
| [项目章程](.specify/memory/constitution.md) | 最高治理文件：9 条核心原则、约束、质量门 |
| [项目路线图](docs/00-PROJECT-ROADMAP.md) | 现状、阶段 0–5、里程碑、关键路径、风险、下一步 |
| [文档分析](docs/01-DOC-ANALYSIS.md) | 五文档逐项分析、22 项一致性问题（A-01～A-22）、覆盖矩阵 |
| [决策登记册](docs/02-DECISION-REGISTER.md) | D-01～D-41 决策、建议、截止阶段 |
| [统一数据字典](docs/04-DATA-DICTIONARY.md) | 合并五文档表清单、版本基线、资源目录映射 |
| [阶段 0 规格](specs/001-phase0-foundation/spec.md) | 6 个用户故事、23 条功能需求、10 条成功标准 |
| [开发指引](AGENTS.md) | 代理/开发者运行时约定、目录、流程 |

### 五份规范（`docs/`）

1. **SRS v0.4** — 软件需求规格：做什么、边界、功能、验收、阶段。
2. **ARS v0.2** — 易学算法与规则：梅花/焦氏/纳甲/择吉公式、映射、金标准。
3. **IR v0.1** — 推演解释规则：措辞等级、模板、冲突解释、合规扫描。
4. **CAL v0.1** — 历法服务：公农历、节气、干支、日界、时区、旬空。
5. **TDS v0.1** — 技术设计：架构、接口、SQLite、隐私、部署、测试。

依赖关系：

```text
SRS ── ARS ── 易学计算结果
 │     IR  ── 文化解释结果
 │     CAL ── 历法基础结果
 └──── TDS ── 软件实现
```

---

## 核心底线（摘自章程）

1. **文档分层**：SRS/ARS/IR/CAL/TDS 各司其职，冲突时以 ARS/IR/CAL 为准。
2. **规则溯源**：每条规则有 `rule_id`、层级（L1/L2/L3）、来源、版本、测试。
3. **可复现**：固定模式同输入同版本必得同结果；摇卦凭快照复现。
4. **隐私**：生辰仅内存、不落盘；问事默认不持久化，HMAC 关联。
5. **不合成总分**：三引擎平行展示，不输出“综合吉凶分数”。
6. **合规措辞**：禁止绝对断语，所有输出附免责声明。
7. **历史只读**：版本升级不静默重算历史。
8. **金标准先行**：未冻结规则不发布正式结果。
9. **离线轻量**：核心功能不依赖网络，不采集/上传数据。

---

## 下一步

1. 关闭阶段 0 剩余决策（D-05/D-06/D-07/D-08/D-10/D-14 已关闭；其余按建议值落地并写入 ADR）。
2. 冻结 CAL 规则集；`CAL-DATA-v0.2` 已完成 HKO 1901–2100 日期级交叉校核。
3. 校准 `HEX-01`、`NJ-01`、`W-01` 与九色象意文案；未冻结前保持 `PENDING`。
4. 阶段 1：接入 Tauri 2 + React UI，复用现有 `app` 服务层；实现九色选择器与单一颜色资源校验（T075）。
5. 补充 Q/B 扰动材料定义、HMAC OS Keychain 真实实现与密钥轮换流程。

---

## 许可与使用性质

个人及家庭成员自用；不对外发布、不商用、不提供付费测算服务。古籍文本与第三方数据须在引入前确认版权与版本来源。
