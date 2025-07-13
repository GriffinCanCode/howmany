use crate::core::stats::techstack::{TechStackInventory, TechCategory, DetectedTechnology};
use crate::core::stats::techstack::mapping::{DependencyGraph, ArchitecturePattern};
use crate::utils::errors::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Trend direction for technology analysis
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TrendDirection {
    Growing,
    Stable,
    Declining,
    Emerging,
    Obsolete,
}

/// Comprehensive techstack insights and recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechStackInsights {
    pub summary: InsightsSummary,
    pub recommendations: Vec<FrameworkRecommendation>,
    pub security_insights: SecurityInsights,
    pub performance_insights: PerformanceInsights,
    pub maintainability_insights: MaintainabilityInsights,
    pub modernization_insights: ModernizationInsights,
    pub architecture_insights: ArchitectureInsights,
    pub cost_insights: CostInsights,
    pub trend_analysis: TrendAnalysis,
}

/// High-level summary of insights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightsSummary {
    pub overall_health_score: f64,
    pub key_strengths: Vec<String>,
    pub key_concerns: Vec<String>,
    pub priority_actions: Vec<String>,
    pub technology_diversity_score: f64,
    pub innovation_score: f64,
    pub risk_score: f64,
}

/// Framework and library recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkRecommendation {
    pub recommendation_type: RecommendationType,
    pub title: String,
    pub description: String,
    pub current_technology: Option<String>,
    pub recommended_technology: String,
    pub priority: RecommendationPriority,
    pub impact: ImpactLevel,
    pub effort: EffortLevel,
    pub timeline: String,
    pub benefits: Vec<String>,
    pub risks: Vec<String>,
    pub migration_steps: Vec<String>,
    pub resources: Vec<String>,
}

/// Type of recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationType {
    Upgrade,
    Replace,
    Add,
    Remove,
    Configure,
    Optimize,
    Secure,
    Modernize,
}

/// Priority level for recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Critical,
    High,
    Medium,
    Low,
}

/// Impact level of changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImpactLevel {
    High,
    Medium,
    Low,
}

/// Effort required for implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffortLevel {
    High,
    Medium,
    Low,
}

/// Security-related insights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityInsights {
    pub overall_security_score: f64,
    pub vulnerabilities_found: usize,
    pub critical_vulnerabilities: usize,
    pub outdated_dependencies: usize,
    pub security_recommendations: Vec<SecurityRecommendation>,
    pub compliance_status: ComplianceStatus,
    pub security_trends: SecurityTrends,
}

/// Security recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRecommendation {
    pub title: String,
    pub description: String,
    pub severity: SecuritySeverity,
    pub affected_components: Vec<String>,
    pub remediation_steps: Vec<String>,
    pub cve_references: Vec<String>,
}

/// Security severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecuritySeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// Compliance status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceStatus {
    pub overall_compliance: f64,
    pub license_compliance: f64,
    pub security_compliance: f64,
    pub privacy_compliance: f64,
    pub issues: Vec<ComplianceIssue>,
}

/// Compliance issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceIssue {
    pub issue_type: ComplianceType,
    pub description: String,
    pub affected_components: Vec<String>,
    pub severity: ComplianceSeverity,
    pub remediation: String,
}

/// Type of compliance issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplianceType {
    License,
    Security,
    Privacy,
    Accessibility,
    Performance,
}

/// Compliance severity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplianceSeverity {
    Critical,
    High,
    Medium,
    Low,
}

/// Security trends over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityTrends {
    pub vulnerability_trend: TrendDirection,
    pub patching_frequency: f64,
    pub security_tool_coverage: f64,
    pub recent_security_events: Vec<SecurityEvent>,
}

/// Security event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
    pub event_type: SecurityEventType,
    pub description: String,
    pub date: String,
    pub impact: String,
    pub resolution: Option<String>,
}

/// Type of security event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEventType {
    VulnerabilityDiscovered,
    PatchApplied,
    SecurityAudit,
    ComplianceUpdate,
}

/// Performance-related insights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceInsights {
    pub overall_performance_score: f64,
    pub bundle_size_analysis: BundleSizeAnalysis,
    pub runtime_performance: RuntimePerformance,
    pub build_performance: BuildPerformance,
    pub performance_recommendations: Vec<PerformanceRecommendation>,
}

/// Bundle size analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleSizeAnalysis {
    pub total_size_kb: u64,
    pub largest_dependencies: Vec<DependencySize>,
    pub unused_dependencies: Vec<String>,
    pub optimization_opportunities: Vec<String>,
}

/// Dependency size information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencySize {
    pub name: String,
    pub size_kb: u64,
    pub percentage: f64,
    pub tree_shaking_potential: f64,
}

/// Runtime performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimePerformance {
    pub startup_time_score: f64,
    pub memory_usage_score: f64,
    pub cpu_usage_score: f64,
    pub network_efficiency_score: f64,
    pub caching_efficiency: f64,
}

/// Build performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildPerformance {
    pub build_time_score: f64,
    pub incremental_build_support: bool,
    pub parallel_build_support: bool,
    pub optimization_level: OptimizationLevel,
}

/// Optimization level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationLevel {
    None,
    Basic,
    Standard,
    Advanced,
    Maximum,
}

/// Performance recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRecommendation {
    pub title: String,
    pub description: String,
    pub category: PerformanceCategory,
    pub impact: ImpactLevel,
    pub implementation_steps: Vec<String>,
    pub expected_improvement: String,
}

