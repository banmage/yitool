use std::path::PathBuf;
use std::process::ExitCode;

use app::{privacy::Needle, AppContext};
use cal::{CalQuery, CalTime, DayBound};
use chrono::{DateTime, NaiveDateTime};
use clap::{Parser, Subcommand};
use domain::VersionSet;
use meihua::MeihuaInput;
use seed::{Mode, SeedInput, SeedScheme};
use serde::Serialize;
use serde_json::json;

#[derive(Parser, Debug)]
#[command(
    name = "yitool",
    version,
    about = "易学学习与推演记录工具（阶段 0 验收入口）"
)]
struct Cli {
    /// 输出格式：json（默认）或 text
    #[arg(long, global = true, default_value = "json")]
    format: String,

    /// 资源根目录（默认 ./resources 或 YITOOL_RESOURCES）
    #[arg(long, global = true)]
    resources: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// 查询农历、节气、四柱、日界与旬空
    Cal {
        #[arg(long)]
        datetime: String,
        #[arg(long, default_value = "Asia/Shanghai")]
        timezone: String,
        #[arg(long, default_value = "ZICHU")]
        day_bound: String,
    },
    /// 生成种子链与输入快照
    Seed {
        #[arg(long)]
        profile: String,
        #[arg(long)]
        run_time: String,
        #[arg(long, default_value = "Asia/Shanghai")]
        timezone: String,
        #[arg(long, default_value = "ZICHU")]
        day_bound: String,
        #[arg(long, default_value = "FIXED")]
        mode: String,
        #[arg(long)]
        color: Option<u8>,
        #[arg(long)]
        aux: Option<u8>,
        #[arg(long, num_args = 2, value_names = ["U0", "D0"])]
        traditional: Option<Vec<u32>>,
        /// 已计算的问事 HMAC（hex）；不要传入原文
        #[arg(long)]
        question_hmac: Option<String>,
        #[arg(long)]
        bazi_present: bool,
        /// 重放用基础种子（hex，32 字节）
        #[arg(long)]
        base_seed: Option<String>,
    },
    /// 梅花起卦（九色或传统数字）
    Meihua {
        #[arg(long)]
        color: Option<u8>,
        #[arg(long)]
        aux: Option<u8>,
        #[arg(long, num_args = 2, value_names = ["U0", "D0"])]
        traditional: Option<Vec<u32>>,
        #[arg(long, default_value_t = 0)]
        q_mod: u32,
        #[arg(long, default_value_t = 0)]
        b_mod: u32,
        #[arg(long)]
        datetime: String,
        #[arg(long, default_value = "Asia/Shanghai")]
        timezone: String,
        #[arg(long, default_value = "ZICHU")]
        day_bound: String,
    },
    /// 校验资源清单与 checksum
    Verify,
    /// 扫描日志/数据库/导出文件中的敏感值
    PrivacyScan {
        #[arg(long)]
        db: Option<PathBuf>,
        #[arg(long)]
        logs: Option<PathBuf>,
        #[arg(long)]
        exports: Option<PathBuf>,
        #[arg(long = "needle")]
        needles: Vec<String>,
    },
    /// 数据库初始化与检查
    Db {
        #[command(subcommand)]
        action: DbAction,
    },
}

#[derive(Subcommand, Debug)]
enum DbAction {
    Init {
        #[arg(long)]
        path: PathBuf,
    },
    Check {
        #[arg(long)]
        path: PathBuf,
    },
}

#[derive(Serialize)]
struct Envelope<T: Serialize> {
    ok: bool,
    data: Option<T>,
    error: Option<ErrorBody>,
    versions: VersionSet,
}

#[derive(Serialize)]
struct ErrorBody {
    code: String,
    message: String,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let resources = cli.resources.clone().unwrap_or_else(app::resources_dir);
    let result = run(&cli, &resources);
    match result {
        Ok((value, text)) => {
            if cli.format == "text" {
                println!("{text}");
            } else {
                let envelope = Envelope {
                    ok: true,
                    data: Some(value),
                    error: None,
                    versions: VersionSet::default(),
                };
                println!("{}", serde_json::to_string_pretty(&envelope).unwrap());
            }
            ExitCode::SUCCESS
        }
        Err((code, message)) => {
            let exit = match code.as_str() {
                "ARGUMENT" => 2,
                "PRIVACY" | "COMPLIANCE" => 4,
                "INTERNAL" => 5,
                _ => 3,
            };
            let envelope: Envelope<serde_json::Value> = Envelope {
                ok: false,
                data: None,
                error: Some(ErrorBody {
                    code: code.clone(),
                    message: message.clone(),
                }),
                versions: VersionSet::default(),
            };
            if cli.format == "text" {
                eprintln!("[{code}] {message}");
            } else {
                println!("{}", serde_json::to_string_pretty(&envelope).unwrap());
            }
            ExitCode::from(exit)
        }
    }
}

