use domain::VersionSet;
use meihua::MeihuaResult;
use seed::SeedChain;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ExportSnapshot {
    pub schema: String,
    pub run_id: String,
    pub created_at: String,
    pub bazi_present: bool,
    pub input_snapshot: seed::InputSnapshot,
    pub draw_result: seed::DrawResult,
    pub snapshot_digest: String,
    pub run_seed: String,
    pub meihua: Option<SafeMeihua>,
    pub versions: VersionSet,
    pub disclaimer: String,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SafeMeihua {
    pub scheme: String,
    pub original: String,
    pub mutual: String,
    pub changed: String,
    pub moving_line: u8,
    pub ti: String,
    pub yong: String,
    pub status: String,
    pub formal_allowed: bool,
}

/// 导出快照：只含非敏感字段；不含生辰材料、问事原文、HMAC 密钥。
pub fn export_snapshot(
    run_id: &str,
    created_at: &str,
    chain: &SeedChain,
    meihua: Option<&MeihuaResult>,
    versions: &VersionSet,
) -> ExportSnapshot {
    ExportSnapshot {
        schema: "SNAPSHOT-v1".into(),
        run_id: run_id.into(),
        created_at: created_at.into(),
        bazi_present: chain.input_snapshot.bazi_present,
        input_snapshot: chain.input_snapshot.clone(),
        draw_result: chain.draw_result.clone(),
        snapshot_digest: chain.snapshot_digest_hex(),
        run_seed: chain.run_seed_hex(),
        meihua: meihua.map(|m| SafeMeihua {
            scheme: m.scheme.clone(),
            original: m.original.name.clone(),
            mutual: m.mutual.name.clone(),
            changed: m.changed.name.clone(),
            moving_line: m.moving_line,
            ti: m.ti.clone(),
            yong: m.yong.clone(),
            status: m.status.label().into(),
            formal_allowed: m.formal_allowed,
        }),
        versions: versions.clone(),
        disclaimer: meihua
            .map(|m| m.disclaimer.clone())
            .unwrap_or_else(|| "本工具仅供个人学习与娱乐，不构成任何决策建议。".into()),
        notes: vec![
            "快照不含生辰与问事原文。".into(),
            "摇卦模式可凭 base_seed/随机快照重放；固定模式可由输入与版本重建。".into(),
        ],
    }
}
