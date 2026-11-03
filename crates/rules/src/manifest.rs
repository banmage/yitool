use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ManifestError {
    #[error("清单不可读: {0}")]
    Read(String),
    #[error("清单格式错误: {0}")]
    Parse(String),
    #[error("资源路径非法（绝对路径或包含 ..）: {0}")]
    UnsafePath(String),
    #[error("资源缺失: {0}")]
    Missing(String),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ManifestEntry {
    pub id: String,
    pub path: String,
    pub version: String,
    pub status: String,
    pub source: String,
    pub sha256: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Manifest {
    pub manifest_id: String,
    pub version: String,
    pub entries: Vec<ManifestEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResourceReport {
    pub id: String,
    pub path: String,
    pub version: String,
    pub status: String,
    pub checksum_ok: bool,
    pub actual_sha256: Option<String>,
    pub error: Option<String>,
}

pub fn verify_manifest(
    resources_root: impl AsRef<Path>,
) -> Result<(Manifest, Vec<ResourceReport>), ManifestError> {
    let root = resources_root.as_ref();
    let manifest_path = root.join("manifests/data.manifest.json");
    let text = std::fs::read_to_string(&manifest_path)
        .map_err(|e| ManifestError::Read(format!("{}: {e}", manifest_path.display())))?;
    let manifest: Manifest =
        serde_json::from_str(&text).map_err(|e| ManifestError::Parse(e.to_string()))?;

    let mut reports = Vec::with_capacity(manifest.entries.len());
    for entry in &manifest.entries {
        let rel = Path::new(&entry.path);
        if rel.is_absolute()
            || rel.components().any(|c| {
                matches!(
                    c,
                    Component::ParentDir | Component::RootDir | Component::Prefix(_)
                )
            })
        {
            return Err(ManifestError::UnsafePath(entry.path.clone()));
        }
        let path: PathBuf = root.join(rel);
        match std::fs::read(&path) {
            Ok(bytes) => {
                let actual = hex::encode(Sha256::digest(&bytes));
                reports.push(ResourceReport {
                    id: entry.id.clone(),
                    path: entry.path.clone(),
                    version: entry.version.clone(),
                    status: entry.status.clone(),
                    checksum_ok: actual == entry.sha256,
                    actual_sha256: Some(actual),
                    error: None,
                });
            }
            Err(e) => reports.push(ResourceReport {
                id: entry.id.clone(),
                path: entry.path.clone(),
                version: entry.version.clone(),
                status: entry.status.clone(),
                checksum_ok: false,
                actual_sha256: None,
                error: Some(format!("{e}")),
            }),
        }
    }
    Ok((manifest, reports))
}

impl Manifest {
    pub fn entry(&self, id: &str) -> Option<&ManifestEntry> {
        self.entries.iter().find(|e| e.id == id)
    }
}
