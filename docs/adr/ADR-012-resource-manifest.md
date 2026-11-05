# ADR-012: 资源 manifest 与 checksum 规范

- **状态：** 已批准
- **日期：** 2026-11-04
- **关联：** TF-04、D-16、TDS 4.3/9.2/9.3/10.4、SRS 8.3
- **决策人：** 项目所有者
- **生效版本：** `MANIFEST-data` / `DATA-v0.2`

## 背景

TDS 4.3 要求资源文件携带 `version`、`checksum`、`status`、`source`，加载时校验 checksum；但未冻结清单文件形态、校验算法、路径安全、缺失文件处理，以及“启动加载”和“显式 verify”的关系。TDS 还列出 `rules.manifest.json` 与 `data.manifest.json` 两份清单，而阶段 0 实际以单一清单覆盖全部只读资源。若不统一，可能出现“CLI 校验通过但应用加载未校验”或路径穿越绕过。

## 决策

1. **单一清单：** 阶段 0 使用 `resources/manifests/data.manifest.json`（`manifest_id = MANIFEST-data`）统一覆盖 `data/`、`cal/`、`ir/` 下的只读资源；TDS 中的 `rules.manifest.json` 暂合并到该清单，阶段 1 如需拆分须另立 ADR 并保持向后兼容。
2. **清单字段：**
   ```json
   {
     "id": "G-01",
     "path": "data/bagua.json",
     "version": "G-01-v0.2",
     "status": "VERIFIED",
     "source": "ARS v0.2 附录 A / 先天八卦；方位按 G-01 后天方位",
     "sha256": "<64 位小写十六进制>"
   }
   ```
   清单整体含 `manifest_id`、`version`、`generated_by`、`not_frozen_notice`、`entries[]`。
3. **校验算法：** 对资源文件**原始字节**计算 SHA-256，小写十六进制编码；不得先解析 JSON 再规范化序列化，否则格式变化会被误判。
4. **路径安全：** `path` 必须是相对资源根目录的路径；拒绝绝对路径、`..`、根路径与 Windows 前缀，防止清单被篡改后读取任意文件。
5. **启动即校验：** `AppContext::load` 读取清单并校验全部条目；任一条目 checksum 失败或文件缺失即拒绝加载，错误信息列出失败 `id`。不允许“先加载、后提示”。
6. **显式校验：** `yitool verify --resources <dir>` 返回每个条目的 `checksum_ok`、`actual_sha256` 与 `error`；任一失败退出码为 3。它用于 CI、发布前检查与故障定位。
7. **缺失与多余：** 清单条目对应文件缺失记为校验失败；未被清单收录的文件不自动信任，也不参与加载。
8. **`PENDING` 语义：** checksum 通过只证明文件未被篡改，不证明规则已校核。`status = PENDING` 的资源仍可加载，但相关结果必须标“待校核”；`status = VERIFIED` 仅表示数据校核状态，不等于规则集整体冻结。
9. **生成方式：** 清单由 `scripts/gen_manifest.py` 从资源文件生成，不手工编辑；新增、删除或修改资源后必须重新生成并运行校验。
10. **版本与升级：** 资源内容变化必须同步资源 `version` 并重新生成清单；旧版本资源保留以按原版本渲染历史。清单本身版本随 `DATA-v0.2` 基线。
11. **信任边界：** 阶段 0 的 SHA-256 只保证完整性，不保证来源真实性；不引入签名。若要防“清单与资源同时被替换”，阶段 1 再评估签名/发布包校验。

## 影响

- **代码：** `crates/rules/src/manifest.rs` 实现清单解析、路径安全与 SHA-256；`AppContext::load` 增加启动校验；CLI `verify` 保留逐条报告。
- **测试：** `crates/rules/tests/manifest.rs` 覆盖真实清单、篡改、缺失与路径穿越；新增 `crates/app/tests/manifest_guard.rs` 验证应用启动拒绝篡改资源。
- **文档：** TDS 4.3 改为单一清单；数据字典 `MANIFEST-data` 状态更新；D-16 关闭。
- **运维：** 资源变更流程固定为“改资源 → 升版本 → 重新生成清单 → `yitool verify` → 全量测试”。

## 验证

- `cargo test -p yitool-rules -p yitool-app`
- `cargo run -p yitool-cli -- verify --resources resources`
- `python3 scripts/gen_manifest.py`
- 篡改任一资源或把 `path` 改为 `../` 时，加载与校验均必须失败。
