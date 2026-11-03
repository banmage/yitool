use chrono::{Duration, NaiveDate};

/// 公历日期 → 整数儒略日（JDN，民用日序号，正午起算约定取整到日）。
pub fn jdn_from_ymd(year: i32, month: u32, day: u32) -> i64 {
    let a = (14 - month as i32) / 12;
    let y = year + 4800 - a;
    let m = month as i32 + 12 * a - 3;
    let jdn = day as i32 + (153 * m + 2) / 5 + 365 * y + y / 4 - y / 100 + y / 400 - 32045;
    jdn as i64
}

/// 整数儒略日 → 公历日期。
pub fn ymd_from_jdn(jdn: i64) -> (i32, u32, u32) {
    // 1970-01-01 的 JDN 为 2440588。
    let date = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap() + Duration::days(jdn - 2440588);
    use chrono::Datelike;
    (date.year(), date.month(), date.day())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anchors() {
        assert_eq!(jdn_from_ymd(2000, 1, 1), 2451545);
        assert_eq!(jdn_from_ymd(1900, 1, 31), 2415051);
        assert_eq!(jdn_from_ymd(1970, 1, 1), 2440588);
    }

    #[test]
    fn roundtrip() {
        for (y, m, d) in [(1900, 1, 31), (2026, 11, 3), (2100, 12, 31)] {
            assert_eq!(ymd_from_jdn(jdn_from_ymd(y, m, d)), (y, m, d));
        }
    }
}
