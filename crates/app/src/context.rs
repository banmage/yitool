use std::path::{Path, PathBuf};

use cal::{CalData, CalQuery, CalResult};
use domain::VersionSet;
use ir::Compliance;
use meihua::{MeihuaInput, MeihuaResult};
use rules::RuleRegistry;
use seed::{SeedChain, SeedInput};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("资源目录不可用: {0}")]
    Resources(String),
    #[error("资源完整性校验失败: {0}")]
    ResourcesIntegrity(String),
    #[error("历法错误: {0}")]
    Cal(#[from] cal::CalError),
    #[error("规则/资源错误: {0}")]
    Rules(#[from] rules::manifest::ManifestError),
    #[error("规则注册表错误: {0}")]
    Registry(String),
    #[error("合规词表错误: {0}")]
    Compliance(String),
    #[error("梅花错误: {0}")]
    Meihua(#[from] meihua::MeihuaError),
    #[error("种子错误: {0}")]
    Seed(#[from] seed::SeedError),
    #[error("存储错误: {0}")]
    Store(String),
    #[error("IO 错误: {0}")]
    Io(String),
}

pub fn resources_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("YITOOL_RESOURCES") {
        return PathBuf::from(dir);
    }
    let cwd = PathBuf::from("resources");
    if cwd.exists() {
        return cwd;
    }
    // 从 crate 目录运行的测试/工具回退到仓库根。
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../resources")
        .canonicalize()
        .unwrap_or(cwd)
}

pub struct AppContext {
    pub resources: PathBuf,
    pub cal: CalData,
    pub rules: RuleRegistry,
    pub compliance: Compliance,
}

impl AppContext {
    pub fn load(resources: impl AsRef<Path>) -> Result<Self, AppError> {
        let resources = resources.as_ref().to_path_buf();
        // ADR-012：启动时先校验资源 manifest/checksum，损坏资源拒绝加载。
        let (_, reports) = rules::verify_manifest(&resources)?;
        let failed: Vec<_> = reports.iter().filter(|r| !r.checksum_ok).collect();
        if !failed.is_empty() {
            let ids = failed
                .iter()
                .map(|r| r.id.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            return Err(AppError::ResourcesIntegrity(format!(
                "{} 个资源校验失败: {ids}",
                failed.len()
            )));
        }
        let cal =
            CalData::load(resources.join("cal")).map_err(|e| AppError::Resources(e.to_string()))?;
        let rules = RuleRegistry::load(resources.join("data/rules.json"))
            .map_err(|e| AppError::Registry(e.to_string()))?;
        let compliance = Compliance::load(resources.join("ir/compliance/forbidden_phrases.json"))
            .map_err(|e| AppError::Compliance(e.to_string()))?;
        Ok(Self {
            resources,
            cal,
            rules,
            compliance,
        })
    }

    pub fn versions(&self) -> VersionSet {
        VersionSet::default().with_cal_data(self.cal.version.clone())
    }

    pub fn cal_query(&self, query: &CalQuery) -> Result<CalResult, AppError> {
        Ok(cal::query(&self.cal, query)?)
    }

    pub fn meihua(&self, input: MeihuaInput, query: &CalQuery) -> Result<MeihuaResult, AppError> {
        let cal_result = self.cal_query(query)?;
        let branch_id = cal_result.jieqi_month_branch.branch_id;
        let mut result = meihua::cast(input, branch_id)?;
        // 统一附上合规免责声明；规则未冻结时保留 PENDING。
        result.disclaimer = self.compliance.disclaimer.clone();
        Ok(result)
    }

    pub fn seed(&self, input: &SeedInput) -> Result<SeedChain, AppError> {
        Ok(seed::generate(input)?)
    }
}
