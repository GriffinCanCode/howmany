use crate::core::stats::techstack::{TechStackInventory, TechCategory, ConfidenceLevel};
use crate::core::stats::techstack::mapping::DependencyGraph;
use crate::utils::errors::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Statistics derived from techstack analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechStackStats {
    pub total_technologies: usize,
    pub technologies_by_category: HashMap<TechCategory, usize>,
    pub confidence_distribution: HashMap<ConfidenceLevel, usize>,
    pub dependency_metrics: DependencyMetrics,
    pub technology_diversity: TechnologyDiversity,
    pub maturity_analysis: MaturityAnalysis,
    pub risk_analysis: RiskAnalysis,
    pub innovation_metrics: InnovationMetrics,
}

/// Metrics about dependencies and their relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyMetrics {
    pub total_dependencies: usize,
    pub direct_dependencies: usize,
    pub transitive_dependencies: usize,
    pub circular_dependencies: usize,
    pub dependency_depth: usize,
    pub coupling_score: f64,
    pub cohesion_score: f64,
    pub maintainability_index: f64,
}

/// Analysis of technology diversity in the stack
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnologyDiversity {
    pub diversity_score: f64,
    pub language_diversity: f64,
    pub framework_diversity: f64,
    pub tool_diversity: f64,
    pub dominant_technologies: Vec<String>,
    pub underrepresented_categories: Vec<TechCategory>,
}

/// Maturity analysis of the technology stack
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaturityAnalysis {
    pub overall_maturity_score: f64,
    pub mature_technologies: usize,
    pub emerging_technologies: usize,
    pub deprecated_technologies: usize,
    pub end_of_life_technologies: usize,
    pub maturity_distribution: HashMap<MaturityLevel, usize>,
}

/// Maturity levels for technologies
#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum MaturityLevel {
    Experimental,
    EarlyAdopter,
    GrowingAdoption,
    Mainstream,
    Mature,
    Declining,
    Deprecated,
}

/// Risk analysis of the technology stack
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAnalysis {
    pub overall_risk_score: f64,
    pub security_risk: f64,
    pub maintenance_risk: f64,
    pub performance_risk: f64,
    pub vendor_lock_in_risk: f64,
    pub obsolescence_risk: f64,
    pub high_risk_technologies: Vec<RiskTechnology>,
}

/// Technology with associated risk information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskTechnology {
    pub name: String,
    pub category: TechCategory,
    pub risk_factors: Vec<String>,
    pub risk_score: f64,
    pub mitigation_strategies: Vec<String>,
}

/// Innovation metrics for the technology stack
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InnovationMetrics {
    pub innovation_score: f64,
    pub cutting_edge_technologies: usize,
    pub stable_technologies: usize,
    pub legacy_technologies: usize,
    pub innovation_potential: f64,
    pub technology_adoption_rate: f64,
    pub future_readiness_score: f64,
}

/// Calculator for techstack statistics
pub struct TechStackStatsCalculator {
    // Configuration for analysis
    risk_thresholds: RiskThresholds,
    maturity_weights: MaturityWeights,
}

/// Risk assessment thresholds
#[derive(Debug, Clone)]
pub struct RiskThresholds {
    pub high_risk_threshold: f64,
    pub medium_risk_threshold: f64,
    pub vulnerability_weight: f64,
    pub maintenance_weight: f64,
    pub performance_weight: f64,
}

/// Weights for maturity scoring
#[derive(Debug, Clone)]
pub struct MaturityWeights {
    pub experimental_weight: f64,
    pub early_adopter_weight: f64,
    pub growing_weight: f64,
    pub mainstream_weight: f64,
    pub mature_weight: f64,
    pub declining_weight: f64,
    pub deprecated_weight: f64,
}

impl TechStackStatsCalculator {
    pub fn new() -> Self {
        Self {
            risk_thresholds: RiskThresholds {
                high_risk_threshold: 0.7,
                medium_risk_threshold: 0.4,
                vulnerability_weight: 0.4,
                maintenance_weight: 0.3,
                performance_weight: 0.3,
            },
            maturity_weights: MaturityWeights {
                experimental_weight: 0.1,
                early_adopter_weight: 0.3,
                growing_weight: 0.6,
                mainstream_weight: 0.9,
                mature_weight: 1.0,
                declining_weight: 0.7,
                deprecated_weight: 0.2,
            },
        }
    }

