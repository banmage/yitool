# ADR-001: 阶段 0 技术栈采用 Tauri 2 + React/TypeScript + Rust + SQLite

- **状态：** 已批准
- **日期：** 2026-11-03
- **关联：** TF-01、TF-02、TF-05、D-02
- **决策人：** 项目所有者
- **生效版本：** APP-v0.1（阶段 0 起）

## 背景

TDS v0.1 推荐 Tauri 2.x + React + TypeScript + Vite + Rust + SQLite，但 TF-01 要求阶段 0 原型验证后冻结。项目即将进入阶段 0，需要确定引擎与存储实现语言。

## 选项

| 选项 | 优点 | 缺点 |
|---|---|---|
| Tauri 2 + React/TS + Rust | 体积小、性能好、类型安全、跨平台安装包简单 | Rust 学习曲线；Linux 需 WebKitGTK 依赖 |
| Tauri 2 + React/TS + Python | 易学规则原型快；Python 生态成熟 | 打包复杂；PyO3/子进程边界增加隐私与性能风险 |
| 纯 Python + PySide6 | 最快出原型 | 跨平台打包与性能较弱；后续迁移成本 |

## 决策

采用 **Tauri 2 + React/TypeScript + Vite + Rust + SQLite（WAL）**。

阶段 0 的实施顺序：

1. 先建 **Rust workspace**：`crates/domain`、`crates/cal`、`crates/seed`、`crates/meihua`、`crates/rules`、`crates/app`、`crates/cli`。
2. 用 Rust 单元测试 + 金标准测试驱动核心；CLI 作为阶段 0 的验收入口。
3. Tauri 壳与 React UI 在阶段 1 接入；阶段 0 不依赖 WebKitGTK 即可运行测试。
4. SQLite 访问放在 `crates/store`；资源加载与 checksum 放在 `crates/rules`。

## 理由

- 与 TDS 推荐一致，避免二次迁移。
- Rust 适合“规则与程序分离 + 强类型 + 高性能 + 单二进制”的目标。
- 阶段 0 用 CLI/测试驱动，不被 GUI 系统依赖阻塞。
- SQLite 单文件、WAL、备份恢复简单，符合本地轻量定位。

## 影响

- **文档：** TDS TF-01 标记为“已批准”；TF-02（数据库加密）、TF-05（CAL 库）仍需单独冻结。
- **代码：** 新增 Rust workspace；CI 需要 Rust stable。
- **数据/金标准：** 金标准用 Rust 测试 + JSON fixture；测试入口 `cargo test`。
- **隐私/合规：** 生辰材料用不实现 `Serialize` 的内存类型；跨进程只传派生结果。

## 验证

- `cargo test` 能在无 GUI 环境运行全部阶段 0 金标准。
- 资源 checksum、日志 Schema、隐私扫描测试通过。
- 阶段 1 能接入 Tauri 2 壳而不改引擎接口。

## 备注

- 本机已安装 Rust 1.99.1（stable）。
- Linux 构建 Tauri GUI 需要 `libwebkit2gtk-4.1-dev` 等系统依赖；阶段 0 不构建 GUI，故不阻塞。
- 若 2 周内 Rust 原型严重受阻，回退到“Tauri + Python 引擎”需新增 ADR，接口契约不变。
