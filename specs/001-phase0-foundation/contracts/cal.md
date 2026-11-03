# CAL Contract: 历法服务

**Version**: CAL-v0.1（实现契约）
**Source**: `docs/历法服务规范 CAL v0.1 .txt`

## 接口

```rust
pub trait CalendarService {
    fn convert_solar_to_lunar(&self, dt: DateTime<FixedOffset>, tz: &Tz, day_bound: DayBound) -> Result<LunarDate, CalError>;
    fn convert_lunar_to_solar(&self, lunar: LunarDate, tz: &Tz) -> Result<DateTime<FixedOffset>, CalError>;
    fn get_jieqi(&self, dt: DateTime<FixedOffset>, tz: &Tz) -> Result<JieqiInfo, CalError>;
    fn get_jieqi_month_branch(&self, dt: DateTime<FixedOffset>, tz: &Tz) -> Result<JieqiMonthInfo, CalError>;
    fn get_lunar_month_index(&self, dt: DateTime<FixedOffset>, tz: &Tz, day_bound: DayBound) -> Result<LunarMonthIndex, CalError>;
    fn get_ganzhi(&self, dt: DateTime<FixedOffset>, tz: &Tz, day_bound: DayBound) -> Result<GanzhiResult, CalError>;
    fn get_effective_date(&self, dt: DateTime<FixedOffset>, day_bound: DayBound) -> Result<NaiveDate, CalError>;
    fn get_xun(&self, dt: DateTime<FixedOffset>, tz: &Tz, day_bound: DayBound) -> Result<XunInfo, CalError>;
    fn get_huangli_basis(&self, dt: DateTime<FixedOffset>, tz: &Tz, day_bound: DayBound) -> Result<HuangliBasis, CalError>;
}
```

## 输出

`CalResult` 字段见 `data-model.md` 2.12。所有输出必须带 `versions` 与 `status`。

## 规则

1. 所有接口显式接收 `tz` 与 `day_bound`，不得依赖系统默认。
2. 年柱以立春时刻切换；月柱以 12 个“节”切换；日柱按 JDN + 基准；时柱按五鼠遁。
3. `LUNAR-MONTH-INDEX` 与 `JIEQI-MONTH-BRANCH` 分离输出，不得互相替代。
4. `ZICHU` 23:00 换日；`ZIZHENG` 00:00 换日；结果标注 `effective_date`。
5. DST 重复取较早偏移；缺失向后平移 1 小时；标注 `ambiguous/gap`。
6. 超出 1900-01-31～2100-12-31 返回 `CAL_OUT_OF_RANGE`。
7. 数据未校核时 `status=PENDING`，界面显示“历法待校核”。

## 错误码

| 错误码 | 含义 |
|---|---|
| `CAL_OUT_OF_RANGE` | 日期超出支持范围 |
| `CAL_INVALID_DATE` | 无效公历日期 |
| `CAL_INVALID_LUNAR_DATE` | 无效农历日期 |
| `CAL_INVALID_TIMEZONE` | 无效 IANA 时区 |
| `CAL_DST_AMBIGUOUS` | 夏令时重复时刻 |
| `CAL_DST_GAP` | 夏令时不存在时刻 |
| `CAL_DATA_UNAVAILABLE` | 历法数据缺失/未冻结 |
| `CAL_JIEQI_UNAVAILABLE` | 节气数据缺失 |

## 数据契约

```json
{
  "lunar_months": [
    {"start_jdn": 2415021, "year": 1900, "month": 1, "leap": false, "days": 29}
  ],
  "jieqi": [
    {"name": "立春", "utc": "2026-02-04T01:46:00Z", "longitude": 315, "kind": "节"}
  ]
}
```

- `lunar_months` 按 `start_jdn` 升序，覆盖 1900-01-31～2100-12-31。
- `jieqi` 覆盖同一范围，24 条/年；`kind` 为 `节|气`。
- 表版本 `CAL-DATA-v0.1`，状态 `PENDING`；正式冻结前须交叉校核。
