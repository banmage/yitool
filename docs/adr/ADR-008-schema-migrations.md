# ADR-008: SQLite schema 版本化迁移与自动备份

- **状态：** 已批准
- **日期：** 2026-11-04
- **关联：** TF-08、D-17、TDS 6.4/11.2、ADR-007
- **决策人：** 项目所有者
- **生效版本：** `DB-v0.2`

## 背景

阶段 0 原实现用一条 `CREATE TABLE IF NOT EXISTS` 建库，随后无条件把 `meta.schema_version` 写成当前版本。这会带来两个问题：

1. 已存在的旧库不会获得新约束，却会被标记为新版本（静默版本说谎）；
2. 升级没有备份与回滚，失败后可能留下半迁移状态。

ADR-007 将 `DB-v0.1` 升级到 `DB-v0.2`（`question_hmac` 必须带 `hmac_key_id`），必须先建立迁移框架。

## 决策

1. **版本化迁移列表：** 代码内维护有序迁移 `DB-v0.1`、`DB-v0.2`…；`SCHEMA_VERSION` 必须等于最后一个迁移版本（测试断言）。已发布迁移不得修改，只能追加新版本。
2. **按版本执行：** `meta.schema_version` 决定起点，只执行其后的迁移；当前版本等于目标时为空操作（幂等）。
3. **未知版本拒绝：** 数据库有表但无 `meta`，或版本不在迁移列表中时，返回 `UnknownSchemaVersion`，不得猜测或覆盖。
4. **升级前自动备份：** 仅对已有库执行；备份文件与原库同目录，命名 `<db>.bak-<old_version>-<nanos>`；新库无需备份。
5. **事务与回滚：** 所有待执行迁移在同一事务中运行；任一步失败则回滚，版本号保持旧值。迁移期间临时关闭外键，提交前执行 `PRAGMA foreign_key_check`，有违规即回滚。
6. **约束升级方式：** SQLite 不能直接修改 `CHECK`，采用“建新表 → 复制 → 删旧表 → 改名 → 重建索引”，并在外键关闭状态下执行，避免级联删除子表。
7. **可观测：** `Store::migrate` 返回 `MigrationReport { from, to, applied, backup }`；`yitool db init` 在 JSON 中输出 `migration` 字段。
8. **历史数据：** 迁移只改 schema，不改写推演记录内容；旧记录按原版本渲染。

## 影响

- **文档：** 决策登记册 D-17 关闭；数据字典 `DB_SCHEMA_VERSION=DB-v0.2`。
- **代码：** `crates/store` 增加 `MIGRATION_V1_SQL`、`MIGRATION_V2_SQL`、`MIGRATIONS`、`MigrationReport`、`init_with_report`。
- **数据/金标准：** 新库从 `DB-v0.1` 基线连续迁移到 `DB-v0.2`；测试覆盖新库、幂等、v0.1 升级、失败回滚、未知版本。
- **隐私/合规：** 迁移前备份可能包含 HMAC 与 `key_id`，但绝不包含问事原文、生辰或密钥本身；备份文件应纳入用户“彻底删除”范围。

## 验证

- `cargo test -p yitool-store`：
  - `fresh_db_runs_all_migrations_without_backup`
  - `migrate_is_idempotent`
  - `v1_db_is_backed_up_and_upgraded`
  - `failed_migration_rolls_back`
  - `unknown_schema_version_is_rejected`
- `yitool db init` 输出 `schema_version=DB-v0.2` 与 `migration.applied`。
- 对 v0.1 库执行 `db init` 后，同目录存在 `.bak-DB-v0.1-*` 文件。
