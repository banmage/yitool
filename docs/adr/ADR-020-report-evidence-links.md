# ADR-020: 完整推演报告与依据/附件链接契约

- 状态：已批准
- 日期：2026-11-05
- 关联：SRS v0.5 FR-13/FR-14、TDS v0.2 `TDS-API-REPORT-001` / `TDS-API-EVIDENCE-001`、D-47
- 决策人：项目所有者
- 生效版本：`REPORT-v1`、`EVIDENCE-v1`、`SRS-v0.5`、`TDS-v0.2`

## 背景

阶段 0 已有结构化 `StepLog`、规则注册表、典籍出处与 manifest，但缺少统一的“完整过程”输出：
结果页若分别实现窗口、复制、下载，容易出现步骤截断、模板不一致或断链；依据与附件也没有稳定链接，
用户无法从步骤直接核对规则、原文与数据资源。SRS v0.5 新增 FR-13/FR-14，需要先冻结渲染与链接契约，
再进入阶段 1 UI。

## 选项

| 选项 | 优点 | 缺点 | 影响 |
|---|---|---|---|
| 每个入口各自拼接文本 | 实现快 | 三处内容漂移、断链、重复维护 | 不可接受 |
| 只导出 JSON 快照 | 结构完整 | 不适合阅读/复制/分享 | 不满足 FR-13 |
| HTML 报告 | 排版好 | 不易复制、依赖渲染器、体积大 | 不作为唯一格式 |
| **统一 Markdown + 稳定链接索引** | 可读、可复制、可保存、可 diff、离线 | 需维护链接契约 | 采用 |

## 决策

1. 以 `REPORT-v1` 作为完整过程报告的唯一规范化格式：`ReportDocument.markdown` 同时用于窗口展示、复制到剪贴板、下载 `.md`。
2. 报告只消费已保存/已生成的 `StepLog`、`VersionSet` 与 `EvidenceIndex`，不重新计算卦象；历史记录按原版本渲染。
3. 以 `EVIDENCE-v1` 作为依据/附件链接契约：
   - 规则：`yitool://rule/<rule_id>`
   - 出处：`yitool://source/<source_ref>`
   - 附件/数据资源：`yitool://resource/<manifest_id>`
   - Markdown 同时输出 `#evidence-<kind>-<id>` 锚点，保证离线阅读器内可跳转。
4. 渲染前执行 `assert_step_coverage`：任一 `rule_id`、`source_ref`、`data_ref` 无法解析即拒绝生成“完整”报告；不输出断链占位。
5. 链接解析只读、白名单、防路径穿越；禁止绝对路径、`..`、任意本地文件与外部 URL。缺失/checksum 失败/版本不兼容返回结构化错误，UI 显示“链接不可用/依据缺失/待校核”。
6. 阶段 0 交付 Rust 渲染与链接索引后端（CLI `report`/`evidence`）；阶段 1 交付窗口、复制按钮、保存对话框与只读阅读器 UI。

## 理由

- 单一 Markdown 源消除三处内容不一致，复制/下载自然一致，便于测试逐字比对。
- Markdown 是纯文本，适合本地离线、用户自存、版本 diff 与隐私扫描；不引入浏览器渲染依赖。
- 稳定 ID + manifest/registry 解析，使“所有依据可点开”可被自动化遍历验证，而不是靠 UI 手工检查。
- 断链显式失败符合项目“不编造、不静默降级”的底线。

## 影响

- 文档：SRS v0.5 新增 FR-13/FR-14；TDS v0.2 新增 3.10/3.11 与 11.4；数据字典新增 `REPORT-SCHEMA-001`、`EVIDENCE-LINK-v1`。
- 代码：`crates/app/evidence.rs`、`crates/app/report.rs`、`crates/app/context.rs`、`crates/cli` 新增 `report`/`evidence`。
- 数据/金标准：报告覆盖全部现有规则（18）、出处（10）、资源（13）；新增 `crates/app/tests/report.rs`。
- 隐私/合规：报告与剪贴板文本沿用导出脱敏规则；报告含免责声明；不新增敏感字段。
- 兼容：历史记录仍按原 `StepLog`/`VersionSet` 渲染；链接 ID 不复用、不原地改名。

## 验证

1. `cargo test -p yitool-app --test report`：全绿。
2. `yitool report ... --out /tmp/report.md` 与 stdout 输出逐字一致；保存后可读回。
3. `yitool evidence` 输出 41 条链接，遍历 `resolve_href` 成功率 100%。
4. 构造未知 `rule_id`/`source_ref`/`data_ref`，渲染返回 `MissingRule`/`MissingSource`/`MissingResource`。
5. 空步骤渲染返回 `EmptySteps`；`https://`、`file://`、`..`、百分号编码与未知 kind 链接返回 `InvalidHref`，未知 ID 返回 `LinkNotFound`。
6. 阶段 1 UI 验收：窗口/复制/下载三处文本一致；断链、路径穿越、外部 URL 用例 100% 被拒绝。