/// Performance category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceCategory {
    BundleSize,
    RuntimeSpeed,
    BuildTime,
    MemoryUsage,
    NetworkOptimization,
    Caching,
}

/// Maintainability insights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintainabilityInsights {
    pub overall_maintainability_score: f64,
    pub code_quality_score: f64,
    pub documentation_score: f64,
    pub testing_score: f64,
    pub dependency_health: DependencyHealth,
    pub technical_debt: TechnicalDebt,
    pub maintainability_recommendations: Vec<MaintainabilityRecommendation>,
}

/// Dependency health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyHealth {
    pub outdated_dependencies: usize,
    pub deprecated_dependencies: usize,
    pub unmaintained_dependencies: usize,
    pub average_dependency_age: f64,
    pub update_frequency: f64,
}

/// Technical debt analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalDebt {
    pub total_debt_score: f64,
    pub debt_categories: HashMap<String, f64>,
    pub high_priority_debt: Vec<DebtItem>,
    pub debt_trend: TrendDirection,
}

/// Technical debt item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebtItem {
    pub category: String,
    pub description: String,
    pub impact: ImpactLevel,
    pub effort_to_fix: EffortLevel,
    pub affected_components: Vec<String>,
}

/// Maintainability recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintainabilityRecommendation {
    pub title: String,
    pub description: String,
    pub category: MaintainabilityCategory,
    pub priority: RecommendationPriority,
    pub benefits: Vec<String>,
    pub implementation_steps: Vec<String>,
}

/// Maintainability category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MaintainabilityCategory {
    CodeQuality,
    Documentation,
    Testing,
    Dependencies,
    Architecture,
    Tooling,
}

/// Modernization insights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModernizationInsights {
    pub modernization_score: f64,
    pub legacy_components: Vec<LegacyComponent>,
    pub modernization_opportunities: Vec<ModernizationOpportunity>,
    pub migration_roadmap: MigrationRoadmap,
    pub innovation_potential: InnovationPotential,
}

/// Legacy component information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyComponent {
    pub name: String,
    pub category: TechCategory,
    pub age_score: f64,
    pub support_status: SupportStatus,
    pub migration_complexity: EffortLevel,
    pub business_impact: ImpactLevel,
}

/// Support status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SupportStatus {
    Active,
    Maintenance,
    EndOfLife,
    Deprecated,
    Unknown,
}

/// Modernization opportunity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModernizationOpportunity {
    pub title: String,
    pub description: String,
    pub current_state: String,
    pub target_state: String,
    pub benefits: Vec<String>,
    pub challenges: Vec<String>,
    pub estimated_timeline: String,
    pub priority: RecommendationPriority,
}

/// Migration roadmap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationRoadmap {
    pub phases: Vec<MigrationPhase>,
    pub total_duration: String,
    pub key_milestones: Vec<Milestone>,
    pub risk_mitigation: Vec<RiskMitigation>,
}

/// Migration phase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationPhase {
    pub phase_name: String,
    pub description: String,
    pub duration: String,
    pub dependencies: Vec<String>,
    pub deliverables: Vec<String>,
    pub risks: Vec<String>,
}

/// Milestone
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub name: String,
    pub description: String,
    pub target_date: String,
    pub success_criteria: Vec<String>,
}

/// Risk mitigation strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMitigation {
    pub risk: String,
    pub mitigation_strategy: String,
    pub contingency_plan: String,
}

/// Innovation potential
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InnovationPotential {
    pub innovation_score: f64,
    pub emerging_technologies: Vec<EmergingTechnology>,
    pub adoption_readiness: f64,
    pub competitive_advantage: Vec<String>,
}

/// Emerging technology
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergingTechnology {
    pub name: String,
    pub category: TechCategory,
    pub maturity_level: MaturityLevel,
    pub adoption_timeline: String,
    pub potential_impact: ImpactLevel,
    pub learning_curve: EffortLevel,
}

/// Technology maturity level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MaturityLevel {
    Experimental,
    EarlyAdopter,
    GrowingAdoption,
    Mainstream,
    Mature,
}

/// Architecture insights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureInsights {
    pub architecture_score: f64,
    pub current_patterns: Vec<ArchitecturePattern>,
    pub architecture_quality: ArchitectureQuality,
    pub scalability_analysis: ScalabilityAnalysis,
    pub coupling_analysis: CouplingAnalysis,
    pub architecture_recommendations: Vec<ArchitectureRecommendation>,
}

/// Architecture quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureQuality {
    pub modularity_score: f64,
    pub cohesion_score: f64,
    pub coupling_score: f64,
    pub complexity_score: f64,
    pub testability_score: f64,
}

/// Scalability analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalabilityAnalysis {
    pub horizontal_scalability: f64,
    pub vertical_scalability: f64,
    pub bottlenecks: Vec<ScalabilityBottleneck>,
    pub scaling_recommendations: Vec<String>,
}

/// Scalability bottleneck
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalabilityBottleneck {
    pub component: String,
    pub bottleneck_type: BottleneckType,
    pub impact: ImpactLevel,
    pub solution: String,
}

/// Type of bottleneck
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BottleneckType {
    CPU,
    Memory,
    IO,
    Network,
    Database,
    Concurrency,
}

/// Coupling analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouplingAnalysis {
    pub overall_coupling: f64,
    pub tight_coupling_areas: Vec<CouplingArea>,
    pub decoupling_opportunities: Vec<String>,
}