    /// Calculate comprehensive statistics from techstack analysis
    pub fn calculate_stats(&self, inventory: &TechStackInventory, dependency_graph: &DependencyGraph) -> Result<TechStackStats> {
        let total_technologies = inventory.technologies.len();
        let technologies_by_category = self.calculate_category_distribution(inventory);
        let confidence_distribution = self.calculate_confidence_distribution(inventory);
        let dependency_metrics = self.calculate_dependency_metrics(dependency_graph);
        let technology_diversity = self.calculate_technology_diversity(inventory);
        let maturity_analysis = self.calculate_maturity_analysis(inventory);
        let risk_analysis = self.calculate_risk_analysis(inventory, dependency_graph);
        let innovation_metrics = self.calculate_innovation_metrics(inventory);

        Ok(TechStackStats {
            total_technologies,
            technologies_by_category,
            confidence_distribution,
            dependency_metrics,
            technology_diversity,
            maturity_analysis,
            risk_analysis,
            innovation_metrics,
        })
    }

    /// Calculate distribution of technologies by category
    fn calculate_category_distribution(&self, inventory: &TechStackInventory) -> HashMap<TechCategory, usize> {
        let mut distribution = HashMap::new();
        
        for technology in &inventory.technologies {
            *distribution.entry(technology.category.clone()).or_insert(0) += 1;
        }
        
        distribution
    }

    /// Calculate distribution of confidence levels
    fn calculate_confidence_distribution(&self, inventory: &TechStackInventory) -> HashMap<ConfidenceLevel, usize> {
        let mut distribution = HashMap::new();
        
        for technology in &inventory.technologies {
            let confidence_level = ConfidenceLevel::from_score(technology.confidence);
            *distribution.entry(confidence_level).or_insert(0) += 1;
        }
        
        distribution
    }

    /// Calculate metrics about dependencies
    fn calculate_dependency_metrics(&self, dependency_graph: &DependencyGraph) -> DependencyMetrics {
        let total_dependencies = dependency_graph.nodes.len();
        let direct_dependencies = dependency_graph.edges.iter()
            .filter(|edge| matches!(edge.relationship, crate::core::stats::techstack::mapping::DependencyRelationship::DirectDependency))
            .count();
        let transitive_dependencies = total_dependencies - direct_dependencies;
        let circular_dependencies = dependency_graph.metrics.circular_dependencies.len();
        
        // Calculate dependency depth (maximum path length)
        let dependency_depth = self.calculate_dependency_depth(dependency_graph);
        
        DependencyMetrics {
            total_dependencies,
            direct_dependencies,
            transitive_dependencies,
            circular_dependencies,
            dependency_depth,
            coupling_score: dependency_graph.metrics.coupling_score,
            cohesion_score: dependency_graph.metrics.cohesion_score,
            maintainability_index: dependency_graph.metrics.maintainability_score,
        }
    }

    /// Calculate technology diversity metrics
    fn calculate_technology_diversity(&self, inventory: &TechStackInventory) -> TechnologyDiversity {
        let category_counts = self.calculate_category_distribution(inventory);
        let total_technologies = inventory.technologies.len() as f64;
        
        // Calculate Shannon diversity index
        let diversity_score = if total_technologies > 0.0 {
            let mut shannon_index = 0.0;
            for count in category_counts.values() {
                let proportion = *count as f64 / total_technologies;
                if proportion > 0.0 {
                    shannon_index -= proportion * proportion.ln();
                }
            }
            shannon_index / (category_counts.len() as f64).ln()
        } else {
            0.0
        };

        // Calculate specific diversity metrics
        let language_diversity = self.calculate_category_diversity(inventory, &[TechCategory::ProgrammingLanguage]);
        let framework_diversity = self.calculate_category_diversity(inventory, &[
            TechCategory::WebFramework, TechCategory::UIFramework, TechCategory::TestingFramework
        ]);
        let tool_diversity = self.calculate_category_diversity(inventory, &[
            TechCategory::BuildTool, TechCategory::Linting, TechCategory::Documentation
        ]);

        // Find dominant technologies (>20% of total)
        let dominant_technologies = inventory.technologies.iter()
            .filter(|tech| tech.confidence > 0.8)
            .take(3)
            .map(|tech| tech.name.clone())
            .collect();

        // Find underrepresented categories
        let underrepresented_categories = category_counts.iter()
            .filter(|(_, &count)| count < 2)
            .map(|(category, _)| category.clone())
            .collect();

        TechnologyDiversity {
            diversity_score,
            language_diversity,
            framework_diversity,
            tool_diversity,
            dominant_technologies,
            underrepresented_categories,
        }
    }

