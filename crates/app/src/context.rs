use std::path::{Path, PathBuf};

use cal::{CalData, CalQuery, CalResult};
use domain::VersionSet;
use ir::Compliance;
use meihua::{MeihuaInput, MeihuaResult};
use rules::manifest::{Manifest, ResourceReport};
use rules::RuleRegistry;
use seed::{SeedChain, SeedInput};
use thiserror::Error;

use crate::evidence::{EvidenceError, EvidenceIndex};

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
    #[error("依据链接错误: {0}")]
    Evidence(#[from] EvidenceError),
    #[error("报告生成错误: {0}")]
    Report(#[from] crate::report::ReportError),
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
    pub manifest: Manifest,
    pub resource_reports: Vec<ResourceReport>,
    pub cal: CalData,
    pub rules: RuleRegistry,
    pub compliance: Compliance,
}

impl AppContext {
    pub fn load(resources: impl AsRef<Path>) -> Result<Self, AppError> {
        let resources = resources.as_ref().to_path_buf();
        // ADR-012：启动时先校验资源 manifest/checksum，损坏资源拒绝加载。
        let (manifest, reports) = rules::verify_manifest(&resources)?;
        let expected_data = VersionSet::default().data;
        if manifest.version != expected_data {
            return Err(AppError::ResourcesIntegrity(format!(
                "资源清单版本不兼容: 期望 {expected_data}，实际 {}",
                manifest.version
            )));
        }
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
            manifest,
            resource_reports: reports,
            cal,
            rules,
            compliance,
        })
    }

    pub fn versions(&self) -> VersionSet {
        VersionSet::default().with_cal_data(self.cal.version.clone())
    }

    /// 构建规则、典籍出处与附件资源的统一链接索引。
    pub fn evidence_index(&self) -> Result<EvidenceIndex, AppError> {
        Ok(EvidenceIndex::build(
            &self.manifest,
            &self.resource_reports,
            &self.rules,
        )?)
    }

    /// 渲染一次梅花结果的完整 Markdown 报告。
    pub fn meihua_report(
        &self,
        result: &MeihuaResult,
        run_id: &str,
        created_at: &str,
        mode: &str,
        seed: Option<&SeedChain>,
    ) -> Result<String, AppError> {
        let evidence = self.evidence_index()?;
        Ok(crate::report::render_meihua_report(
            result,
            &self.versions(),
            &evidence,
            run_id,
            created_at,
            mode,
            seed,
        )?)
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
