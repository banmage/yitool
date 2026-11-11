use std::{fs, path::PathBuf, sync::Mutex};

use app::AppContext;
use chrono::Utc;
use store::{Profile, RunSummary, Setting, Store};
use tauri::{Manager, State};
use yitool_desktop::{
    create_profile, delete_history, delete_profile, draw_nine_color, get_settings, list_history,
    list_profiles, load_form_options, load_history, perform_cast_with_store, rename_profile,
    resolve_evidence as resolve_evidence_entry, save_settings, CastRequest, CastingPayload,
    DrawSelection, FormOptions,
};

struct DesktopState {
    context: AppContext,
    form_options: FormOptions,
    store: Mutex<Store>,
}

#[tauri::command]
fn get_form_options(state: State<'_, DesktopState>) -> FormOptions {
    state.form_options.clone()
}

#[tauri::command]
fn draw_colors() -> Result<DrawSelection, String> {
    draw_nine_color()
}

#[tauri::command]
fn cast_meihua(
    state: State<'_, DesktopState>,
    request: CastRequest,
) -> Result<CastingPayload, String> {
    let store = state
        .store
        .lock()
        .map_err(|_| "本地数据库不可用".to_string())?;
    perform_cast_with_store(&state.context, Some(&store), request)
}

#[tauri::command]
fn history_list(
    state: State<'_, DesktopState>,
    profile_id: String,
) -> Result<Vec<RunSummary>, String> {
    let store = state
        .store
        .lock()
        .map_err(|_| "本地数据库不可用".to_string())?;
    list_history(&store, &profile_id)
}

#[tauri::command]
fn history_get(
    state: State<'_, DesktopState>,
    profile_id: String,
    run_id: String,
) -> Result<Option<app::HistoryDetail>, String> {
    let store = state
        .store
        .lock()
        .map_err(|_| "本地数据库不可用".to_string())?;
    load_history(&store, &profile_id, &run_id)
}

#[tauri::command]
fn history_delete(
    state: State<'_, DesktopState>,
    profile_id: String,
    run_id: String,
) -> Result<(), String> {
    let store = state
        .store
        .lock()
        .map_err(|_| "本地数据库不可用".to_string())?;
    delete_history(&store, &profile_id, &run_id)
}

#[tauri::command]
fn profiles_list(state: State<'_, DesktopState>) -> Result<Vec<Profile>, String> {
    let store = state
        .store
        .lock()
        .map_err(|_| "本地数据库不可用".to_string())?;
    list_profiles(&store)
}

#[tauri::command]
fn profile_create(state: State<'_, DesktopState>, display_name: String) -> Result<Profile, String> {
    let store = state
        .store
        .lock()
        .map_err(|_| "本地数据库不可用".to_string())?;
    create_profile(&store, &display_name, &Utc::now().to_rfc3339())
}

#[tauri::command]
fn profile_rename(
    state: State<'_, DesktopState>,
    profile_id: String,
    display_name: String,
) -> Result<(), String> {
    let store = state
        .store
        .lock()
        .map_err(|_| "本地数据库不可用".to_string())?;
    rename_profile(&store, &profile_id, &display_name)
}

#[tauri::command]
fn profile_delete(state: State<'_, DesktopState>, profile_id: String) -> Result<(), String> {
    let store = state
        .store
        .lock()
        .map_err(|_| "本地数据库不可用".to_string())?;
    delete_profile(&store, &profile_id)
}

#[tauri::command]
fn settings_get(state: State<'_, DesktopState>, profile_id: String) -> Result<Setting, String> {
    let store = state
        .store
        .lock()
        .map_err(|_| "本地数据库不可用".to_string())?;
    get_settings(&store, &profile_id)
}

#[tauri::command]
fn settings_save(state: State<'_, DesktopState>, setting: Setting) -> Result<(), String> {
    let store = state
        .store
        .lock()
        .map_err(|_| "本地数据库不可用".to_string())?;
    save_settings(&store, &setting)
}

#[tauri::command]
fn resolve_evidence(
    state: State<'_, DesktopState>,
    href: String,
) -> Result<app::EvidenceEntry, String> {
    resolve_evidence_entry(&state.context, &href)
}

fn locate_resources(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let resource_dir = app
        .path()
        .resource_dir()
        .map_err(|error| format!("无法定位应用资源目录：{error}"))?;
    let mut candidates = vec![resource_dir.join("resources"), resource_dir];
    candidates.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../resources"));

    candidates
        .into_iter()
        .find(|path| path.join("manifests/data.manifest.json").is_file())
        .ok_or_else(|| "找不到已打包或开发用的本地 resources 资源目录".into())
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let resources = locate_resources(app.handle()).map_err(std::io::Error::other)?;
            let context = AppContext::load(&resources).map_err(std::io::Error::other)?;
            let form_options = load_form_options(&resources).map_err(std::io::Error::other)?;
            let data_dir = app.path().app_data_dir().map_err(std::io::Error::other)?;
            fs::create_dir_all(&data_dir).map_err(std::io::Error::other)?;
            let store = Store::init(data_dir.join("yitool.db")).map_err(std::io::Error::other)?;
            app::history::ensure_default_profile(&store, &Utc::now().to_rfc3339())
                .map_err(std::io::Error::other)?;
            app.manage(DesktopState {
                context,
                form_options,
                store: Mutex::new(store),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_form_options,
            draw_colors,
            cast_meihua,
            history_list,
            history_get,
            history_delete,
            profiles_list,
            profile_create,
            profile_rename,
            profile_delete,
            settings_get,
            settings_save,
            resolve_evidence
        ])
        .run(tauri::generate_context!())
        .expect("启动桌面应用失败");
}