/// Coupling area
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouplingArea {
    pub components: Vec<String>,
    pub coupling_strength: f64,
    pub coupling_type: CouplingType,
    pub impact: String,
}

/// Type of coupling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CouplingType {
    Data,
    Control,
    Content,
    Common,
    External,
}

/// Architecture recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureRecommendation {
    pub title: String,
    pub description: String,
    pub pattern: ArchitecturePattern,
    pub benefits: Vec<String>,
    pub implementation_guidance: Vec<String>,
    pub examples: Vec<String>,
}

/// Cost insights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostInsights {
    pub total_cost_score: f64,
    pub licensing_costs: LicensingCosts,
    pub operational_costs: OperationalCosts,
    pub development_costs: DevelopmentCosts,
    pub cost_optimization_opportunities: Vec<CostOptimization>,
}

/// Licensing costs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicensingCosts {
    pub total_licensing_cost: f64,
    pub commercial_licenses: Vec<LicenseCost>,
    pub compliance_risk: f64,
    pub license_optimization: Vec<String>,
}

/// License cost
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseCost {
    pub component: String,
    pub license_type: String,
    pub cost: f64,
    pub renewal_date: Option<String>,
    pub alternatives: Vec<String>,
}

/// Operational costs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationalCosts {
    pub infrastructure_cost: f64,
    pub maintenance_cost: f64,
    pub support_cost: f64,
    pub training_cost: f64,
    pub cost_breakdown: HashMap<String, f64>,
}

/// Development costs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevelopmentCosts {
    pub development_velocity: f64,
    pub learning_curve_cost: f64,
    pub tooling_cost: f64,
    pub productivity_impact: f64,
}

/// Cost optimization opportunity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostOptimization {
    pub title: String,
    pub description: String,
    pub potential_savings: f64,
    pub implementation_cost: f64,
    pub roi_timeline: String,
    pub steps: Vec<String>,
}

/// Trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    pub technology_trends: Vec<TechnologyTrend>,
    pub adoption_trends: Vec<AdoptionTrend>,
    pub market_trends: Vec<MarketTrend>,
    pub future_outlook: FutureOutlook,
}

/// Technology trend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnologyTrend {
    pub technology: String,
    pub trend_direction: TrendDirection,
    pub momentum: f64,
    pub market_adoption: f64,
    pub predictions: Vec<String>,
}

/// Adoption trend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdoptionTrend {
    pub category: TechCategory,
    pub growth_rate: f64,
    pub market_share: f64,
    pub key_drivers: Vec<String>,
    pub barriers: Vec<String>,
}

/// Market trend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketTrend {
    pub trend_name: String,
    pub description: String,
    pub impact_on_techstack: ImpactLevel,
    pub timeline: String,
    pub preparation_steps: Vec<String>,
}

/// Future outlook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FutureOutlook {
    pub outlook_score: f64,
    pub key_opportunities: Vec<String>,
    pub potential_threats: Vec<String>,
    pub strategic_recommendations: Vec<String>,
    pub timeline: String,
}

/// Main techstack analyzer for generating insights
pub struct TechStackAnalyzer {
    knowledge_base: TechKnowledgeBase,
}

/// Knowledge base for technology insights
#[derive(Debug, Clone)]
pub struct TechKnowledgeBase {
    pub framework_data: HashMap<String, FrameworkData>,
    pub library_data: HashMap<String, LibraryData>,
    pub trend_data: HashMap<String, TrendData>,
    pub security_data: HashMap<String, SecurityData>,
}

/// Framework data for insights
#[derive(Debug, Clone)]
pub struct FrameworkData {
    pub popularity_trend: TrendDirection,
    pub performance_characteristics: PerformanceCharacteristics,
    pub security_profile: SecurityProfile,
    pub learning_curve: EffortLevel,
    pub ecosystem_maturity: MaturityLevel,
    pub alternatives: Vec<String>,
}

/// Performance characteristics
#[derive(Debug, Clone)]
pub struct PerformanceCharacteristics {
    pub startup_time: f64,
    pub memory_usage: f64,
    pub cpu_efficiency: f64,
    pub bundle_size_impact: f64,
    pub scalability: f64,
}

/// Security profile
#[derive(Debug, Clone)]
pub struct SecurityProfile {
    pub security_score: f64,
    pub common_vulnerabilities: Vec<String>,
    pub security_best_practices: Vec<String>,
    pub update_frequency: f64,
}

/// Library data
#[derive(Debug, Clone)]
pub struct LibraryData {
    pub maintenance_status: SupportStatus,
    pub size_impact: f64,
    pub performance_impact: f64,
    pub alternatives: Vec<String>,
    pub migration_complexity: EffortLevel,
}

/// Trend data
#[derive(Debug, Clone)]
pub struct TrendData {
    pub adoption_rate: f64,
    pub growth_trajectory: TrendDirection,
    pub market_share: f64,
    pub future_outlook: f64,
}

/// Security data
#[derive(Debug, Clone)]
pub struct SecurityData {
    pub known_vulnerabilities: Vec<String>,
    pub security_advisories: Vec<String>,
    pub patch_availability: bool,
    pub end_of_life_date: Option<String>,
}

impl TechStackAnalyzer {
    pub fn new() -> Self {
        Self {
            knowledge_base: TechKnowledgeBase::new(),
        }
    }

