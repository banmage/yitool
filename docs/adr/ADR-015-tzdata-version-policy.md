# ADR-015: tzdata 版本与升级策略

- **状态：** 已批准
- **日期：** 2026-11-05
- **关联：** D-42、CF-09、CAL 6.5、CAL-TZ-002、CAL 9.2
- **决策人：** 项目所有者
- **生效版本：** `CAL-RULE-v0.2` / `CAL-v0.2` / tzdata `2025b`

## 背景

CAL 6.5 要求使用 IANA tzdata，并把版本记录在输出与历史中；CAL 10 的 CF-09 要求冻结“tzdata 版本与升级策略”。项目使用 `chrono-tz` 内置的 IANA 数据库，但此前只输出 `IANA_TZDB_VERSION`，没有明确“当前冻结哪个版本、升级时如何避免改变历史、缓存如何失效”。若不冻结，夏令时与历史时区结果可能随依赖升级静默变化。

## 决策

1. **数据来源：** 使用 `chrono-tz` 内置的 IANA tzdata，不在运行时下载或更新时区数据库，保持离线可用。
2. **当前冻结：** `chrono-tz 0.10.4` 内置 IANA tzdata **2025b**；`Cargo.lock` 固定精确版本与 checksum。每次构建/运行的 `VersionSet.tzdata` 必须来自 `chrono_tz::IANA_TZDB_VERSION`，不得手工写死。
3. **历史复现：** 每次推演记录保存 `tzdata` 版本；历史记录按原版本渲染，不因升级自动重算。若用户显式重算，必须作为新记录并写入新版本。
4. **升级流程：**
   1. 升级 `chrono-tz` 依赖并更新 `Cargo.lock`；
   2. 运行 `cargo test -p yitool-cal --test timezone_dst` 与 `cargo test -p yitool-cal --test anchors`；
   3. 检查受影响的 DST/历史时区样例，必要时新增回归样例；
   4. 更新验收记录中的 tzdata 版本与设备信息；
   5. 重新生成资源 manifest（若版本基线变化）。
5. **缓存：** CAL 缓存键必须包含 `timezone`、`day_bound`、`cal_data_version`、`tzdata_version`；tzdata 升级后旧缓存必须失效或按新版本重建。
6. **状态：** 阶段 0 发布构建报告 `2025b` 时，`CAL-TZDATA` 记为 `VERIFIED`。若依赖或报告版本不一致，立即回退为 `PENDING`，不得带病发布。
7. **范围：** 本决策只冻结版本与升级流程，不承诺 tzdata 未来版本的正确性；未来升级仍须走第 4 条流程并保留旧版本历史。

## 影响

- **文档：** CAL 6.5 补充“内置、离线、版本随输出、历史保留”；数据字典 `CAL-TZDATA` 状态更新；决策登记册新增 D-42 并关闭。
- **代码：** `crates/domain/src/version.rs` 与 `crates/cal/src/engine.rs` 继续从 `chrono_tz::IANA_TZDB_VERSION` 取值；`crates/store` 保存 `tzdata_version`。
- **数据/金标准：** `timezone_dst.rs` 固定 `2026-03-08 02:30 America/New_York → 03:30 -04:00`、`2026-11-01 01:30 → -04:00`；验收记录固定 `2025b`。
- **隐私/合规：** 时区版本属于非敏感元数据；不新增网络访问。

## 验证

- `cargo test -p yitool-cal --test timezone_dst`：DST 重复/缺失与显式时区通过。
- `cargo test -p yitool-store --test schema`：历史记录含 `tzdata_version`。
- `cargo run -p yitool-cli -- cal --datetime "2026-11-03T12:00:00+08:00"`：输出 `versions.tzdata = "2025b"`。
- 升级 `chrono-tz` 后，旧历史记录的 `tzdata` 仍为原值，重算生成新记录。
