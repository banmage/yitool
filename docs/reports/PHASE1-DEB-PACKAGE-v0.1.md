# 阶段 1 Linux .deb 安装包验证记录

**日期：** 2026-11-11
**范围：** `TDS-DEPLOY-001`、D-26、`specs/003-phase1-complete`
**结论：** 通过（Linux amd64 本地验证）

## 构建

```bash
./scripts/package-desktop.sh deb
```

该脚本构建安装包并在产物目录生成 `SHA256SUMS`。

产物：

| 项目 | 值 |
|---|---|
| 文件 | `apps/desktop/src-tauri/target/release/bundle/deb/yitool_0.1.0_amd64.deb` |
| 大小 | 4.67 MiB（4,897,354 bytes） |
| 包名/版本 | `yitool` / `0.1.0` |
| 架构 | `amd64` |
| 依赖 | `libwebkit2gtk-4.1-0`、`libgtk-3-0` |
| SHA-256 | `645c7ecff1063f5efff0549022c93368039abaed0ac8ed427a1e128cb72ddbc9` |

校验和文件位于同一产物目录的 `SHA256SUMS`。重新构建会产生新的哈希，部署时应以当次构建的 `SHA256SUMS` 为准。

> 校验和仅证明文件完整性，不证明来源真实性；阶段 1 安装包未签名（ADR-024）。

## 包内容检查

`dpkg-deb -I` 与 `dpkg-deb -c` 结果：

- `/usr/bin/yitool-desktop`：可执行文件；
- `/usr/lib/yitool/resources/`：只读资源，含 `data/`、`cal/`、`ir/`、`manifests/data.manifest.json`；
- `/usr/share/applications/yitool.desktop`：桌面入口；
- 未包含 `yitool.db`、档案、历史、报告、导出、日志、生辰、问事原文或密钥文件。

## 解包运行验证

```bash
dpkg-deb -x yitool_0.1.0_amd64.deb /tmp/yitool-pkg
cd /tmp/yitool-pkg
DISPLAY=:0 GDK_BACKEND=x11 \
XDG_DATA_HOME=/tmp/yitool-pkg-data/share \
XDG_RUNTIME_DIR=/tmp/yitool-smoke/runtime \
WEBKIT_DISABLE_DMABUF_RENDERER=1 \
./usr/bin/yitool-desktop
```

结果：

- 应用正常启动并保持运行；
- 首次启动在隔离的 `XDG_DATA_HOME` 下创建 `local.yitool.desktop/yitool.db`；
- 资源从包内 `usr/lib/yitool/resources` 加载：临时移开源码树 `resources/` 后再次启动，应用仍能初始化数据库并运行，证明未依赖源码目录回退；
- 未触碰真实用户数据目录。

## 未覆盖项

- Windows x64 NSIS 安装包：本机缺少 Clang/LLD 与 NSIS，未本地构建；已新增 GitHub Actions `desktop-windows-package` 任务在 `windows-latest` 构建并生成 `SHA256SUMS`，安装与运行仍需 Windows 手工验证。
- macOS `.dmg`：无构建环境，不声明支持。
- 代码签名、公证、自动更新：阶段 1 不启用（ADR-024）。
- 安装到系统路径后的桌面菜单启动未验证（解包运行已覆盖资源与数据库初始化）。
