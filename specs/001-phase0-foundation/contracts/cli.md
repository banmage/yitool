# CLI Contract: `yitool`

**Version**: CLI-v0.1
**Purpose**: 阶段 0 的可执行验收入口；阶段 1 由 Tauri 命令层复用同一库接口。

## 通用约定

- 所有成功输出为 JSON（`--format json`，默认），人类可读模式 `--format text`。
- 所有输出包含 `ok`、`data`/`error`、`versions`。
- 错误退出码：`0` 成功；`2` 参数错误；`3` 数据/资源错误；`4` 隐私/合规错误；`5` 内部错误。
- 所有时间参数使用 ISO 8601 + 时区偏移；`--timezone` 使用 IANA 名称。
- 所有命令禁止在日志/输出中打印生辰、问事原文、HMAC 密钥。

## `yitool cal`

查询历法结果。

```bash
yitool cal \
  --datetime "2026-11-03T12:00:00+08:00" \
  --timezone "Asia/Shanghai" \
  --day-bound ZICHU \
  --format json
```

输出：`CalResult`（见 `data-model.md` 2.12）。

错误：`CAL_OUT_OF_RANGE`、`CAL_INVALID_DATE`、`CAL_INVALID_TIMEZONE`、`CAL_DST_AMBIGUOUS`、`CAL_DST_GAP`、`CAL_DATA_UNAVAILABLE`。

## `yitool seed`

生成种子链并输出快照（不含生辰）。

```bash
yitool seed \
  --profile "family-a" \
  --run-time "2026-11-03T12:00:00+08:00" \
  --timezone "Asia/Shanghai" \
  --day-bound ZICHU \
  --mode FIXED \
  --color 1 \
  --aux-type ZODIAC \
  --aux 3 \
  --question-hmac NONE
```

输出：

```json
{
  "ok": true,
  "data": {
    "base_seed": "hex",
    "draw_seed": "hex",
    "draw_result": {"color_id": 1, "aux_id": 3},
    "input_snapshot": { "...": "..." },
    "snapshot_digest": "hex",
    "run_seed": "hex"
  },
  "versions": {"seed_algo": "SEED-v2"}
}
```

## `yitool meihua`

九色模式：

```bash
yitool meihua \
  --color 1 --aux 3 \
  --datetime "2026-11-03T12:00:00+08:00" \
  --timezone "Asia/Shanghai" --day-bound ZICHU
```

传统数字模式：

```bash
yitool meihua --traditional 9 16 \
  --datetime "2026-11-03T12:00:00+08:00" \
  --timezone "Asia/Shanghai" --day-bound ZICHU
```

输出：本卦、互卦、变卦、动爻、体用、旺衰、`jishen`（阶段 0 为 `PENDING`）、`steps[]`、`versions`。

## `yitool verify`

校验资源 manifest 与 checksum。

```bash
yitool verify --resources resources
```

输出：每个资源 `id/status/checksum_ok`；任一失败退出码 3。

## `yitool privacy-scan`

扫描日志/数据库/导出文件是否含敏感值。

```bash
yitool privacy-scan \
  --db /tmp/app.db \
  --logs /tmp/logs \
  --exports /tmp/exports \
  --needle "1990-01-01 12:00" \
  --needle "问事原文"
```

输出：`hits[]`（文件、偏移、类型）；命中退出码 4。

## `yitool db`

```bash
yitool db init --path /tmp/app.db
yitool db check --path /tmp/app.db
```

初始化 schema、WAL、外键；`check` 返回 schema 版本与完整性。

## 兼容性

- 新增命令/字段必须升 `CLI-v0.x`，旧字段保留至少一个小版本。
- 阶段 1 的 Tauri 命令必须复用同一输入/输出结构，不另起一套。