    /// Calculate maturity analysis
    fn calculate_maturity_analysis(&self, inventory: &TechStackInventory) -> MaturityAnalysis {
        let mut maturity_distribution = HashMap::new();
        let mut mature_technologies = 0;
        let mut emerging_technologies = 0;
        let mut deprecated_technologies = 0;
        let mut end_of_life_technologies = 0;

        for technology in &inventory.technologies {
            let maturity = self.assess_technology_maturity(technology);
            *maturity_distribution.entry(maturity.clone()).or_insert(0) += 1;

            match maturity {
                MaturityLevel::Mature | MaturityLevel::Mainstream => mature_technologies += 1,
                MaturityLevel::Experimental | MaturityLevel::EarlyAdopter => emerging_technologies += 1,
                MaturityLevel::Deprecated => deprecated_technologies += 1,
                MaturityLevel::Declining => end_of_life_technologies += 1,
                _ => {}
            }
        }

        let overall_maturity_score = self.calculate_overall_maturity_score(&maturity_distribution);

        MaturityAnalysis {
            overall_maturity_score,
            mature_technologies,
            emerging_technologies,
            deprecated_technologies,
            end_of_life_technologies,
            maturity_distribution,
        }
    }

    /// Calculate risk analysis
    fn calculate_risk_analysis(&self, inventory: &TechStackInventory, dependency_graph: &DependencyGraph) -> RiskAnalysis {
        let mut high_risk_technologies = Vec::new();
        let mut total_security_risk = 0.0;
        let mut total_maintenance_risk = 0.0;
        let mut total_performance_risk = 0.0;
        let mut total_vendor_risk = 0.0;
        let mut total_obsolescence_risk = 0.0;

        for technology in &inventory.technologies {
            let risk_score = self.calculate_technology_risk(technology, dependency_graph);
            
            if risk_score > self.risk_thresholds.high_risk_threshold {
                high_risk_technologies.push(RiskTechnology {
                    name: technology.name.clone(),
                    category: technology.category.clone(),
                    risk_factors: self.identify_risk_factors(technology),
                    risk_score,
                    mitigation_strategies: self.suggest_mitigation_strategies(technology),
                });
            }

            // Aggregate risk components
            total_security_risk += self.calculate_security_risk(technology);
            total_maintenance_risk += self.calculate_maintenance_risk(technology);
            total_performance_risk += self.calculate_performance_risk(technology);
            total_vendor_risk += self.calculate_vendor_lock_in_risk(technology);
            total_obsolescence_risk += self.calculate_obsolescence_risk(technology);
        }

        let count = inventory.technologies.len() as f64;
        let overall_risk_score = if count > 0.0 {
            (total_security_risk + total_maintenance_risk + total_performance_risk + 
             total_vendor_risk + total_obsolescence_risk) / (count * 5.0)
        } else {
            0.0
        };

        RiskAnalysis {
            overall_risk_score,
            security_risk: total_security_risk / count.max(1.0),
            maintenance_risk: total_maintenance_risk / count.max(1.0),
            performance_risk: total_performance_risk / count.max(1.0),
            vendor_lock_in_risk: total_vendor_risk / count.max(1.0),
            obsolescence_risk: total_obsolescence_risk / count.max(1.0),
            high_risk_technologies,
        }
    }

