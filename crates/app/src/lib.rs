//! 应用服务编排：资源加载、CAL 查询、梅花推演、种子链、隐私扫描与快照导出。

pub mod context;
pub mod evidence;
pub mod export;
pub mod logging;
pub mod privacy;
pub mod report;

pub use context::{resources_dir, AppContext, AppError};
pub use evidence::{EvidenceEntry, EvidenceError, EvidenceIndex, EvidenceKind};
pub use export::{export_snapshot, ExportSnapshot};
pub use logging::sanitize_event;
pub use privacy::{scan_paths, Needle, PrivacyHit};
pub use report::{render_markdown, render_meihua_report, write_markdown, ReportError, ReportMeta};