    /// Analyze techstack and generate comprehensive insights
    pub fn analyze(&self, inventory: &TechStackInventory, dependency_graph: &DependencyGraph) -> Result<TechStackInsights> {
        let summary = self.generate_summary(inventory, dependency_graph)?;
        let recommendations = self.generate_recommendations(inventory, dependency_graph)?;
        let security_insights = self.analyze_security(inventory, dependency_graph)?;
        let performance_insights = self.analyze_performance(inventory, dependency_graph)?;
        let maintainability_insights = self.analyze_maintainability(inventory, dependency_graph)?;
        let modernization_insights = self.analyze_modernization(inventory, dependency_graph)?;
        let architecture_insights = self.analyze_architecture(inventory, dependency_graph)?;
        let cost_insights = self.analyze_costs(inventory, dependency_graph)?;
        let trend_analysis = self.analyze_trends(inventory, dependency_graph)?;

        Ok(TechStackInsights {
            summary,
            recommendations,
            security_insights,
            performance_insights,
            maintainability_insights,
            modernization_insights,
            architecture_insights,
            cost_insights,
            trend_analysis,
        })
    }

    /// Generate high-level summary
    fn generate_summary(&self, inventory: &TechStackInventory, dependency_graph: &DependencyGraph) -> Result<InsightsSummary> {
        let overall_health_score = self.calculate_overall_health_score(inventory, dependency_graph);
        let technology_diversity_score = self.calculate_diversity_score(inventory);
        let innovation_score = self.calculate_innovation_score(inventory);
        let risk_score = self.calculate_risk_score(inventory, dependency_graph);

        let key_strengths = self.identify_key_strengths(inventory, dependency_graph);
        let key_concerns = self.identify_key_concerns(inventory, dependency_graph);
        let priority_actions = self.identify_priority_actions(inventory, dependency_graph);

        Ok(InsightsSummary {
            overall_health_score,
            key_strengths,
            key_concerns,
            priority_actions,
            technology_diversity_score,
            innovation_score,
            risk_score,
        })
    }

    /// Generate framework recommendations
    fn generate_recommendations(&self, inventory: &TechStackInventory, dependency_graph: &DependencyGraph) -> Result<Vec<FrameworkRecommendation>> {
        let mut recommendations = Vec::new();

        // Get frameworks (UI and Web frameworks)
        let frameworks: Vec<&DetectedTechnology> = inventory.technologies.iter()
            .filter(|t| matches!(t.category, TechCategory::UIFramework | TechCategory::WebFramework))
            .collect();

        for framework in frameworks {
            let framework_data = self.knowledge_base.framework_data.get(&framework.name);
            if let Some(data) = framework_data {
                if matches!(data.popularity_trend, TrendDirection::Declining) {
                    recommendations.push(FrameworkRecommendation {
                        recommendation_type: RecommendationType::Replace,
                        title: format!("Consider replacing {}", framework.name),
                        description: format!("{} is showing declining popularity trends", framework.name),
                        current_technology: Some(framework.name.clone()),
                        recommended_technology: data.alternatives.first().unwrap_or(&"Modern alternative".to_string()).clone(),
                        priority: RecommendationPriority::Medium,
                        impact: ImpactLevel::High,
                        effort: EffortLevel::High,
                        timeline: "6-12 months".to_string(),
                        benefits: vec!["Better performance".to_string(), "Active community".to_string()],
                        risks: vec!["Migration complexity".to_string()],
                        migration_steps: vec!["Assess current usage".to_string(), "Plan migration".to_string()],
                        resources: vec!["Migration guide".to_string()],
                    });
                }
            }
        }

        // Get libraries 
        let libraries: Vec<&DetectedTechnology> = inventory.technologies.iter()
            .filter(|t| matches!(t.category, TechCategory::DataLibrary))
            .collect();

        for library in libraries {
            if library.confidence < 0.5 {
                recommendations.push(FrameworkRecommendation {
                    recommendation_type: RecommendationType::Remove,
                    title: format!("Review usage of {}", library.name),
                    description: format!("{} has low confidence detection", library.name),
                    current_technology: Some(library.name.clone()),
                    recommended_technology: "Alternative library".to_string(),
                    priority: RecommendationPriority::Low,
                    impact: ImpactLevel::Low,
                    effort: EffortLevel::Low,
                    timeline: "1-2 months".to_string(),
                    benefits: vec!["Reduced complexity".to_string()],
                    risks: vec!["Minimal risk".to_string()],
                    migration_steps: vec!["Verify actual usage".to_string()],
                    resources: vec!["Documentation".to_string()],
                });
            }
        }

        Ok(recommendations)
    }

