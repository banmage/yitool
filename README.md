# 易学学习与推演记录工具（个人及家庭自用）

本地离线、学习娱乐型的易学推演记录工具。以传统易学经典为学习依据，提供可视化、步骤化、可溯源的推演过程展示、黄历文化查询、择日文化参考、生活提醒与情绪治愈内容。

> **合规声明：** 本系统不对外发布、不提供公网服务、不商用、不提供付费测算，不替代医疗、法律、投资、安全、婚姻等专业判断。所有输出均为传统易学文化学习与娱乐参考，不构成生活决策唯一依据。

---

## 当前状态

| 项目 | 状态 |
|---|---|
| Spec Kit | ✅ 已初始化（`specify-cli 1.2.5`，DSH 集成） |
| 规范文档 | ✅ SRS v0.5、ARS v0.2、IR v0.1、TDS v0.2 草案已齐；CAL v0.2 已冻结 |
| 规范冻结 | ⏳ 未整体冻结；**CAL v0.2 / CAL-RULE-v0.2 已冻结**（ADR-013～016），**W-01-v0.2、HEX-01-v0.2、NJ-01-v0.2、MEIHUA-CORE-v0.2 已校核**（ADR-017～019、021），**REPORT-v1 / EVIDENCE-v1 已批准**（ADR-020），其余规则/数据未校核项以 `PENDING` 显式标注 |
| 历法数据 | ✅ `CAL-DATA-v0.2` 经 HKO 1901–2100 逐日校核（6 条午夜边界容差） |
| 代码 | ✅ Rust workspace + `yitool` CLI；🛠️ Tauri 2 + React/TS 阶段 1 桌面闭环 |
| 自动测试 | ✅ 阶段 0 单元/金标准/CAL/隐私/分布/报告/链接；阶段 1 前端与桌面命令测试见 `apps/desktop` |
| 当前阶段 | **阶段 1 — 梅花单引擎桌面闭环（已实现并通过 GUI 冒烟）；阶段 0 未整体关闭，JL-01 仍为 PENDING** |
| 当前规格 | [`specs/003-phase1-complete/spec.md`](specs/003-phase1-complete/spec.md) |

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

阶段 0 CLI 命令：`cal`、`seed`、`meihua`、`report`、`evidence`、`verify`、`privacy-scan`、`db init|check`。
详细验收见 [`specs/001-phase0-foundation/quickstart.md`](specs/001-phase0-foundation/quickstart.md)。

## 阶段 1 桌面应用

前置：Rust stable、Node.js 22+、npm，以及平台所需的 Tauri/WebView 构建依赖。

```bash
cd apps/desktop
npm ci
npm test
npm run build
npm run tauri dev

# 生成 Ubuntu/Debian amd64 部署测试包
npm run tauri -- build --bundles deb

# Linux 上交叉生成 Windows x64 NSIS 安装包（需 LLVM/LLD、Clang、NSIS）
rustup target add x86_64-pc-windows-msvc
cargo install --locked cargo-xwin
npm run tauri -- build --target x86_64-pc-windows-msvc --bundles nsis --runner cargo-xwin
```

应用从已校验的本地资源载入九色/生肖选项与梅花规则。Linux 的 WebKitGTK/GTK 构建依赖见 [Tauri v2 prerequisites](https://v2.tauri.app/start/prerequisites/)。UI 仅接入梅花单引擎，支持九色与传统数字输入、手动/随机抽卡、种子链展示、本地 SQLite 历史、单库多档案与档案级设置；未冻结项目保持“待校核”。历史只保存非敏感推演记录、步骤、报告与版本，不保存生辰、问事原文或 HMAC 密钥，全程离线。
部署包输出到 `apps/desktop/src-tauri/target/release/bundle/deb/`。该包仅用于部署验证；阶段 1 与 JL-01 尚未关闭，不代表正式发行授权。
Windows NSIS 安装包输出到 `apps/desktop/src-tauri/target/x86_64-pc-windows-msvc/release/bundle/nsis/`。Linux 交叉构建属实验性且当前未签名；Windows 安装与运行仍需在 Windows 环境验证。MSI 安装包只能在 Windows 上构建。

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
| [决策登记册](docs/02-DECISION-REGISTER.md) | D-01～D-48 决策、建议、截止阶段 |
| [统一数据字典](docs/04-DATA-DICTIONARY.md) | 合并五文档表清单、版本基线、资源目录映射 |
| [阶段 0 规格](specs/001-phase0-foundation/spec.md) | 6 个用户故事、23 条功能需求、10 条成功标准 |
| [阶段 1 规格](specs/002-phase1-meihua-desktop/spec.md) | 九色梅花桌面闭环、报告导出与依据只读阅读器 |
| [阶段 1 补全规格](specs/003-phase1-complete/spec.md) | 传统数字、随机抽卡、种子日志、历史、档案与设置 |
| [开发指引](AGENTS.md) | 代理/开发者运行时约定、目录、流程 |

### 五份规范（`docs/`）

1. **SRS v0.5** — 软件需求规格：做什么、边界、功能、验收、阶段、完整过程报告与依据链接。
2. **ARS v0.2** — 易学算法与规则：梅花/焦氏/纳甲/择吉公式、映射、金标准。
3. **IR v0.1** — 推演解释规则：措辞等级、模板、冲突解释、合规扫描。
4. **CAL v0.2** — 历法服务：公农历、节气、干支、日界、时区、旬空；规则集已冻结（`CAL-RULE-v0.2`）。
5. **TDS v0.2** — 技术设计：架构、接口、SQLite、隐私、部署、报告渲染与依据链接。

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

1. 关闭阶段 0 剩余决策：CAL 规则集已冻结（`CAL v0.2` / `CAL-RULE-v0.2`，ADR-016）；W-01 旺衰表、HEX-01 八宫表、NJ-01 纳甲表、梅花核心规则已校核（ADR-017～019、021）；剩余 D-09（变卦六亲，阶段 3 前）。
2. 校勘规则数据：`W-01`、`HEX-01`、`NJ-01`、`MEIHUA-CORE-v0.2` 已完成；`JL-01` 分批入库，未完成前保持 `PENDING`。
3. 阶段 1：Tauri 2 + React UI 已复用现有 `app` 服务层完成九色/传统数字、抽卡、历史、档案与设置；剩余 D-26 安装包签名决策。
4. 补充 Q/B 扰动材料定义、HMAC OS Keychain 真实实现与密钥轮换流程。

---

## 许可与使用性质

个人及家庭成员自用；不对外发布、不商用、不提供付费测算服务。古籍文本与第三方数据须在引入前确认版权与版本来源。