fn run(cli: &Cli, resources: &PathBuf) -> Result<(serde_json::Value, String), (String, String)> {
    match &cli.command {
        Command::Cal {
            datetime,
            timezone,
            day_bound,
        } => {
            let ctx = load_context(resources)?;
            let query = CalQuery {
                time: parse_time(datetime)?,
                timezone: timezone.clone(),
                day_bound: parse_day_bound(day_bound)?,
            };
            let result = ctx.cal_query(&query).map_err(|e| app_code(&e))?;
            let text = format!(
                "{} {} 农历 {}{}月{} {} {}",
                result.solar,
                result.ganzhi.day,
                result.lunar.month,
                if result.lunar.leap { "闰" } else { "" },
                result.lunar.day,
                result.ganzhi.year,
                result.ganzhi.month
            );
            Ok((serde_json::to_value(result).unwrap(), text))
        }
        Command::Seed {
            profile,
            run_time,
            timezone,
            day_bound,
            mode,
            color,
            aux,
            traditional,
            question_hmac,
            bazi_present,
            base_seed,
        } => {
            let ctx = load_context(resources)?;
            let scheme = if traditional.is_some() {
                SeedScheme::TraditionalNumber
            } else {
                SeedScheme::NineColor
            };
            let base_seed = match base_seed {
                Some(hex_str) => {
                    let bytes = hex::decode(hex_str)
                        .map_err(|e| ("ARGUMENT".into(), format!("base_seed 非法: {e}")))?;
                    Some(
                        <[u8; 32]>::try_from(bytes.as_slice())
                            .map_err(|_| ("ARGUMENT".into(), "base_seed 必须为 32 字节".into()))?,
                    )
                }
                None => None,
            };
            let input = SeedInput {
                profile_id: profile.clone(),
                run_time: run_time.clone(),
                timezone: timezone.clone(),
                day_bound: day_bound.clone(),
                mode: parse_mode(mode)?,
                scheme,
                color: *color,
                aux: *aux,
                traditional: traditional.as_ref().map(|v| [v[0], v[1]]),
                question_hmac: question_hmac
                    .clone()
                    .filter(|v| v != "NONE" && !v.is_empty()),
                bazi_present: *bazi_present,
                base_seed,
            };
            let chain = ctx.seed(&input).map_err(|e| app_code(&e))?;
            let value = json!({
                "base_seed": chain.base_seed_hex(),
                "draw_seed": chain.draw_seed_hex(),
                "draw_result": chain.draw_result,
                "input_snapshot": chain.input_snapshot,
                "snapshot_digest": chain.snapshot_digest_hex(),
                "run_seed": chain.run_seed_hex(),
                "subseeds": {
                    "meihua": hex::encode(chain.subseed("meihua")),
                    "liuyao": hex::encode(chain.subseed("liuyao")),
                }
            });
            let text = format!(
                "base={} run={} digest={}",
                chain.base_seed_hex(),
                chain.run_seed_hex(),
                chain.snapshot_digest_hex()
            );
            Ok((value, text))
        }
        Command::Meihua {
            color,
            aux,
            traditional,
            q_mod,
            b_mod,
            datetime,
            timezone,
            day_bound,
        } => {
            let ctx = load_context(resources)?;
            let query = CalQuery {
                time: parse_time(datetime)?,
                timezone: timezone.clone(),
                day_bound: parse_day_bound(day_bound)?,
            };
            let input = match traditional {
                Some(v) => MeihuaInput::Traditional {
                    u0: v[0],
                    d0: v[1],
                    q_mod: *q_mod,
                    b_mod: *b_mod,
                },
                None => MeihuaInput::NineColor {
                    color: color.ok_or(("ARGUMENT".into(), "缺少 --color".into()))?,
                    aux: aux.ok_or(("ARGUMENT".into(), "缺少 --aux".into()))?,
                    q_mod: *q_mod,
                    b_mod: *b_mod,
                },
            };
            let result = ctx.meihua(input, &query).map_err(|e| app_code(&e))?;
            let text = format!(
                "{} 之 {}，动{}爻；互 {}；体 {} 用 {}；状态 {}",
                result.original.name,
                result.changed.name,
                result.moving_line,
                result.mutual.name,
                result.ti,
                result.yong,
                result.status.label()
            );
            Ok((serde_json::to_value(result).unwrap(), text))
        }
        Command::Verify => {
            let (manifest, reports) =
                rules::verify_manifest(resources).map_err(|e| ("DATA".into(), e.to_string()))?;
            let failed: Vec<_> = reports.iter().filter(|r| !r.checksum_ok).collect();
            let value = json!({
                "manifest_id": manifest.manifest_id,
                "version": manifest.version,
                "entries": reports,
                "all_ok": failed.is_empty(),
            });
            if !failed.is_empty() {
                return Err(("DATA".into(), format!("{} 个资源校验失败", failed.len())));
            }
            Ok((value, "all resources verified".into()))
        }
        Command::PrivacyScan {
            db,
            logs,
            exports,
            needles,
        } => {
            let mut paths = Vec::new();
            for p in [db, logs, exports].into_iter().flatten() {
                paths.push(p.clone());
            }
            let needles: Vec<Needle> = needles
                .iter()
                .map(|n| Needle::new("sensitive", n.clone()))
                .collect();
            let hits = app::scan_paths(&paths, &needles)
                .map_err(|e| ("INTERNAL".into(), e.to_string()))?;
            let value = json!({ "hits": hits, "hit_count": hits.len() });
            if !hits.is_empty() {
                return Err(("PRIVACY".into(), format!("发现 {} 处敏感值", hits.len())));
            }
            Ok((value, "privacy scan clean".into()))
        }
        Command::Db { action } => match action {
            DbAction::Init { path } => {
                let (store, migration) = store::Store::init_with_report(path)
                    .map_err(|e| ("DATA".into(), e.to_string()))?;
                let check = store.check().map_err(|e| ("DATA".into(), e.to_string()))?;
                let mut value = serde_json::to_value(check).unwrap();
                value["migration"] = serde_json::to_value(migration).unwrap();
                Ok((value, "database initialized".into()))
            }
            DbAction::Check { path } => {
                let store = store::Store::open(path).map_err(|e| ("DATA".into(), e.to_string()))?;
                let check = store.check().map_err(|e| ("DATA".into(), e.to_string()))?;
                Ok((serde_json::to_value(check).unwrap(), "database ok".into()))
            }
        },
    }
}

