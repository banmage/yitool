use domain::BaziMaterial;
use static_assertions::assert_not_impl_any;

// 编译期门禁：生辰材料不得实现 Serialize/Deserialize。
assert_not_impl_any!(BaziMaterial: serde::Serialize);
assert_not_impl_any!(BaziMaterial: serde::de::DeserializeOwned);

#[test]
fn debug_never_leaks_values() {
    let b = BaziMaterial::new(1990, 1, 1, 12, 0);
    let text = format!("{b:?}");
    assert_eq!(text, "BaziMaterial(<redacted>)");
}

#[test]
fn digest_is_not_empty() {
    let b = BaziMaterial::new(1990, 1, 1, 12, 0);
    assert_ne!(b.digest(), [0u8; 32]);
}