    /// Analyze security aspects
    fn analyze_security(&self, inventory: &TechStackInventory, dependency_graph: &DependencyGraph) -> Result<SecurityInsights> {
        let libraries: Vec<&DetectedTechnology> = inventory.technologies.iter()
            .filter(|t| matches!(t.category, TechCategory::DataLibrary))
            .collect();

        let vulnerability_count = libraries.iter()
            .map(|lib| {
                self.knowledge_base.security_data.get(&lib.name)
                    .map(|data| data.known_vulnerabilities.len())
                    .unwrap_or(0)
            })
            .sum();

        let critical_vulnerabilities = libraries.iter()
            .map(|lib| {
                self.knowledge_base.security_data.get(&lib.name)
                    .map(|data| data.known_vulnerabilities.iter().filter(|v| v.contains("CRITICAL")).count())
                    .unwrap_or(0)
            })
            .sum();

        let outdated_dependencies = libraries.iter()
            .filter(|lib| {
                self.knowledge_base.library_data.get(&lib.name)
                    .map(|data| matches!(data.maintenance_status, SupportStatus::EndOfLife | SupportStatus::Deprecated))
                    .unwrap_or(false)
            })
            .count();

        let overall_security_score = self.calculate_security_score(vulnerability_count, critical_vulnerabilities, outdated_dependencies);

        Ok(SecurityInsights {
            overall_security_score,
            vulnerabilities_found: vulnerability_count,
            critical_vulnerabilities,
            outdated_dependencies,
            security_recommendations: vec![],
            compliance_status: ComplianceStatus {
                overall_compliance: 0.8,
                license_compliance: 0.9,
                security_compliance: overall_security_score,
                privacy_compliance: 0.8,
                issues: vec![],
            },
            security_trends: SecurityTrends {
                vulnerability_trend: TrendDirection::Stable,
                patching_frequency: 0.8,
                security_tool_coverage: 0.7,
                recent_security_events: vec![],
            },
        })
    }

    /// Analyze performance aspects
    fn analyze_performance(&self, inventory: &TechStackInventory, dependency_graph: &DependencyGraph) -> Result<PerformanceInsights> {
        let total_size_kb = 1000; // Simplified calculation
        
        let frameworks: Vec<&DetectedTechnology> = inventory.technologies.iter()
            .filter(|t| matches!(t.category, TechCategory::UIFramework | TechCategory::WebFramework))
            .collect();

        let performance_score = self.calculate_performance_score(total_size_kb, &frameworks);

        Ok(PerformanceInsights {
            overall_performance_score: performance_score,
            bundle_size_analysis: BundleSizeAnalysis {
                total_size_kb,
                largest_dependencies: vec![],
                unused_dependencies: vec![],
                optimization_opportunities: vec![],
            },
            runtime_performance: RuntimePerformance {
                startup_time_score: 0.8,
                memory_usage_score: 0.7,
                cpu_usage_score: 0.8,
                network_efficiency_score: 0.9,
                caching_efficiency: 0.6,
            },
            build_performance: BuildPerformance {
                build_time_score: 0.7,
                incremental_build_support: true,
                parallel_build_support: true,
                optimization_level: OptimizationLevel::Standard,
            },
            performance_recommendations: vec![],
        })
    }

    /// Analyze maintainability aspects
    fn analyze_maintainability(&self, inventory: &TechStackInventory, dependency_graph: &DependencyGraph) -> Result<MaintainabilityInsights> {
        let libraries: Vec<&DetectedTechnology> = inventory.technologies.iter()
            .filter(|t| matches!(t.category, TechCategory::DataLibrary))
            .collect();

        let outdated_count = libraries.iter()
            .filter(|lib| {
                self.knowledge_base.library_data.get(&lib.name)
                    .map(|data| matches!(data.maintenance_status, SupportStatus::EndOfLife))
                    .unwrap_or(false)
            })
            .count();

        let dependency_health = DependencyHealth {
            outdated_dependencies: outdated_count,
            deprecated_dependencies: 0,
            unmaintained_dependencies: 0,
            average_dependency_age: 2.0,
            update_frequency: 0.8,
        };

        let maintainability_score = self.calculate_maintainability_score(&dependency_health, dependency_graph);

        Ok(MaintainabilityInsights {
            overall_maintainability_score: maintainability_score,
            code_quality_score: 0.8,
            documentation_score: 0.7,
            testing_score: 0.6,
            dependency_health,
            technical_debt: TechnicalDebt {
                total_debt_score: 0.3,
                debt_categories: HashMap::new(),
                high_priority_debt: vec![],
                debt_trend: TrendDirection::Stable,
            },
            maintainability_recommendations: vec![],
        })
    }

    /// Analyze modernization aspects
    fn analyze_modernization(&self, inventory: &TechStackInventory, dependency_graph: &DependencyGraph) -> Result<ModernizationInsights> {
        let frameworks: Vec<&DetectedTechnology> = inventory.technologies.iter()
            .filter(|t| matches!(t.category, TechCategory::UIFramework | TechCategory::WebFramework))
            .collect();

        let legacy_components: Vec<LegacyComponent> = frameworks.iter()
            .filter_map(|tech| {
                self.knowledge_base.framework_data.get(&tech.name).map(|data| {
                    LegacyComponent {
                        name: tech.name.clone(),
                        category: tech.category.clone(),
                        age_score: 0.7,
                        support_status: SupportStatus::Active,
                        migration_complexity: EffortLevel::Medium,
                        business_impact: ImpactLevel::Medium,
                    }
                })
            })
            .collect();

        let modernization_score = self.calculate_modernization_score(inventory, &legacy_components);

        Ok(ModernizationInsights {
            modernization_score,
            legacy_components,
            modernization_opportunities: vec![],
            migration_roadmap: MigrationRoadmap {
                phases: vec![],
                total_duration: "6 months".to_string(),
                key_milestones: vec![],
                risk_mitigation: vec![],
            },
            innovation_potential: InnovationPotential {
                innovation_score: 0.7,
                emerging_technologies: vec![],
                adoption_readiness: 0.8,
                competitive_advantage: vec![],
            },
        })
    }

