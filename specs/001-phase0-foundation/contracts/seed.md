# Seed Contract: 种子链与随机

**Version**: SEED-v2
**Source**: ARS v0.2 第 2 章、TDS v0.1 第 6 章

## 类型

```rust
pub struct SeedInput {
    pub algo: String,          // "SEED-v2"
    pub mode: Mode,            // FIXED | CAST
    pub profile_id: String,
    pub run_time: String,      // ISO8601
    pub timezone: String,      // IANA
    pub day_bound: DayBound,
    pub rule_set: String,      // RULE-v0.2
    pub data_version: String,  // DATA-v0.2
    pub cal_version: String,   // CAL-v0.1
}

pub struct InputSnapshot {
    pub color_scheme: ColorScheme,
    pub color_id: Option<u8>,
    pub aux_type: AuxType,
    pub aux_id: Option<u8>,
    pub traditional_numbers: Option<[u32; 2]>,
    pub question_hmac: String,       // HMAC 或 "NONE"
    pub bazi_present: bool,
    pub liuyao_input_type: LiuyaoInputType,
    pub random_snapshot_digest: Option<String>,
}
```

## 派生

```text
base_seed      = SHA256(canonical_json(SeedInput))
draw_seed      = HMAC-SHA256(base_seed, "DRAW|" + draw_purpose)
draw_result    = rejection_sample(draw_seed, range)
snapshot_digest= SHA256(canonical_json(InputSnapshot))
run_seed       = HMAC-SHA256(base_seed, "RUN|" + snapshot_digest)
sub_seed(e,s,p)= HMAC-SHA256(run_seed, e + "|" + s + "|" + p)
```

## 规则

1. `base_seed` 不含抽卡结果。
2. `draw_seed` 固定模式确定性派生；摇卦模式用 OS CSPRNG 并写快照。
3. 固定模式不调用 OS 熵；`FIXED_DERIVED` 六爻由 `sub_seed` 派生。
4. 含生辰时，生辰材料仅在内存参与 `run_seed`，不进入 `InputSnapshot`。
5. `question_hmac="NONE"` 时，`Q_mod=0`；问事关联与 Q 扰动是两个独立开关（D-07）。
6. A/B 统一条件对照共用 `ab_base_seed`、`ab_draw_seed` 与 `shared_random_snapshot`；`FIXED_DERIVED` 必须取共同快照（D-06）。
7. 日志只记录 `snapshot_digest`、`rng_version`、`bazi_present`，不记录生辰材料。

## 随机

```text
uniform_int(n):
  limit = (256 // n) * n
  loop:
    b = os_random_u8()
    if b < limit: return b % n
```

- 九色抽卡：`n=9`；生肖：`n=12`；铜钱：每枚 `n=2`，三枚求和。
- 六爻概率：6=1/8、7=3/8、8=3/8、9=1/8。
- 禁止直接取模。

## 测试

- 同输入同版本固定模式逐项一致。
- 快照重放一致。
- 基础种子不含抽卡结果。
- 改问事 HMAC 不改变 A/B 共同随机快照（D-06 关闭后）。
- 分布卡方检验。
