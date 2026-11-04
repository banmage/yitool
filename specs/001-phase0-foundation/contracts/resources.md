# Resource Contract: 只读资源与 manifest

**Version**: DATA-v0.2
**Source**: TDS v0.1 4.3、ARS v0.2 附录 D、`docs/04-DATA-DICTIONARY.md`

## manifest

`resources/manifests/data.manifest.json`：

```json
{
  "manifest_id": "MANIFEST-data",
  "version": "DATA-v0.2",
  "generated_at": "2026-11-03T00:00:00Z",
  "entries": [
    {
      "id": "HEX-01",
      "path": "data/hex64.json",
      "version": "DATA-v0.2",
      "status": "PENDING",
      "source": "ARS v0.2 附录 A",
      "sha256": "hex",
      "updated_at": "2026-11-03"
    }
  ]
}
```

## 加载规则

1. 启动时读取 manifest，校验每个文件存在且 SHA-256 匹配。
2. 校验失败：拒绝加载该资源，返回 `RESOURCE_CHECKSUM_MISMATCH`；不得静默使用。
3. `PENDING` 资源可加载，但相关结果必须标“待校核”。
4. 颜色资源只有 `COLOR-NUM-02`（九色唯一标准）；不存在其他色制的只读资源。
5. `DEPRECATED` 资源不加载；历史按原版本资源渲染。
6. 资源升级不修改历史记录。

## 资源文件头

每个 JSON 数据文件建议自带元信息：

```json
{
  "meta": {
    "id": "HEX-01",
    "version": "DATA-v0.2",
    "status": "PENDING",
    "source": "ARS v0.2 附录 A",
    "updated_at": "2026-11-03"
  },
  "data": []
}
```

## 错误码

| 错误码 | 含义 |
|---|---|
| `RESOURCE_NOT_FOUND` | 文件缺失 |
| `RESOURCE_CHECKSUM_MISMATCH` | 校验和不匹配 |
| `RESOURCE_VERSION_MISMATCH` | 版本不兼容 |
| `RESOURCE_PARSE_ERROR` | JSON 解析失败 |
| `RESOURCE_PENDING` | 资源待校核（警告，不阻断加载） |