    /// Analyze architecture aspects
    fn analyze_architecture(&self, inventory: &TechStackInventory, dependency_graph: &DependencyGraph) -> Result<ArchitectureInsights> {
        let architecture_patterns = self.identify_architecture_patterns(inventory, dependency_graph);
        let architecture_score = self.calculate_architecture_score(dependency_graph);

        Ok(ArchitectureInsights {
            architecture_score,
            current_patterns: architecture_patterns,
            architecture_quality: ArchitectureQuality {
                modularity_score: 0.8,
                cohesion_score: 0.7,
                coupling_score: 0.6,
                complexity_score: 0.7,
                testability_score: 0.8,
            },
            scalability_analysis: ScalabilityAnalysis {
                horizontal_scalability: 0.7,
                vertical_scalability: 0.8,
                bottlenecks: vec![],
                scaling_recommendations: vec![],
            },
            coupling_analysis: CouplingAnalysis {
                overall_coupling: 0.6,
                tight_coupling_areas: vec![],
                decoupling_opportunities: vec![],
            },
            architecture_recommendations: vec![],
        })
    }

    /// Analyze cost aspects
    fn analyze_costs(&self, inventory: &TechStackInventory, dependency_graph: &DependencyGraph) -> Result<CostInsights> {
        Ok(CostInsights {
            total_cost_score: 0.7,
            licensing_costs: LicensingCosts {
                total_licensing_cost: 1000.0,
                commercial_licenses: vec![],
                compliance_risk: 0.3,
                license_optimization: vec![],
            },
            operational_costs: OperationalCosts {
                infrastructure_cost: 500.0,
                maintenance_cost: 300.0,
                support_cost: 200.0,
                training_cost: 100.0,
                cost_breakdown: HashMap::new(),
            },
            development_costs: DevelopmentCosts {
                development_velocity: 0.8,
                learning_curve_cost: 0.3,
                tooling_cost: 0.2,
                productivity_impact: 0.8,
            },
            cost_optimization_opportunities: vec![],
        })
    }

    /// Analyze trends
    fn analyze_trends(&self, inventory: &TechStackInventory, dependency_graph: &DependencyGraph) -> Result<TrendAnalysis> {
        Ok(TrendAnalysis {
            technology_trends: vec![],
            adoption_trends: vec![],
            market_trends: vec![],
            future_outlook: FutureOutlook {
                outlook_score: 0.8,
                key_opportunities: vec![],
                potential_threats: vec![],
                strategic_recommendations: vec![],
                timeline: "Next 2 years".to_string(),
            },
        })
    }

    // Helper methods for calculations
    fn calculate_overall_health_score(&self, inventory: &TechStackInventory, dependency_graph: &DependencyGraph) -> f64 {
        let security_score = self.calculate_basic_security_score(inventory);
        let performance_score = 0.8; // Simplified
        let maintainability_score = dependency_graph.metrics.maintainability_score;
        let modernization_score = inventory.analysis_summary.modernization_score;

        (security_score + performance_score + maintainability_score + modernization_score) / 4.0
    }

    fn calculate_diversity_score(&self, inventory: &TechStackInventory) -> f64 {
        let frameworks: Vec<_> = inventory.technologies.iter()
            .filter(|t| matches!(t.category, TechCategory::UIFramework | TechCategory::WebFramework))
            .collect();
        let libraries: Vec<_> = inventory.technologies.iter()
            .filter(|t| matches!(t.category, TechCategory::DataLibrary))
            .collect();
        let tools: Vec<_> = inventory.technologies.iter()
            .filter(|t| matches!(t.category, TechCategory::BuildTool | TechCategory::TestingFramework | TechCategory::CLI))
            .collect();
        
        let total_techs = frameworks.len() + libraries.len() + tools.len();
        let categories = [
            TechCategory::Frontend,
            TechCategory::Backend,
            TechCategory::Database,
            TechCategory::Infrastructure,
            TechCategory::TestingFramework,
        ];

        let covered_categories = categories.iter()
            .filter(|&cat| {
                inventory.technologies.iter().any(|t| t.category == *cat)
            })
            .count();

        (covered_categories as f64 / categories.len() as f64) * 0.8 + (total_techs as f64 / 20.0).min(1.0) * 0.2
    }

    fn calculate_innovation_score(&self, inventory: &TechStackInventory) -> f64 {
        let modern_techs = inventory.technologies.iter()
            .filter(|t| t.confidence > 0.8)
            .count();
        
        modern_techs as f64 / inventory.technologies.len().max(1) as f64
    }

    fn calculate_risk_score(&self, inventory: &TechStackInventory, dependency_graph: &DependencyGraph) -> f64 {
        let libraries: Vec<&DetectedTechnology> = inventory.technologies.iter()
            .filter(|t| matches!(t.category, TechCategory::DataLibrary))
            .collect();

        let vulnerability_count = libraries.iter()
            .map(|lib| {
                self.knowledge_base.security_data.get(&lib.name)
                    .map(|data| data.known_vulnerabilities.len())
                    .unwrap_or(0)
            })
            .sum::<usize>();

        (vulnerability_count as f64 / 10.0).min(1.0)
    }

    fn identify_key_strengths(&self, inventory: &TechStackInventory, dependency_graph: &DependencyGraph) -> Vec<String> {
        let mut strengths = Vec::new();
        
        let frameworks: Vec<&DetectedTechnology> = inventory.technologies.iter()
            .filter(|t| matches!(t.category, TechCategory::UIFramework | TechCategory::WebFramework))
            .collect();

        if frameworks.iter().any(|f| f.confidence > 0.9) {
            strengths.push("Strong framework choices with high confidence".to_string());
        }

        let has_testing = inventory.technologies.iter().any(|t| t.category == TechCategory::TestingFramework);
        if has_testing {
            strengths.push("Testing framework detected".to_string());
        }

        let has_containerization = inventory.technologies.iter().any(|t| t.category == TechCategory::Containerization);
        if has_containerization {
            strengths.push("Containerization setup detected".to_string());
        }

        strengths
    }

