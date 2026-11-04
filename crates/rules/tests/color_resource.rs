use std::path::PathBuf;

use rules::verify_manifest;
use serde_json::Value;

fn resources() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../resources")
}

/// SC-010：颜色资源只有九色一张表，不存在其他色制的数据或只读入口。
#[test]
fn color_resource_is_nine_color_only() {
    let root = resources();
    let raw = std::fs::read_to_string(root.join("data/color_num.json")).unwrap();
    let doc: Value = serde_json::from_str(&raw).unwrap();
    let obj = doc.as_object().unwrap();

    let mut keys: Vec<&str> = obj.keys().map(String::as_str).collect();
    keys.sort_unstable();
    assert_eq!(
        keys,
        ["description", "nine_color", "source", "status", "version"],
        "颜色资源只允许九色标准字段"
    );
    assert!(!raw.contains("seven_color"));
    assert!(!raw.contains("READONLY"));

    let colors = obj["nine_color"].as_array().unwrap();
    let expected = [
        "赤色", "橙色", "黄色", "绿色", "青色", "蓝色", "紫色", "黑色", "白色",
    ];
    assert_eq!(colors.len(), expected.len(), "九色表必须恰好 9 条");
    for (index, color) in colors.iter().enumerate() {
        let id = (index + 1) as u64;
        assert_eq!(color["color_id"].as_u64().unwrap(), id);
        assert_eq!(color["name"].as_str().unwrap(), expected[index]);
        assert_eq!(color["xiangshu"].as_u64().unwrap(), id);
    }

    assert_eq!(doc["version"], "COLOR-NUM-02");
    assert_eq!(doc["status"], "VERIFIED");

    let (manifest, _) = verify_manifest(&root).unwrap();
    let entry = manifest.entry("COLOR-NUM-02").unwrap();
    assert_eq!(entry.version, "COLOR-NUM-02");
    assert_eq!(entry.status, "VERIFIED");
}
