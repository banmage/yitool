use std::path::PathBuf;

use domain::{trigram_key, Trigram};

#[test]
fn domain_directions_match_g01_resource() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../resources/data/bagua.json");
    let doc: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    assert_eq!(doc["version"], "G-01-v0.2");

    let rows = doc["trigrams"].as_array().unwrap();
    assert_eq!(rows.len(), 8);
    for row in rows {
        let key = row["key"].as_str().unwrap();
        let trigram = Trigram::ALL
            .into_iter()
            .find(|t| trigram_key(*t) == key)
            .unwrap_or_else(|| panic!("未知卦键 {key}"));
        assert_eq!(row["direction"], trigram.direction(), "{key} 方位不一致");
        assert_eq!(row["element"], trigram.element().name(), "{key} 五行不一致");
    }
}
