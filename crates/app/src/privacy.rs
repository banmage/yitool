use std::path::{Path, PathBuf};

use serde::Serialize;

#[derive(Debug, Clone)]
pub struct Needle {
    pub kind: String,
    pub value: String,
}

impl Needle {
    pub fn new(kind: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            kind: kind.into(),
            value: value.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PrivacyHit {
    pub file: String,
    pub offset: usize,
    pub kind: String,
}

/// 递归扫描文件/目录中的敏感值。SQLite 主库与 WAL 按原始字节扫描。
pub fn scan_paths(paths: &[PathBuf], needles: &[Needle]) -> std::io::Result<Vec<PrivacyHit>> {
    let mut files = Vec::new();
    for path in paths {
        collect_files(path, &mut files)?;
    }
    let mut hits = Vec::new();
    for file in files {
        let bytes = match std::fs::read(&file) {
            Ok(b) => b,
            Err(_) => continue,
        };
        for needle in needles {
            if needle.value.is_empty() {
                continue;
            }
            let raw = needle.value.as_bytes();
            let mut start = 0usize;
            while start + raw.len() <= bytes.len() {
                if &bytes[start..start + raw.len()] == raw {
                    hits.push(PrivacyHit {
                        file: file.display().to_string(),
                        offset: start,
                        kind: needle.kind.clone(),
                    });
                    start += raw.len();
                } else {
                    start += 1;
                }
            }
        }
    }
    Ok(hits)
}

fn collect_files(path: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if path.is_file() {
        out.push(path.to_path_buf());
        return Ok(());
    }
    if !path.exists() {
        return Ok(());
    }
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let p = entry.path();
        if p.is_dir() {
            collect_files(&p, out)?;
        } else if p.is_file() {
            out.push(p);
        }
    }
    Ok(())
}