    /// Calculate innovation metrics
    fn calculate_innovation_metrics(&self, inventory: &TechStackInventory) -> InnovationMetrics {
        let mut cutting_edge_technologies = 0;
        let mut stable_technologies = 0;
        let mut legacy_technologies = 0;
        let mut total_innovation_score = 0.0;

        for technology in &inventory.technologies {
            let maturity = self.assess_technology_maturity(technology);
            let innovation_score = self.calculate_technology_innovation_score(technology);
            total_innovation_score += innovation_score;

            match maturity {
                MaturityLevel::Experimental | MaturityLevel::EarlyAdopter => cutting_edge_technologies += 1,
                MaturityLevel::Mainstream | MaturityLevel::Mature => stable_technologies += 1,
                MaturityLevel::Declining | MaturityLevel::Deprecated => legacy_technologies += 1,
                _ => {}
            }
        }

        let count = inventory.technologies.len() as f64;
        let innovation_score = total_innovation_score / count.max(1.0);
        let innovation_potential = self.calculate_innovation_potential(inventory);
        let technology_adoption_rate = self.calculate_adoption_rate(inventory);
        let future_readiness_score = self.calculate_future_readiness(inventory);

        InnovationMetrics {
            innovation_score,
            cutting_edge_technologies,
            stable_technologies,
            legacy_technologies,
            innovation_potential,
            technology_adoption_rate,
            future_readiness_score,
        }
    }

    // Helper methods for calculations

    fn calculate_dependency_depth(&self, dependency_graph: &DependencyGraph) -> usize {
        // Simplified calculation - in a real implementation, this would use graph traversal
        let avg_connections = if dependency_graph.nodes.is_empty() {
            0.0
        } else {
            dependency_graph.edges.len() as f64 / dependency_graph.nodes.len() as f64
        };
        (avg_connections * 2.0) as usize
    }

    fn calculate_category_diversity(&self, inventory: &TechStackInventory, categories: &[TechCategory]) -> f64 {
        let category_count = inventory.technologies.iter()
            .filter(|tech| categories.contains(&tech.category))
            .count();
        
        if category_count == 0 {
            0.0
        } else {
            category_count as f64 / inventory.technologies.len() as f64
        }
    }

    fn assess_technology_maturity(&self, technology: &crate::core::stats::techstack::DetectedTechnology) -> MaturityLevel {
        // Simplified maturity assessment based on technology characteristics
        match technology.category {
            TechCategory::ProgrammingLanguage => {
                if technology.name.contains("Rust") || technology.name.contains("Go") {
                    MaturityLevel::GrowingAdoption
                } else if technology.name.contains("JavaScript") || technology.name.contains("Python") {
                    MaturityLevel::Mature
                } else {
                    MaturityLevel::Mainstream
                }
            }
            TechCategory::WebFramework => {
                if technology.name.contains("React") || technology.name.contains("Vue") {
                    MaturityLevel::Mature
                } else if technology.name.contains("Svelte") {
                    MaturityLevel::GrowingAdoption
                } else {
                    MaturityLevel::Mainstream
                }
            }
            _ => MaturityLevel::Mainstream,
        }
    }

    fn calculate_overall_maturity_score(&self, distribution: &HashMap<MaturityLevel, usize>) -> f64 {
        let total_count: usize = distribution.values().sum();
        if total_count == 0 {
            return 0.0;
        }

        let mut weighted_score = 0.0;
        for (maturity, count) in distribution {
            let weight = match maturity {
                MaturityLevel::Experimental => self.maturity_weights.experimental_weight,
                MaturityLevel::EarlyAdopter => self.maturity_weights.early_adopter_weight,
                MaturityLevel::GrowingAdoption => self.maturity_weights.growing_weight,
                MaturityLevel::Mainstream => self.maturity_weights.mainstream_weight,
                MaturityLevel::Mature => self.maturity_weights.mature_weight,
                MaturityLevel::Declining => self.maturity_weights.declining_weight,
                MaturityLevel::Deprecated => self.maturity_weights.deprecated_weight,
            };
            weighted_score += weight * (*count as f64);
        }

        weighted_score / total_count as f64
    }

