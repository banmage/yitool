# Implementation Plan: 阶段 1 补全

**Branch**: `003-phase1-complete` | **Date**: 2026-11-11 | **Spec**: [spec.md](./spec.md)

## Summary

在 `specs/002` 九色桌面闭环之上补齐 SRS 阶段 1 剩余范围：传统数字输入、随机抽卡、种子链与结构化日志、SQLite 历史、档案与设置。所有计算继续复用 `app`/`meihua`/`seed`/`store`；桌面层只做输入编排、持久化与展示。

## Technical Context

**Language/Version**: Rust stable; TypeScript/React  
**Primary Dependencies**: Tauri 2, React, existing `app`/`cal`/`meihua`/`seed`/`store` crates, `rusqlite`  
**Storage**: 本地 SQLite（`yitool.db`），只保存非敏感记录；数据库 schema 升级到 `DB-v0.3`  
**Testing**: Rust 单元/集成测试、前端 Vitest、现有金标准/隐私/分布/报告测试、`scripts/check.sh`  
**Target Platform**: Tauri 桌面（Linux 起步，保留跨平台配置）  
**Constraints**: 离线；不保存生辰/问事原文；不合成吉凶总分；历史按原版本渲染

## Constitution Check

- **隐私**: 只保存 run/step/快照摘要/报告与版本；输入结构不含生辰与问事原文。
- **溯源**: 每个步骤继续使用 `StepLog` 的 `rule_ids/data_refs/source_ref/rule_level`。
- **版本**: 历史详情使用保存时的报告与版本，不静默重算；数据库迁移版本化并备份。
- **合规**: 结果继续标 `PENDING`、附免责声明；不新增绝对断语或综合评分。
- **阶段范围**: 只补阶段 1；不提前启用焦氏、纳甲、择吉或三引擎。

## Design

### 1. 数据库（DB-v0.3）

- 新增 `run_reports` 表：`run_id`、`scheme`、`status`、`cal_json`、`result_json`、`report_markdown`。
- 重建 `run_steps`，允许 `VERIFIED/FROZEN/DEPRECATED` 状态（当前 schema 只允许 `OK/PENDING/ERROR`）。
- 新增查询：档案列表、run 记录、步骤、报告、笔记；删除仍走级联。
- 迁移前自动备份；失败回滚，不破坏原库。

### 2. 桌面命令

- `cast_meihua`：接受 `mode`、`scheme`、九色/传统数字、`draw`、档案、时间/时区/日界；生成种子链、调用梅花、渲染报告、写入历史。
- `list_history` / `get_history` / `delete_history`：按档案隔离。
- `list_profiles` / `create_profile` / `rename_profile` / `delete_profile`。
- `get_settings` / `save_settings`。
- `get_form_options` 与 `resolve_evidence` 保持现有契约。

### 3. 种子与报告

- `SeedInput` 的 `run_time` 使用本次本地时间字符串；固定模式不引入 OS 熵。
- 随机抽卡在种子链内完成；抽卡结果写入 `InputSnapshot` 与报告摘要。
- 报告增加基础种子、抽卡子种子、快照摘要、推演种子四行；窗口/复制/下载继续共用同一 Markdown。

### 4. 前端

- 顶部档案选择与设置入口。
- 输入模式切换：九色 / 传统数字；手动选择 / 随机抽卡。
- 结果区保留现有结构；新增“历史”面板与详情复用报告对话框。
- 设置面板覆盖默认输入模式、辅助象模式、日界、时区、动画、隐私脱敏。

## Project Structure

```text
crates/store/          DB-v0.3 迁移、历史查询、档案与设置查询
crates/app/            历史编排、报告种子摘要、保存/读取
apps/desktop/src-tauri 桌面命令与 SQLite 生命周期
apps/desktop/src       React 输入模式、历史、档案、设置
specs/003-phase1-complete/
```

## Risks

- 现有 DB-v0.2 的 `run_steps` CHECK 约束不支持 `VERIFIED`；必须迁移，不能静默映射。
- Tauri 在本机可能缺少 GTK/WebKit；命令层测试使用 `--no-default-features`。
- 历史保存会增加隐私扫描面；需要覆盖数据库、报告、导出与日志。
- 前端测试的 API mock 必须同步扩展，避免新增命令导致测试误报。

## Verification

1. `cargo test --workspace`
2. `cargo test -p yitool-desktop --no-default-features`
3. `cd apps/desktop && npm test && npm run build`
4. `./scripts/check.sh`
5. 在具备 GTK/WebKit 的环境运行 `npm run tauri dev` 冒烟。