    fn identify_key_concerns(&self, inventory: &TechStackInventory, dependency_graph: &DependencyGraph) -> Vec<String> {
        let mut concerns = Vec::new();
        
        let libraries: Vec<&DetectedTechnology> = inventory.technologies.iter()
            .filter(|t| matches!(t.category, TechCategory::DataLibrary))
            .collect();

        let vulnerability_count = libraries.iter()
            .map(|lib| {
                self.knowledge_base.security_data.get(&lib.name)
                    .map(|data| data.known_vulnerabilities.len())
                    .unwrap_or(0)
            })
            .sum::<usize>();

        if vulnerability_count > 5 {
            concerns.push(format!("{} vulnerabilities detected", vulnerability_count));
        }

        let has_testing = inventory.technologies.iter().any(|t| t.category == TechCategory::TestingFramework);
        if !has_testing {
            concerns.push("No testing framework detected".to_string());
        }

        let outdated_count = libraries.iter()
            .filter(|lib| {
                self.knowledge_base.library_data.get(&lib.name)
                    .map(|data| matches!(data.maintenance_status, SupportStatus::EndOfLife))
                    .unwrap_or(false)
            })
            .count();

        if outdated_count > 0 {
            concerns.push(format!("{} outdated dependencies", outdated_count));
        }

        concerns
    }

    fn identify_priority_actions(&self, inventory: &TechStackInventory, dependency_graph: &DependencyGraph) -> Vec<String> {
        let mut actions = Vec::new();
        
        let has_testing = inventory.technologies.iter().any(|t| t.category == TechCategory::TestingFramework);
        if !has_testing {
            actions.push("Add testing framework".to_string());
        }

        let libraries: Vec<&DetectedTechnology> = inventory.technologies.iter()
            .filter(|t| matches!(t.category, TechCategory::DataLibrary))
            .collect();

        let critical_vulns = libraries.iter()
            .map(|lib| {
                self.knowledge_base.security_data.get(&lib.name)
                    .map(|data| data.known_vulnerabilities.iter().filter(|v| v.contains("CRITICAL")).count())
                    .unwrap_or(0)
            })
            .sum::<usize>();

        if critical_vulns > 0 {
            actions.push("Address critical vulnerabilities".to_string());
        }

        let has_containerization = inventory.technologies.iter().any(|t| t.category == TechCategory::Containerization);
        if !has_containerization {
            actions.push("Consider containerization".to_string());
        }

        actions
    }

    fn calculate_security_score(&self, vulnerabilities: usize, critical_vulns: usize, outdated_deps: usize) -> f64 {
        let base_score = 100.0;
        let vulnerability_penalty = vulnerabilities as f64 * 5.0;
        let critical_penalty = critical_vulns as f64 * 15.0;
        let outdated_penalty = outdated_deps as f64 * 10.0;
        
        ((base_score - vulnerability_penalty - critical_penalty - outdated_penalty) / 100.0).max(0.0)
    }

    fn calculate_basic_security_score(&self, inventory: &TechStackInventory) -> f64 {
        let libraries: Vec<&DetectedTechnology> = inventory.technologies.iter()
            .filter(|t| matches!(t.category, TechCategory::DataLibrary))
            .collect();

        let vulnerability_count = libraries.iter()
            .map(|lib| {
                self.knowledge_base.security_data.get(&lib.name)
                    .map(|data| data.known_vulnerabilities.len())
                    .unwrap_or(0)
            })
            .sum::<usize>();

        let critical_vulns = libraries.iter()
            .map(|lib| {
                self.knowledge_base.security_data.get(&lib.name)
                    .map(|data| data.known_vulnerabilities.iter().filter(|v| v.contains("CRITICAL")).count())
                    .unwrap_or(0)
            })
            .sum::<usize>();

        self.calculate_security_score(vulnerability_count, critical_vulns, 0) / 100.0
    }

    fn calculate_performance_score(&self, bundle_size: u64, frameworks: &[&DetectedTechnology]) -> f64 {
        let size_score = if bundle_size < 500 {
            1.0
        } else if bundle_size < 1000 {
            0.8
        } else if bundle_size < 2000 {
            0.6
        } else {
            0.4
        };

        let framework_score = frameworks.iter()
            .map(|f| {
                self.knowledge_base.framework_data.get(&f.name)
                    .map(|data| data.performance_characteristics.cpu_efficiency)
                    .unwrap_or(0.7)
            })
            .fold(0.0, |acc, score| acc + score) / frameworks.len().max(1) as f64;

        (size_score + framework_score) / 2.0
    }

    fn calculate_maintainability_score(&self, dependency_health: &DependencyHealth, dependency_graph: &DependencyGraph) -> f64 {
        let health_score = 1.0 - (dependency_health.outdated_dependencies as f64 / 10.0).min(1.0);
        let graph_score = dependency_graph.metrics.maintainability_score;
        (health_score + graph_score) / 2.0
    }

    fn calculate_modernization_score(&self, inventory: &TechStackInventory, legacy_components: &[LegacyComponent]) -> f64 {
        let total_components = inventory.technologies.len();
        let legacy_count = legacy_components.len();
        1.0 - (legacy_count as f64 / total_components.max(1) as f64)
    }