    fn calculate_technology_risk(&self, technology: &crate::core::stats::techstack::DetectedTechnology, _dependency_graph: &DependencyGraph) -> f64 {
        let security_risk = self.calculate_security_risk(technology);
        let maintenance_risk = self.calculate_maintenance_risk(technology);
        let performance_risk = self.calculate_performance_risk(technology);
        
        (security_risk * self.risk_thresholds.vulnerability_weight +
         maintenance_risk * self.risk_thresholds.maintenance_weight +
         performance_risk * self.risk_thresholds.performance_weight)
    }

    fn calculate_security_risk(&self, technology: &crate::core::stats::techstack::DetectedTechnology) -> f64 {
        // Simplified security risk calculation
        let base_risk = 1.0 - technology.confidence;
        
        // Adjust based on category
        match technology.category {
            TechCategory::Security | TechCategory::Authentication => base_risk * 0.5,
            TechCategory::Database | TechCategory::WebFramework => base_risk * 1.2,
            _ => base_risk,
        }
    }

    fn calculate_maintenance_risk(&self, technology: &crate::core::stats::techstack::DetectedTechnology) -> f64 {
        // Simplified maintenance risk calculation
        let maturity = self.assess_technology_maturity(technology);
        match maturity {
            MaturityLevel::Deprecated => 0.9,
            MaturityLevel::Declining => 0.7,
            MaturityLevel::Experimental => 0.6,
            MaturityLevel::EarlyAdopter => 0.4,
            MaturityLevel::GrowingAdoption => 0.3,
            MaturityLevel::Mainstream => 0.2,
            MaturityLevel::Mature => 0.1,
        }
    }

    fn calculate_performance_risk(&self, technology: &crate::core::stats::techstack::DetectedTechnology) -> f64 {
        // Simplified performance risk calculation
        match technology.category {
            TechCategory::Database => 0.3,
            TechCategory::WebFramework => 0.2,
            TechCategory::Runtime => 0.4,
            _ => 0.1,
        }
    }

    fn calculate_vendor_lock_in_risk(&self, technology: &crate::core::stats::techstack::DetectedTechnology) -> f64 {
        // Simplified vendor lock-in risk calculation
        if technology.oss == Some(true) {
            0.1
        } else if technology.saas == Some(true) {
            0.8
        } else {
            0.4
        }
    }

    fn calculate_obsolescence_risk(&self, technology: &crate::core::stats::techstack::DetectedTechnology) -> f64 {
        let maturity = self.assess_technology_maturity(technology);
        match maturity {
            MaturityLevel::Deprecated => 0.9,
            MaturityLevel::Declining => 0.7,
            MaturityLevel::Experimental => 0.5,
            _ => 0.2,
        }
    }

    fn identify_risk_factors(&self, technology: &crate::core::stats::techstack::DetectedTechnology) -> Vec<String> {
        let mut factors = Vec::new();
        
        if technology.confidence < 0.5 {
            factors.push("Low detection confidence".to_string());
        }
        
        let maturity = self.assess_technology_maturity(technology);
        if matches!(maturity, MaturityLevel::Deprecated | MaturityLevel::Declining) {
            factors.push("Technology is deprecated or declining".to_string());
        }
        
        if technology.oss == Some(false) {
            factors.push("Proprietary technology with potential vendor lock-in".to_string());
        }
        
        factors
    }

    fn suggest_mitigation_strategies(&self, technology: &crate::core::stats::techstack::DetectedTechnology) -> Vec<String> {
        let mut strategies = Vec::new();
        
        let maturity = self.assess_technology_maturity(technology);
        if matches!(maturity, MaturityLevel::Deprecated | MaturityLevel::Declining) {
            strategies.push("Consider migration to modern alternatives".to_string());
        }
        
        if technology.oss == Some(false) {
            strategies.push("Evaluate open-source alternatives".to_string());
        }
        
        strategies.push("Regular security audits and updates".to_string());
        strategies.push("Monitor technology roadmap and support status".to_string());
        
        strategies
    }

