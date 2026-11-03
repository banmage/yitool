use std::path::PathBuf;

use rules::{verify_manifest, ManifestError, RuleRegistry};

fn resources() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../resources")
}

#[test]
fn real_manifest_verifies() {
    let (manifest, reports) = verify_manifest(resources()).unwrap();
    assert_eq!(manifest.manifest_id, "MANIFEST-data");
    assert!(!reports.is_empty());
    for report in &reports {
        assert!(
            report.checksum_ok,
            "资源 {} ({}) checksum 失败: {:?}",
            report.id, report.path, report.error
        );
    }
}

#[test]
fn registry_loads_and_is_traceable() {
    let registry = RuleRegistry::load(resources().join("data/rules.json")).unwrap();
    assert!(registry.rule("MEIHUA-QIGUA-001").is_some());
    assert!(registry.assert_traceable("MEIHUA-QIGUA-001", "SRC-MEIHUA-001"));
    assert_eq!(registry.rules_for_engine("CAL").len(), 4);
}

#[test]
fn tampered_file_fails_checksum() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::create_dir_all(root.join("manifests")).unwrap();
    std::fs::create_dir_all(root.join("data")).unwrap();
    std::fs::write(root.join("data/x.json"), b"hello").unwrap();
    let sha = "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824";
    let manifest = format!(
        r#"{{"manifest_id":"M","version":"DATA-v0.2","entries":[{{"id":"X","path":"data/x.json","version":"v1","status":"VERIFIED","source":"test","sha256":"{sha}"}}]}}"#
    );
    std::fs::write(root.join("manifests/data.manifest.json"), manifest).unwrap();

    let (_, reports) = verify_manifest(root).unwrap();
    assert!(reports[0].checksum_ok);

    std::fs::write(root.join("data/x.json"), b"tampered").unwrap();
    let (_, reports) = verify_manifest(root).unwrap();
    assert!(!reports[0].checksum_ok);
}

#[test]
fn path_traversal_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::create_dir_all(root.join("manifests")).unwrap();
    let manifest = r#"{"manifest_id":"M","version":"DATA-v0.2","entries":[{"id":"X","path":"../secret","version":"v1","status":"VERIFIED","source":"test","sha256":"00"}]}"#;
    std::fs::write(root.join("manifests/data.manifest.json"), manifest).unwrap();
    let err = verify_manifest(root).unwrap_err();
    assert!(matches!(err, ManifestError::UnsafePath(_)));
}

#[test]
fn missing_file_is_reported() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::create_dir_all(root.join("manifests")).unwrap();
    let manifest = r#"{"manifest_id":"M","version":"DATA-v0.2","entries":[{"id":"X","path":"data/none.json","version":"v1","status":"VERIFIED","source":"test","sha256":"00"}]}"#;
    std::fs::write(root.join("manifests/data.manifest.json"), manifest).unwrap();
    let (_, reports) = verify_manifest(root).unwrap();
    assert!(!reports[0].checksum_ok);
    assert!(reports[0].error.is_some());
}
