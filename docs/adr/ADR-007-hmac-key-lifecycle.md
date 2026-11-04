# ADR-007: 问事 HMAC 密钥、`key_id` 与轮换语义

- **状态：** 已批准
- **日期：** 2026-11-04
- **关联：** O-03、TF-03、D-08、SRS 3.3.5/6.1、TDS 5.2、ADR-005
- **决策人：** 项目所有者
- **生效版本：** `DB-v0.2` / `SEED-v2`

## 背景

问事原文默认不持久化；历史关联依赖本地密钥 HMAC。阶段 0 已有 `question_hmac`、`hmac_key_id` 列与 mock `KeyProvider`，但尚未冻结：

- 密钥存放位置与生命周期；
- `key_id` 是否随每条记录保存；
- 轮换密钥时历史记录如何处理；
- “关联历史”与“Q 扰动”是否共用密钥/开关。

## 决策

1. **密钥存放：** 真实密钥存入 OS Keychain（macOS Keychain / Windows Credential Manager / Linux Secret Service），应用只持有 `key_id` 与按需取用的密钥句柄。阶段 0 使用 `MockKeyProvider` 验证契约，阶段 1 接入系统 Keychain。
2. **`key_id` 必填：** 任何写入 `question_hmac` 的推演记录必须同时写入 `hmac_key_id`，与是否保存问事原文无关。`DB-v0.2` 以 schema 约束强制：
   ```sql
   CHECK(question_hmac IS NULL OR hmac_key_id IS NOT NULL)
   ```
3. **轮换不重算：** 轮换只影响新记录。旧记录保留原 `question_hmac` 与 `hmac_key_id`，不得用新密钥重算或覆盖；按 `key_id` 查找旧密钥，找不到时显示“关联不可用”，不得删除历史。
4. **关联与扰动分离：** 保存 HMAC 用于历史关联不等于启用 Q 扰动；Q 扰动按 ADR-005 单独开关。关闭关联不影响既有记录。
5. **原文加密保存：** 仅在用户显式确认后加密保存原文，并单独记录加密方案版本；默认不保存。HMAC 密钥不得复用为原文加密密钥。
6. **禁止输出：** 密钥、原文、生辰材料不得进入日志、错误信息、导出或快照；HMAC 与 `key_id` 可进入记录。
7. **HMAC 计算：** NFC 规范化后加域前缀 `QUESTION-HMAC-v1`，输出 64 位小写十六进制；空文本按空字符串处理并提示。

## 影响

- **文档：** 决策登记册 D-08 关闭；存储版本升至 `DB-v0.2`；SRS/TDS 数据字典同步 schema 版本。
- **代码：** `store::SCHEMA_VERSION = DB-v0.2`，新增约束；`domain::VersionSet::db_schema` 同步；新增 `crates/store/tests/hmac_key.rs`。
- **数据/金标准：** 无算法金标准变化；新增关联/轮换/约束测试。
- **隐私/合规：** 强化“HMAC 必须可追溯到 key_id”，避免轮换后历史关联静默失效。

## 验证

- `cargo test -p yitool-store --test hmac_key`：保存原文必须有 HMAC 与 `key_id`；仅关联的 HMAC 也必须有 `key_id`；轮换不重写旧记录。
- `yitool db check` 输出 `schema_version=DB-v0.2`。
- 阶段 1 接入 Keychain 后，补“删除/缺失密钥时历史仍可展示、关联标记不可用”的集成测试。