    fn calculate_technology_innovation_score(&self, technology: &crate::core::stats::techstack::DetectedTechnology) -> f64 {
        let maturity = self.assess_technology_maturity(technology);
        let base_score = match maturity {
            MaturityLevel::Experimental => 0.9,
            MaturityLevel::EarlyAdopter => 0.8,
            MaturityLevel::GrowingAdoption => 0.7,
            MaturityLevel::Mainstream => 0.5,
            MaturityLevel::Mature => 0.3,
            MaturityLevel::Declining => 0.1,
            MaturityLevel::Deprecated => 0.0,
        };
        
        base_score * technology.confidence
    }

    fn calculate_innovation_potential(&self, inventory: &TechStackInventory) -> f64 {
        let emerging_count = inventory.technologies.iter()
            .filter(|tech| {
                let maturity = self.assess_technology_maturity(tech);
                matches!(maturity, MaturityLevel::Experimental | MaturityLevel::EarlyAdopter)
            })
            .count();
        
        emerging_count as f64 / inventory.technologies.len().max(1) as f64
    }

    fn calculate_adoption_rate(&self, inventory: &TechStackInventory) -> f64 {
        let modern_count = inventory.technologies.iter()
            .filter(|tech| {
                let maturity = self.assess_technology_maturity(tech);
                matches!(maturity, MaturityLevel::GrowingAdoption | MaturityLevel::Mainstream | MaturityLevel::Mature)
            })
            .count();
        
        modern_count as f64 / inventory.technologies.len().max(1) as f64
    }

    fn calculate_future_readiness(&self, inventory: &TechStackInventory) -> f64 {
        let future_ready_count = inventory.technologies.iter()
            .filter(|tech| {
                let maturity = self.assess_technology_maturity(tech);
                !matches!(maturity, MaturityLevel::Declining | MaturityLevel::Deprecated)
            })
            .count();
        
        future_ready_count as f64 / inventory.technologies.len().max(1) as f64
    }
}

impl Default for TechStackStatsCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::stats::techstack::DetectedTechnology;

    #[test]
    fn test_stats_calculator_creation() {
        let calculator = TechStackStatsCalculator::new();
        assert!(calculator.risk_thresholds.high_risk_threshold > 0.0);
    }

    #[test]
    fn test_category_distribution() {
        let calculator = TechStackStatsCalculator::new();
        let inventory = create_test_inventory();
        let distribution = calculator.calculate_category_distribution(&inventory);
        
        assert!(distribution.contains_key(&TechCategory::ProgrammingLanguage));
    }

    #[test]
    fn test_maturity_assessment() {
        let calculator = TechStackStatsCalculator::new();
        let tech = DetectedTechnology {
            name: "React".to_string(),
            version: Some("18.0.0".to_string()),
            category: TechCategory::WebFramework,
            confidence: 0.9,
            evidence: vec![],
            description: "JavaScript library".to_string(),
            website: None,
            documentation: None,
            license: None,
            oss: Some(true),
            saas: Some(false),
            pricing: None,
            implies: vec![],
            requires: vec![],
            excludes: vec![],
        };
        
        let maturity = calculator.assess_technology_maturity(&tech);
        assert!(matches!(maturity, MaturityLevel::Mature));
    }

    fn create_test_inventory() -> TechStackInventory {
        use crate::core::stats::techstack::detection::{AnalysisSummary, ArchitectureType, DeploymentType, SecurityPosture};
        
        TechStackInventory {
            technologies: vec![
                DetectedTechnology {
                    name: "JavaScript".to_string(),
                    version: None,
                    category: TechCategory::ProgrammingLanguage,
                    confidence: 0.9,
                    evidence: vec![],
                    description: "Programming language".to_string(),
                    website: None,
                    documentation: None,
                    license: None,
                    oss: Some(true),
                    saas: Some(false),
                    pricing: None,
                    implies: vec![],
                    requires: vec![],
                    excludes: vec![],
                }
            ],
            total_files_analyzed: 1,
            overall_confidence: 0.9,
            analysis_summary: AnalysisSummary {
                total_technologies: 1,
                primary_language: Some("JavaScript".to_string()),
                architecture_type: ArchitectureType::Unknown,
                deployment_type: DeploymentType::Unknown,
                security_posture: SecurityPosture::Unknown,
                modernization_score: 0.8,
            },
        }
    }
} 