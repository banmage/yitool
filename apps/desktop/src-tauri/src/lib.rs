use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use app::{AppContext, EvidenceEntry};
use cal::{CalQuery, CalResult, CalTime, DayBound};
use chrono::{NaiveDateTime, Utc};
use domain::VersionSet;
use meihua::{MeihuaInput, MeihuaResult};
use seed::{Mode, SeedChain, SeedInput, SeedScheme};
use serde::{Deserialize, Serialize};
use store::{Profile, RunRecord, RunReport, RunSummary, Setting, Store};

static NEXT_RUN_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CastRequest {
    #[serde(default = "default_mode")]
    pub mode: String,
    #[serde(default = "default_scheme")]
    pub scheme: String,
    #[serde(default)]
    pub color_id: Option<u8>,
    #[serde(default)]
    pub aux_id: Option<u8>,
    #[serde(default)]
    pub traditional: Option<[u32; 2]>,
    #[serde(default)]
    pub profile_id: Option<String>,
    pub datetime: String,
    pub timezone: String,
    pub day_bound: String,
}

fn default_mode() -> String {
    "FIXED".into()
}

fn default_scheme() -> String {
    "NINE_COLOR".into()
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeedView {
    pub mode: String,
    pub base_seed: String,
    pub draw_seed: String,
    pub snapshot_digest: String,
    pub run_seed: String,
    pub color_id: Option<u8>,
    pub aux_id: Option<u8>,
    pub traditional: Option<[u32; 2]>,
}

impl From<&SeedChain> for SeedView {
    fn from(chain: &SeedChain) -> Self {
        Self {
            mode: format!("{:?}", chain.input_snapshot.color_scheme),
            base_seed: chain.base_seed_hex(),
            draw_seed: chain.draw_seed_hex(),
            snapshot_digest: chain.snapshot_digest_hex(),
            run_seed: chain.run_seed_hex(),
            color_id: chain.draw_result.color_id,
            aux_id: chain.draw_result.aux_id,
            traditional: chain.draw_result.traditional_numbers,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CastingPayload {
    pub run_id: String,
    pub created_at: String,
    pub mode: String,
    pub scheme: String,
    pub cal: CalResult,
    pub result: MeihuaResult,
    pub versions: VersionSet,
    pub report_markdown: String,
    pub seed: SeedView,
    pub stored: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FormOptions {
    pub colors: Vec<ColorOption>,
    pub zodiacs: Vec<ZodiacOption>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
pub struct ColorOption {
    pub color_id: u8,
    pub name: String,
    pub element: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
pub struct ZodiacOption {
    pub zodiac_id: u8,
    pub name: String,
    pub branch: String,
}

#[derive(Debug, Deserialize)]
struct ColorResource {
    status: String,
    nine_color: Vec<ColorOption>,
}

#[derive(Debug, Deserialize)]
struct ZodiacResource {
    status: String,
    items: Vec<ZodiacOption>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DrawSelection {
    pub color_id: u8,
    pub aux_id: u8,
}

pub fn draw_nine_color() -> Result<DrawSelection, String> {
    let color_id = seed::secure_uniform_1_to(9).map_err(|e| format!("随机源不可用：{e}"))? as u8;
    let aux_id = seed::secure_uniform_1_to(12).map_err(|e| format!("随机源不可用：{e}"))? as u8;
    Ok(DrawSelection { color_id, aux_id })
}

/// 兼容旧调用：不持久化，仅完成一次推演。
pub fn perform_cast(context: &AppContext, request: CastRequest) -> Result<CastingPayload, String> {
    perform_cast_with_store(context, None, request)
}

/// 推演并可选写入本地历史；桌面命令始终传入已初始化的 `Store`。
pub fn perform_cast_with_store(
    context: &AppContext,
    store: Option<&Store>,
    request: CastRequest,
) -> Result<CastingPayload, String> {
    let local_time = parse_local_time(&request.datetime)?;
    let day_bound = match request.day_bound.as_str() {
        "ZICHU" => DayBound::Zichu,
        "ZIZHENG" => DayBound::Zizheng,
        _ => return Err("日界必须为 ZICHU 或 ZIZHENG".into()),
    };
    if request.timezone.trim().is_empty() {
        return Err("时区不能为空".into());
    }
    let mode = parse_mode(&request.mode)?;
    let scheme = parse_scheme(&request.scheme)?;
    let profile_id = request
        .profile_id
        .clone()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "default".into());

    if let Some(store) = store {
        if store
            .get_profile(&profile_id)
            .map_err(|error| error.to_string())?
            .is_none()
        {
            return Err(format!("档案不存在：{profile_id}"));
        }
    }

    let (color, aux, traditional) = match scheme {
        SeedScheme::NineColor => {
            let color = request
                .color_id
                .ok_or_else(|| "九色模式必须选择九色".to_string())?;
            let aux = request
                .aux_id
                .ok_or_else(|| "九色模式必须选择生肖".to_string())?;
            if !(1..=9).contains(&color) {
                return Err(format!("九色 color_id 越界：{color}（应为 1..=9）"));
            }
            if !(1..=12).contains(&aux) {
                return Err(format!("生肖 aux_id 越界：{aux}（应为 1..=12）"));
            }
            (Some(color), Some(aux), None)
        }
        SeedScheme::TraditionalNumber => {
            let numbers = request
                .traditional
                .ok_or_else(|| "传统数字模式必须输入两个正整数".to_string())?;
            if numbers[0] == 0 || numbers[1] == 0 {
                return Err("传统数字必须为正整数".into());
            }
            (None, None, Some(numbers))
        }
    };

    let query = CalQuery {
        time: CalTime::Local(local_time),
        timezone: request.timezone.clone(),
        day_bound,
    };
    let seed_input = SeedInput {
        profile_id: profile_id.clone(),
        run_time: request.datetime.clone(),
        timezone: request.timezone.clone(),
        day_bound: request.day_bound.clone(),
        mode,
        scheme,
        color,
        aux,
        traditional,
        question_hmac: None,
        bazi_present: false,
        base_seed: None,
    };
    let chain = context.seed(&seed_input).map_err(|e| e.to_string())?;

    let input = match scheme {
        SeedScheme::NineColor => MeihuaInput::NineColor {
            color: chain
                .draw_result
                .color_id
                .ok_or_else(|| "种子链缺少九色结果".to_string())?,
            aux: chain
                .draw_result
                .aux_id
                .ok_or_else(|| "种子链缺少生肖结果".to_string())?,
            q_mod: 0,
            b_mod: 0,
        },
        SeedScheme::TraditionalNumber => {
            let numbers = chain
                .draw_result
                .traditional_numbers
                .ok_or_else(|| "种子链缺少传统数字".to_string())?;
            MeihuaInput::Traditional {
                u0: numbers[0],
                d0: numbers[1],
                q_mod: 0,
                b_mod: 0,
            }
        }
    };

    let cal = context
        .cal_query(&query)
        .map_err(|error| error.to_string())?;
    let result = context
        .meihua(input, &query)
        .map_err(|error| error.to_string())?;

    let now = Utc::now();
    let created_at = now.to_rfc3339();
    let run_id = format!(
        "MEIHUA-{}-{}",
        now.format("%Y%m%dT%H%M%S%.3fZ"),
        NEXT_RUN_ID.fetch_add(1, Ordering::Relaxed)
    );
    let versions = context.versions();
    let report_markdown = context
        .meihua_report(
            &result,
            &run_id,
            &created_at,
            request.mode.as_str(),
            Some(&chain),
        )
        .map_err(|error| error.to_string())?;

    let stored = if let Some(store) = store {
        let run = RunRecord {
            run_id: run_id.clone(),
            profile_id: profile_id.clone(),
            created_at: created_at.clone(),
            mode: request.mode.clone(),
            engine_scope: "MEIHUA".into(),
            question_hmac: None,
            question_saved: false,
            bazi_present: false,
            hmac_key_id: None,
            random_snapshot_digest: Some(chain.snapshot_digest_hex()),
            versions: versions.clone(),
            disclaimer_present: true,
        };
        let report = RunReport {
            run_id: run_id.clone(),
            scheme: result.scheme.clone(),
            status: result.status.label().into(),
            original_name: result.original.name.clone(),
            changed_name: result.changed.name.clone(),
            moving_line: result.moving_line,
            formal_allowed: result.formal_allowed,
            cal_json: serde_json::to_string(&cal)
                .map_err(|error| format!("历法结果序列化失败：{error}"))?,
            result_json: serde_json::to_string(&result)
                .map_err(|error| format!("推演结果序列化失败：{error}"))?,
            report_markdown: report_markdown.clone(),
        };
        let snapshot_json = serde_json::to_string(&chain.input_snapshot)
            .map_err(|error| format!("输入快照序列化失败：{error}"))?;
        let digest = chain.snapshot_digest_hex();
        app::history::save_run(
            store,
            &run,
            &result.steps,
            &report,
            Some((&snapshot_json, &digest, "RNG-v1")),
        )
        .map_err(|error| error.to_string())?;
        true
    } else {
        false
    };

    Ok(CastingPayload {
        run_id,
        created_at,
        mode: request.mode,
        scheme: result.scheme.clone(),
        cal,
        result,
        versions,
        report_markdown,
        seed: SeedView::from(&chain),
        stored,
    })
}

fn parse_mode(value: &str) -> Result<Mode, String> {
    match value.to_ascii_uppercase().as_str() {
        "FIXED" => Ok(Mode::Fixed),
        "CAST" => Ok(Mode::Cast),
        _ => Err("推演模式必须为 FIXED 或 CAST".into()),
    }
}

fn parse_scheme(value: &str) -> Result<SeedScheme, String> {
    match value.to_ascii_uppercase().as_str() {
        "NINE_COLOR" => Ok(SeedScheme::NineColor),
        "TRADITIONAL_NUMBER" => Ok(SeedScheme::TraditionalNumber),
        _ => Err("输入方案必须为 NINE_COLOR 或 TRADITIONAL_NUMBER".into()),
    }
}

pub fn resolve_evidence(context: &AppContext, href: &str) -> Result<EvidenceEntry, String> {
    context
        .evidence_index()
        .map_err(|error| error.to_string())?
        .resolve(href)
        .cloned()
        .map_err(|error| error.to_string())
}

pub fn parse_local_time(value: &str) -> Result<NaiveDateTime, String> {
    NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M")
        .or_else(|_| NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S"))
        .map_err(|_| "起卦时间格式无效，请填写有效的本地日期与时间".into())
}

pub fn load_form_options(resources: &Path) -> Result<FormOptions, String> {
    let colors: ColorResource = read_json(resources.join("data/color_num.json"))?;
    let zodiacs: ZodiacResource = read_json(resources.join("data/zodiac.json"))?;
    if colors.status != "VERIFIED"
        || zodiacs.status != "VERIFIED"
        || colors.nine_color.len() != 9
        || zodiacs.items.len() != 12
    {
        return Err("九色或生肖资源未校核，无法初始化输入选项".into());
    }
    Ok(FormOptions {
        colors: colors.nine_color,
        zodiacs: zodiacs.items,
    })
}

fn read_json<T: for<'de> Deserialize<'de>>(path: PathBuf) -> Result<T, String> {
    let content = fs::read_to_string(&path)
        .map_err(|error| format!("读取输入资源失败（{}）：{error}", path.display()))?;
    serde_json::from_str(&content)
        .map_err(|error| format!("解析输入资源失败（{}）：{error}", path.display()))
}

pub fn list_history(store: &Store, profile_id: &str) -> Result<Vec<RunSummary>, String> {
    app::history::list_runs(store, profile_id).map_err(|error| error.to_string())
}

pub fn load_history(
    store: &Store,
    profile_id: &str,
    run_id: &str,
) -> Result<Option<app::HistoryDetail>, String> {
    app::history::load_run(store, profile_id, run_id).map_err(|error| error.to_string())
}

pub fn delete_history(store: &Store, profile_id: &str, run_id: &str) -> Result<(), String> {
    app::history::delete_run(store, profile_id, run_id).map_err(|error| error.to_string())
}

pub fn list_profiles(store: &Store) -> Result<Vec<Profile>, String> {
    store.list_profiles().map_err(|error| error.to_string())
}

pub fn create_profile(
    store: &Store,
    display_name: &str,
    created_at: &str,
) -> Result<Profile, String> {
    if display_name.trim().is_empty() {
        return Err("档案名称不能为空".into());
    }
    if display_name.chars().count() > 50 {
        return Err("档案名称不能超过 50 个字符".into());
    }
    let profile = Profile {
        profile_id: format!("profile-{}", NEXT_RUN_ID.fetch_add(1, Ordering::Relaxed)),
        display_name: display_name.trim().into(),
        created_at: created_at.into(),
        is_default: false,
    };
    store
        .create_profile(&profile)
        .map_err(|error| error.to_string())?;
    store
        .save_setting(&Setting::default_for(&profile.profile_id))
        .map_err(|error| error.to_string())?;
    Ok(profile)
}

pub fn rename_profile(store: &Store, profile_id: &str, display_name: &str) -> Result<(), String> {
    store
        .rename_profile(profile_id, display_name)
        .map_err(|error| error.to_string())
}

pub fn delete_profile(store: &Store, profile_id: &str) -> Result<(), String> {
    let profile = store
        .get_profile(profile_id)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("档案不存在：{profile_id}"))?;
    if profile.is_default {
        return Err("默认档案不可删除".into());
    }
    store
        .delete_profile(profile_id)
        .map_err(|error| error.to_string())
}

pub fn get_settings(store: &Store, profile_id: &str) -> Result<Setting, String> {
    Ok(store
        .get_setting(profile_id)
        .map_err(|error| error.to_string())?
        .unwrap_or_else(|| Setting::default_for(profile_id)))
}

pub fn save_settings(store: &Store, setting: &Setting) -> Result<(), String> {
    if store
        .get_profile(&setting.profile_id)
        .map_err(|error| error.to_string())?
        .is_none()
    {
        return Err(format!("档案不存在：{}", setting.profile_id));
    }
    store
        .save_setting(setting)
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resources() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../resources")
    }

    fn temp_store() -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::init(dir.path().join("app.db")).unwrap();
        app::history::ensure_default_profile(&store, "2026-11-11T12:00:00+08:00").unwrap();
        (dir, store)
    }

    fn nine_color_request(color: u8, aux: u8) -> CastRequest {
        CastRequest {
            mode: "FIXED".into(),
            scheme: "NINE_COLOR".into(),
            color_id: Some(color),
            aux_id: Some(aux),
            traditional: None,
            profile_id: Some("default".into()),
            datetime: "2026-11-06T12:30".into(),
            timezone: "Asia/Shanghai".into(),
            day_bound: "ZICHU".into(),
        }
    }

    #[test]
    fn form_options_come_from_verified_resource_tables() {
        let options = load_form_options(&resources()).unwrap();

        assert_eq!(options.colors.len(), 9);
        assert_eq!(options.colors.first().unwrap().name, "赤色");
        assert_eq!(options.zodiacs.len(), 12);
        assert_eq!(options.zodiacs.first().unwrap().name, "鼠");
        let serialized = serde_json::to_value(options).unwrap();
        assert!(serialized["colors"][0].get("colorId").is_some());
        assert!(serialized["zodiacs"][0].get("zodiacId").is_some());
    }

    #[test]
    fn malformed_local_time_is_rejected() {
        assert!(parse_local_time("not-a-date").is_err());
        assert!(parse_local_time("2026-02-30T12:00").is_err());
    }

    #[test]
    fn accepted_local_time_formats_are_explicit() {
        assert!(parse_local_time("2026-11-06T12:30").is_ok());
        assert!(parse_local_time("2026-11-06T12:30:45").is_ok());
    }

    #[test]
    fn cast_uses_existing_engine_and_keeps_pending_state_in_report() {
        let context = AppContext::load(resources()).unwrap();
        let (_dir, store) = temp_store();
        let payload =
            perform_cast_with_store(&context, Some(&store), nine_color_request(1, 3)).unwrap();

        assert!(!payload.result.formal_allowed);
        assert_eq!(payload.result.status.label(), "PENDING");
        assert!(!payload.result.steps.is_empty());
        assert!(payload.report_markdown.contains("免责声明"));
        assert!(payload.report_markdown.contains("不得作为正式结论"));
        assert!(payload.report_markdown.contains("基础种子"));
        assert!(payload.report_markdown.contains("九色为系统自定义扩展象数"));
        assert_eq!(payload.versions, context.versions());
        assert!(payload.stored);

        let serialized = serde_json::to_value(&payload).unwrap();
        assert!(serialized.get("runId").is_some());
        assert!(serialized.get("reportMarkdown").is_some());
        assert!(serialized.get("seed").is_some());
        assert!(serialized["result"].get("formal_allowed").is_some());
        assert!(serialized["versions"].get("cal_data").is_some());

        let history = list_history(&store, "default").unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].run_id, payload.run_id);
        let detail = load_history(&store, "default", &payload.run_id)
            .unwrap()
            .unwrap();
        assert_eq!(detail.report.report_markdown, payload.report_markdown);
        assert_eq!(detail.steps.len(), payload.result.steps.len());
    }

    #[test]
    fn cast_supports_traditional_numbers_and_reports_l2() {
        let context = AppContext::load(resources()).unwrap();
        let mut request = nine_color_request(1, 3);
        request.scheme = "TRADITIONAL_NUMBER".into();
        request.color_id = None;
        request.aux_id = None;
        request.traditional = Some([9, 16]);

        let payload = perform_cast(&context, request).unwrap();
        assert_eq!(payload.result.scheme, "TRADITIONAL_NUMBER");
        assert_eq!(payload.result.original.name, "天地否");
        assert_eq!(payload.result.moving_line, 1);
        assert_eq!(payload.result.changed.name, "天雷无妄");
        assert_eq!(payload.seed.traditional, Some([9, 16]));
    }

    #[test]
    fn cast_rejects_out_of_range_color_and_invalid_day_bound() {
        let context = AppContext::load(resources()).unwrap();
        let invalid_color = perform_cast(&context, nine_color_request(0, 3));
        assert!(invalid_color.is_err());

        let mut request = nine_color_request(1, 3);
        request.day_bound = "UNKNOWN".into();
        assert!(perform_cast(&context, request).is_err());
    }

    #[test]
    fn cast_rejects_zero_traditional_number() {
        let context = AppContext::load(resources()).unwrap();
        let mut request = nine_color_request(1, 3);
        request.scheme = "TRADITIONAL_NUMBER".into();
        request.color_id = None;
        request.aux_id = None;
        request.traditional = Some([0, 16]);
        assert!(perform_cast(&context, request).is_err());
    }

    #[test]
    fn history_is_isolated_by_profile_and_delete_is_scoped() {
        let context = AppContext::load(resources()).unwrap();
        let (_dir, store) = temp_store();
        let payload =
            perform_cast_with_store(&context, Some(&store), nine_color_request(1, 3)).unwrap();
        let other = create_profile(&store, "家人 B", "2026-11-11T12:00:00+08:00").unwrap();

        assert!(list_history(&store, &other.profile_id).unwrap().is_empty());
        let mut request = nine_color_request(2, 4);
        request.profile_id = Some(other.profile_id.clone());
        let other_payload = perform_cast_with_store(&context, Some(&store), request).unwrap();

        assert!(delete_history(&store, &other.profile_id, &payload.run_id).is_err());
        delete_history(&store, &other.profile_id, &other_payload.run_id).unwrap();
        assert_eq!(list_history(&store, "default").unwrap().len(), 1);
        assert!(list_history(&store, &other.profile_id).unwrap().is_empty());
    }

    #[test]
    fn evidence_resolution_rejects_external_and_unknown_links() {
        let context = AppContext::load(resources()).unwrap();
        assert!(resolve_evidence(&context, "https://example.com").is_err());
        assert!(resolve_evidence(&context, "yitool://rule/UNKNOWN").is_err());
    }

    #[test]
    fn draw_selection_stays_in_range() {
        for _ in 0..100 {
            let draw = draw_nine_color().unwrap();
            assert!((1..=9).contains(&draw.color_id));
            assert!((1..=12).contains(&draw.aux_id));
        }
    }
}