fn load_context(resources: &PathBuf) -> Result<AppContext, (String, String)> {
    AppContext::load(resources).map_err(|e| ("DATA".into(), e.to_string()))
}

fn parse_time(raw: &str) -> Result<CalTime, (String, String)> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(raw) {
        return Ok(CalTime::Instant(dt));
    }
    for fmt in ["%Y-%m-%dT%H:%M:%S", "%Y-%m-%d %H:%M:%S", "%Y-%m-%dT%H:%M"] {
        if let Ok(naive) = NaiveDateTime::parse_from_str(raw, fmt) {
            return Ok(CalTime::Local(naive));
        }
    }
    Err((
        "ARGUMENT".into(),
        format!("无法解析时间 {raw}；请使用 ISO 8601（含或不含偏移）"),
    ))
}

fn parse_day_bound(raw: &str) -> Result<DayBound, (String, String)> {
    match raw.to_ascii_uppercase().as_str() {
        "ZICHU" => Ok(DayBound::Zichu),
        "ZIZHENG" => Ok(DayBound::Zizheng),
        _ => Err(("ARGUMENT".into(), format!("非法 day_bound: {raw}"))),
    }
}

fn parse_mode(raw: &str) -> Result<Mode, (String, String)> {
    match raw.to_ascii_uppercase().as_str() {
        "FIXED" => Ok(Mode::Fixed),
        "CAST" => Ok(Mode::Cast),
        _ => Err(("ARGUMENT".into(), format!("非法 mode: {raw}"))),
    }
}

fn cal_code(err: &cal::CalError) -> String {
    match err {
        cal::CalError::OutOfRange => "CAL_OUT_OF_RANGE".into(),
        cal::CalError::InvalidDate(_) => "CAL_INVALID_DATE".into(),
        cal::CalError::InvalidTimezone(_) => "CAL_INVALID_TIMEZONE".into(),
        cal::CalError::DataUnavailable(_) => "CAL_DATA_UNAVAILABLE".into(),
        cal::CalError::DataInvalid(_) => "CAL_DATA_UNAVAILABLE".into(),
        cal::CalError::DstGap => "CAL_DST_GAP".into(),
        cal::CalError::DstAmbiguous => "CAL_DST_AMBIGUOUS".into(),
        cal::CalError::Domain(_) => "CAL_DATA_UNAVAILABLE".into(),
    }
}

fn app_code(err: &app::AppError) -> (String, String) {
    match err {
        app::AppError::Cal(e) => (cal_code(e), e.to_string()),
        app::AppError::Resources(_) | app::AppError::Rules(_) => ("DATA".into(), err.to_string()),
        app::AppError::Registry(_) | app::AppError::Compliance(_) => {
            ("DATA".into(), err.to_string())
        }
        app::AppError::Meihua(e) => (
            match e {
                meihua::MeihuaError::InvalidColor(_)
                | meihua::MeihuaError::InvalidAux(_)
                | meihua::MeihuaError::InvalidTraditional
                | meihua::MeihuaError::InvalidPerturbation(_, _) => "ARGUMENT".into(),
                meihua::MeihuaError::Step(_) => "INTERNAL".into(),
            },
            e.to_string(),
        ),
        app::AppError::Seed(e) => (
            match e {
                seed::SeedError::MissingInput(_) | seed::SeedError::InvalidInput(_) => {
                    "ARGUMENT".into()
                }
                seed::SeedError::EntropyUnavailable(_) => "INTERNAL".into(),
            },
            e.to_string(),
        ),
        app::AppError::Store(_) | app::AppError::Io(_) => ("INTERNAL".into(), err.to_string()),
    }
}
