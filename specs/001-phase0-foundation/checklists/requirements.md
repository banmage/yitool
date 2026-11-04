# Specification Quality Checklist: 阶段 0 — 可信基础与可复现内核

**Purpose**: Validate specification completeness and quality before proceeding to planning
**Created**: 2026-11-03
**Feature**: [spec.md](../spec.md)

## Content Quality

- [x] No implementation details (languages, frameworks, APIs)
  - 说明：阶段 0 是基础设施特性，规格描述“可验证结果”而非框架/库；技术栈明确留到 plan 与 ADR 决定。
- [x] Focused on user value and business needs
  - 说明：以“可复现、历法可信、可溯源、隐私可验证、金标准验收、档案隔离”为用户价值。
- [x] Written for non-technical stakeholders
  - 说明：领域术语（种子链、CAL、金标准）在 `docs/` 有定义；规格避免代码级细节。
- [x] All mandatory sections completed
  - 说明：User Scenarios、Requirements、Success Criteria、Assumptions 均已填写。

## Requirement Completeness

- [x] No [NEEDS CLARIFICATION] markers remain
  - 说明：关键分歧已转为 Assumptions 与决策登记册 D-01～D-20；未阻塞规格。
- [x] Requirements are testable and unambiguous
  - 说明：FR-001～FR-023 均有可观察行为或明确状态。
- [x] Success criteria are measurable
  - 说明：SC-001～SC-010 均含比率、时间、数量或 0 命中。
- [x] Success criteria are technology-agnostic (no implementation details)
  - 说明：未绑定 Tauri/Rust/SQLite；仅描述结果与验收方式。
- [x] All acceptance scenarios are defined
  - 说明：6 个用户故事均含 Given/When/Then 场景。
- [x] Edge cases are identified
  - 说明：闰月、日界、DST、越界、资源损坏、PENDING、颜色越界、密钥、删除、动画中断、随机源失败。
- [x] Scope is clearly bounded
  - 说明：Assumptions 明确阶段 0 不做焦氏 4096、纳甲装卦、择吉权重、古籍全文与完整 UI。
- [x] Dependencies and assumptions identified
  - 说明：依赖五份规范、决策登记册、基准数据与 OS Keychain。

## Feature Readiness

- [x] All functional requirements have clear acceptance criteria
  - 说明：每个 FR 均映射到用户故事或 Success Criteria。
- [x] User scenarios cover primary flows
  - 说明：覆盖固定/摇卦、CAL、溯源、隐私、测试、档案六条主流程。
- [x] Feature meets measurable outcomes defined in Success Criteria
  - 说明：SC 与 FR 一一对应，可在阶段 0 验收。
- [x] No implementation details leak into specification
  - 说明：仅保留“本地存储、OS Keychain、测试入口”等必要能力描述。

## Notes

- 规格已可进入 `/speckit-plan`；建议先关闭决策登记册 D-01～D-11，或在 plan 中以 ADR 形式冻结。
- 阶段 0 的“用户”主要是开发者/维护者与家庭使用者；用户故事以可验证结果表达。
- 若 D-11（忌神/弱势卦位）选择“删除 FR-06”，需同步修订 SRS 与规格。
