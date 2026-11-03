//! 日志/导出字段白名单：只允许结构化、非敏感字段进入日志。

use serde_json::Value;

/// 敏感键名（小写匹配）。命中即删除，不做“脱敏后保留”。
pub const FORBIDDEN_KEYS: [&str; 8] = [
    "bazi",
    "bazi_material",
    "birth",
    "birthday",
    "question_text",
    "question_raw",
    "hmac_key",
    "hmac_secret",
];

/// 递归删除敏感键，返回被删除的键名（用于测试/审计，不含值）。
pub fn sanitize_event(value: &mut Value) -> Vec<String> {
    let mut removed = Vec::new();
    sanitize_inner(value, &mut removed);
    removed
}

fn sanitize_inner(value: &mut Value, removed: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            let keys: Vec<String> = map.keys().cloned().collect();
            for key in keys {
                if FORBIDDEN_KEYS.contains(&key.to_ascii_lowercase().as_str()) {
                    map.remove(&key);
                    removed.push(key);
                } else if let Some(child) = map.get_mut(&key) {
                    sanitize_inner(child, removed);
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                sanitize_inner(item, removed);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn removes_sensitive_keys_recursively() {
        let mut event = json!({
            "run_id": "r1",
            "bazi": {"year": 1990},
            "nested": [{"question_text": "原文"}, {"ok": true}]
        });
        let removed = sanitize_event(&mut event);
        assert!(removed.contains(&"bazi".to_string()));
        assert!(removed.contains(&"question_text".to_string()));
        assert!(event.get("bazi").is_none());
        assert_eq!(event["run_id"], "r1");
        assert_eq!(event["nested"][1]["ok"], true);
    }
}
