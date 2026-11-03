//! 只读资源清单、checksum 校验与规则/出处注册表。

pub mod manifest;
pub mod registry;

pub use manifest::{verify_manifest, Manifest, ManifestEntry, ManifestError, ResourceReport};
pub use registry::{RuleRecord, RuleRegistry, SourceRecord};
