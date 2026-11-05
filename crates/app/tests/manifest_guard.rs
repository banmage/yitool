use std::path::Path;

use app::{AppContext, AppError};

fn write_manifest(root: &Path, version: &str, sha256: &str) {
    std::fs::create_dir_all(root.join("manifests")).unwrap();
    std::fs::create_dir_all(root.join("data")).unwrap();
    std::fs::write(root.join("data/x.json"), b"trusted").unwrap();
    let manifest = serde_json::json!({
        "manifest_id": "MANIFEST-data",
        "version": version,
        "entries": [{
            "id": "TEST-RESOURCE",
            "path": "data/x.json",
            "version": "TEST-v1",
            "status": "VERIFIED",
            "source": "test",
            "sha256": sha256
        }]
    });
    std::fs::write(
        root.join("manifests/data.manifest.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
}

#[test]
fn app_context_rejects_tampered_resource() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write_manifest(root, "DATA-v0.2", &"0".repeat(64));

    let error = match AppContext::load(root) {
        Ok(_) => panic!("应拒绝被篡改资源，实际加载成功"),
        Err(error) => error,
    };
    match error {
        AppError::ResourcesIntegrity(message) => {
            assert!(message.contains("TEST-RESOURCE"), "{message}");
            assert!(message.contains("1 个资源校验失败"), "{message}");
        }
        other => panic!("应返回资源完整性错误，实际: {other}"),
    }
}

#[test]
fn app_context_rejects_incompatible_manifest_version() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    // data/x.json == b"trusted"
    write_manifest(
        root,
        "DATA-v0.1",
        "a9a089195c68d2adeee23beaa2c3a93b1d4cdf09046e7a9e520b3b166dff3e6a",
    );

    let error = match AppContext::load(root) {
        Ok(_) => panic!("应拒绝不兼容版本的资源清单，实际加载成功"),
        Err(error) => error,
    };
    match error {
        AppError::ResourcesIntegrity(message) => {
            assert!(message.contains("版本不兼容"), "{message}");
            assert!(message.contains("DATA-v0.2"), "{message}");
            assert!(message.contains("DATA-v0.1"), "{message}");
        }
        other => panic!("应返回资源完整性错误，实际: {other}"),
    }
}