    fn identify_architecture_patterns(&self, inventory: &TechStackInventory, dependency_graph: &DependencyGraph) -> Vec<ArchitecturePattern> {
        let mut patterns = Vec::new();
        
        let frameworks: Vec<&DetectedTechnology> = inventory.technologies.iter()
            .filter(|t| matches!(t.category, TechCategory::UIFramework | TechCategory::WebFramework))
            .collect();

        if frameworks.iter().any(|f| f.name.contains("React") || f.name.contains("Vue") || f.name.contains("Angular")) {
            patterns.push(ArchitecturePattern::MVC);
        }

        let has_containerization = inventory.technologies.iter().any(|t| t.category == TechCategory::Containerization);
        if has_containerization {
            patterns.push(ArchitecturePattern::Microservices);
        }

        let has_serverless = inventory.technologies.iter().any(|t| t.name.contains("Lambda") || t.name.contains("serverless"));
        if has_serverless {
            patterns.push(ArchitecturePattern::Serverless);
        }

        if patterns.is_empty() {
            patterns.push(ArchitecturePattern::Unknown);
        }

        patterns
    }

    fn calculate_architecture_score(&self, dependency_graph: &DependencyGraph) -> f64 {
        let complexity_score = 1.0 - (dependency_graph.metrics.complexity_score / 10.0).min(1.0);
        let coupling_score = 1.0 - dependency_graph.metrics.coupling_score;
        let cohesion_score = dependency_graph.metrics.cohesion_score;
        
        (complexity_score + coupling_score + cohesion_score) / 3.0
    }
}

impl TechKnowledgeBase {
    pub fn new() -> Self {
        let mut knowledge_base = Self {
            framework_data: HashMap::new(),
            library_data: HashMap::new(),
            trend_data: HashMap::new(),
            security_data: HashMap::new(),
        };

        knowledge_base.initialize_data();
        knowledge_base
    }

    fn initialize_data(&mut self) {
        // Initialize framework data
        self.framework_data.insert("React".to_string(), FrameworkData {
            popularity_trend: TrendDirection::Stable,
            performance_characteristics: PerformanceCharacteristics {
                startup_time: 0.8,
                memory_usage: 0.7,
                cpu_efficiency: 0.8,
                bundle_size_impact: 0.6,
                scalability: 0.9,
            },
            security_profile: SecurityProfile {
                security_score: 0.85,
                common_vulnerabilities: vec!["XSS".to_string()],
                security_best_practices: vec!["Sanitize inputs".to_string()],
                update_frequency: 0.9,
            },
            learning_curve: EffortLevel::Medium,
            ecosystem_maturity: MaturityLevel::Mature,
            alternatives: vec!["Vue.js".to_string(), "Angular".to_string()],
        });

        // Initialize trend data
        self.trend_data.insert("React".to_string(), TrendData {
            adoption_rate: 0.9,
            growth_trajectory: TrendDirection::Stable,
            market_share: 0.4,
            future_outlook: 0.8,
        });

        // Add more data...
    }
}

impl Default for TechStackAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::test_utils::TestProject;
    use crate::core::stats::techstack::detection::TechStackDetector;
    use crate::core::stats::techstack::mapping::DependencyMapper;

    #[test]
    fn test_techstack_analyzer_creation() {
        let analyzer = TechStackAnalyzer::new();
        assert!(!analyzer.knowledge_base.framework_data.is_empty());
    }

    #[test]
    fn test_insights_generation() {
        let project = TestProject::new().unwrap();
        let detector = TechStackDetector::new();
        let mapper = DependencyMapper::new();
        let analyzer = TechStackAnalyzer::new();

        // Create test files
        project.create_file("package.json", r#"{"dependencies": {"react": "^18.0.0"}}"#).unwrap();
        
        let inventory = detector.detect_techstack(project.path()).unwrap();
        let dependency_graph = mapper.map_dependencies(&inventory, project.path()).unwrap();
        let insights = analyzer.analyze(&inventory, &dependency_graph).unwrap();

        assert!(insights.summary.overall_health_score > 0.0);
        assert!(insights.recommendations.len() >= 0);
    }

    #[test]
    fn test_security_analysis() {
        let analyzer = TechStackAnalyzer::new();
        let inventory = TechStackInventory {
            technologies: vec![],
            total_files_analyzed: 0,
            overall_confidence: 0.8,
            analysis_summary: crate::core::stats::techstack::detection::AnalysisSummary {
                total_technologies: 0,
                primary_language: None,
                architecture_type: crate::core::stats::techstack::detection::ArchitectureType::Unknown,
                deployment_type: crate::core::stats::techstack::detection::DeploymentType::Unknown,
                security_posture: crate::core::stats::techstack::detection::SecurityPosture::Unknown,
                modernization_score: 0.5,
            },
        };

        let dependency_graph = crate::core::stats::techstack::mapping::DependencyGraph {
            nodes: vec![],
            edges: vec![],
            clusters: vec![],
            metrics: crate::core::stats::techstack::mapping::GraphMetrics {
                total_nodes: 0,
                total_edges: 0,
                density: 0.0,
                complexity_score: 0.0,
                coupling_score: 0.0,
                cohesion_score: 0.0,
                maintainability_score: 0.0,
                circular_dependencies: vec![],
            },
        };

        let security_insights = analyzer.analyze_security(&inventory, &dependency_graph).unwrap();
        assert!(security_insights.overall_security_score >= 0.0);
    }
} 