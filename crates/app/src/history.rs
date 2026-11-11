//! 历史、档案与设置的轻量编排；只处理非敏感字段。

use domain::StepLog;
use serde::Serialize;
use store::{Profile, RunRecord, RunReport, RunSummary, Setting, Store, StoreError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HistoryError {
    #[error(transparent)]
    Store(#[from] StoreError),
}

#[derive(Debug, Clone, Serialize)]
pub struct HistoryDetail {
    pub run: RunRecord,
    pub report: RunReport,
    pub steps: Vec<StepLog>,
    pub notes: Vec<store::Note>,
}

/// 保存一次成功推演：run、步骤、报告与可选随机快照在同一事务内完成。
#[allow(clippy::too_many_arguments)]
pub fn save_run(
    store: &Store,
    run: &RunRecord,
    steps: &[StepLog],
    report: &RunReport,
    snapshot: Option<(&str, &str, &str)>,
) -> Result<(), HistoryError> {
    store.save_run_bundle(run, steps, report, snapshot)?;
    Ok(())
}

pub fn list_runs(store: &Store, profile_id: &str) -> Result<Vec<RunSummary>, HistoryError> {
    Ok(store.list_run_summaries(profile_id)?)
}

pub fn load_run(
    store: &Store,
    profile_id: &str,
    run_id: &str,
) -> Result<Option<HistoryDetail>, HistoryError> {
    let Some(run) = store.get_run(profile_id, run_id)? else {
        return Ok(None);
    };
    let Some(report) = store.get_run_report(profile_id, run_id)? else {
        return Ok(None);
    };
    let steps = store.list_steps(run_id)?;
    let notes = store.list_notes(run_id)?;
    Ok(Some(HistoryDetail {
        run,
        report,
        steps,
        notes,
    }))
}

pub fn delete_run(store: &Store, profile_id: &str, run_id: &str) -> Result<(), HistoryError> {
    if !store.run_visible(profile_id, run_id)? {
        return Err(StoreError::NotFound(run_id.into()).into());
    }
    store.delete_run(run_id)?;
    Ok(())
}

/// 首次启动时建立默认档案与默认设置；已有档案时直接返回第一个。
pub fn ensure_default_profile(store: &Store, created_at: &str) -> Result<Profile, HistoryError> {
    if let Some(profile) = store.list_profiles()?.into_iter().next() {
        if store.get_setting(&profile.profile_id)?.is_none() {
            store.save_setting(&Setting::default_for(&profile.profile_id))?;
        }
        return Ok(profile);
    }
    let profile = Profile {
        profile_id: "default".into(),
        display_name: "默认档案".into(),
        created_at: created_at.into(),
        is_default: true,
    };
    store.create_profile(&profile)?;
    store.save_setting(&Setting::default_for(&profile.profile_id))?;
    Ok(profile)
}
