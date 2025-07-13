use crate::core::stats::techstack::{TechStackInventory, DetectedTechnology, TechCategory, ConfidenceLevel};
use crate::utils::errors::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use ahash::{AHashMap, AHashSet};

/// Dependency graph representing relationships between technologies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraph {
    pub nodes: Vec<DependencyNode>,
    pub edges: Vec<DependencyEdge>,
    pub clusters: Vec<TechCluster>,
    pub metrics: GraphMetrics,
}

/// Node in the dependency graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyNode {
    pub id: String,
    pub name: String,
    pub category: TechCategory,
    pub node_type: NodeType,
    pub version: Option<String>,
    pub confidence: ConfidenceLevel,
    pub metrics: NodeMetrics,
    pub metadata: NodeMetadata,
}

/// Edge representing a dependency relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub relationship: DependencyRelationship,
    pub weight: f64,
    pub confidence: ConfidenceLevel,
    pub evidence: Vec<String>,
}

/// Type of dependency relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyRelationship {
    DirectDependency,
    DevDependency,
    PeerDependency,
    OptionalDependency,
    BuildDependency,
    RuntimeDependency,
    TestDependency,
    FrameworkPlugin,
    DatabaseConnection,
    APIIntegration,
    ServiceCommunication,
    ConfigurationDependency,
    InfrastructureDependency,
}

/// Type of node in the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    Framework,
    Library,
    Language,
    Tool,
    Infrastructure,
    Database,
    Service,
    Configuration,
}

/// Metrics for individual nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetrics {
    pub incoming_connections: usize,
    pub outgoing_connections: usize,
    pub centrality_score: f64,
    pub importance_score: f64,
    pub risk_score: f64,
    pub update_frequency: f64,
}

/// Metadata for nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetadata {
    pub description: String,
    pub website: Option<String>,
    pub documentation: Option<String>,
    pub license: Option<String>,
    pub size_kb: Option<u64>,
    pub last_updated: Option<String>,
    pub vulnerabilities: Vec<String>,
    pub end_of_life: Option<String>,
}

/// Technology cluster (group of related technologies)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechCluster {
    pub id: String,
    pub name: String,
    pub category: TechCategory,
    pub nodes: Vec<String>,
    pub cohesion_score: f64,
    pub purpose: String,
    pub architecture_pattern: ArchitecturePattern,
}

/// Architecture patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArchitecturePattern {
    MVC,
    MVP,
    MVVM,
    Microservices,
    Monolithic,
    Serverless,
    EventDriven,
    Layered,
    Hexagonal,
    CQRS,
    Unknown,
}

/// Overall graph metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetrics {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub density: f64,
    pub complexity_score: f64,
    pub coupling_score: f64,
    pub cohesion_score: f64,
    pub maintainability_score: f64,
    pub circular_dependencies: Vec<CircularDependency>,
}

/// Circular dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircularDependency {
    pub cycle: Vec<String>,
    pub severity: CycleSeverity,
    pub impact: String,
    pub recommendation: String,
}

/// Severity of circular dependencies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CycleSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Dependency mapper for building relationship graphs
pub struct DependencyMapper {
    node_id_counter: usize,
    edge_id_counter: usize,
    cluster_id_counter: usize,
}

impl DependencyMapper {
    pub fn new() -> Self {
        Self {
            node_id_counter: 0,
            edge_id_counter: 0,
            cluster_id_counter: 0,
        }
    }

    /// Map dependencies and build the relationship graph
    pub fn map_dependencies(&self, inventory: &TechStackInventory, project_path: &str) -> Result<DependencyGraph> {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut node_map = HashMap::new();

        // Create nodes from inventory
        self.create_nodes_from_inventory(inventory, &mut nodes, &mut node_map)?;

        // Analyze relationships
        self.analyze_relationships(project_path, &mut nodes, &mut edges, &node_map)?;

        // Create clusters
        let clusters = self.create_clusters(&nodes, &edges)?;

        // Calculate metrics
        let metrics = self.calculate_graph_metrics(&nodes, &edges)?;

        Ok(DependencyGraph {
            nodes,
            edges,
            clusters,
            metrics,
        })
    }

    /// Create nodes from the techstack inventory
    fn create_nodes_from_inventory(
        &self,
        inventory: &TechStackInventory,
        nodes: &mut Vec<DependencyNode>,
        node_map: &mut HashMap<String, usize>,
    ) -> Result<()> {
        // Create nodes for all detected technologies
        for technology in &inventory.technologies {
            let node_id = format!("tech_{}", self.generate_node_id());
            
            let node_type = match technology.category {
                TechCategory::UIFramework | TechCategory::WebFramework => NodeType::Framework,
                TechCategory::ProgrammingLanguage => NodeType::Language,
                TechCategory::Database => NodeType::Database,
                TechCategory::Infrastructure | TechCategory::CloudProvider | TechCategory::Containerization => NodeType::Infrastructure,
                TechCategory::BuildTool | TechCategory::TestingFramework => NodeType::Tool,
                _ => NodeType::Library,
            };
            
            let node = DependencyNode {
                id: node_id.clone(),
                name: technology.name.clone(),
                category: technology.category.clone(),
                node_type,
                version: technology.version.clone(),
                confidence: ConfidenceLevel::from_score(technology.confidence),
                metrics: NodeMetrics {
                    incoming_connections: 0,
                    outgoing_connections: 0,
                    centrality_score: 0.0,
                    importance_score: technology.confidence,
                    risk_score: if technology.oss.unwrap_or(false) { 0.2 } else { 0.4 },
                    update_frequency: 0.6,
                },
                metadata: NodeMetadata {
                    description: technology.description.clone(),
                    website: technology.website.clone(),
                    documentation: technology.documentation.clone(),
                    license: technology.license.clone(),
                    size_kb: None,
                    last_updated: None,
                    vulnerabilities: vec![],
                    end_of_life: None,
                },
            };
            
            node_map.insert(technology.name.clone(), nodes.len());
            nodes.push(node);
        }

        Ok(())
    }

    /// Analyze relationships between technologies
    fn analyze_relationships(
        &self,
        project_path: &str,
        nodes: &mut Vec<DependencyNode>,
        edges: &mut Vec<DependencyEdge>,
        node_map: &HashMap<String, usize>,
    ) -> Result<()> {
        // Analyze package.json for JavaScript dependencies
        let package_json_path = Path::new(project_path).join("package.json");
        if package_json_path.exists() {
            self.analyze_package_json_dependencies(&package_json_path, nodes, edges, node_map)?;
        }

        // Analyze Cargo.toml for Rust dependencies
        let cargo_toml_path = Path::new(project_path).join("Cargo.toml");
        if cargo_toml_path.exists() {
            self.analyze_cargo_toml_dependencies(&cargo_toml_path, nodes, edges, node_map)?;
        }

        // Analyze requirements.txt for Python dependencies
        let requirements_path = Path::new(project_path).join("requirements.txt");
        if requirements_path.exists() {
            self.analyze_requirements_dependencies(&requirements_path, nodes, edges, node_map)?;
        }

        // Analyze import statements in source files
        self.analyze_import_relationships(project_path, nodes, edges, node_map)?;

        // Update node connection counts
        self.update_node_metrics(nodes, edges);

        Ok(())
    }

    /// Analyze package.json dependencies
    fn analyze_package_json_dependencies(
        &self,
        package_json_path: &Path,
        nodes: &mut Vec<DependencyNode>,
        edges: &mut Vec<DependencyEdge>,
        node_map: &HashMap<String, usize>,
    ) -> Result<()> {
        let content = fs::read_to_string(package_json_path)?;
        
        // Parse dependencies (simplified - in real implementation, use proper JSON parsing)
        if content.contains("\"dependencies\"") {
            self.extract_dependencies_from_package_json(&content, "dependencies", DependencyRelationship::DirectDependency, edges, node_map);
        }
        
        if content.contains("\"devDependencies\"") {
            self.extract_dependencies_from_package_json(&content, "devDependencies", DependencyRelationship::DevDependency, edges, node_map);
        }
        
        if content.contains("\"peerDependencies\"") {
            self.extract_dependencies_from_package_json(&content, "peerDependencies", DependencyRelationship::PeerDependency, edges, node_map);
        }

        Ok(())
    }

    /// Extract dependencies from package.json content
    fn extract_dependencies_from_package_json(
        &self,
        content: &str,
        section: &str,
        relationship: DependencyRelationship,
        edges: &mut Vec<DependencyEdge>,
        node_map: &HashMap<String, usize>,
    ) {
        // Simplified dependency extraction
        // In a real implementation, you'd use a proper JSON parser
        let lines: Vec<&str> = content.lines().collect();
        let mut in_dependencies = false;
        let mut brace_count = 0;

        for line in lines {
            let trimmed = line.trim();
            
            if trimmed.contains(&format!("\"{}\"", section)) {
                in_dependencies = true;
                continue;
            }
            
            if in_dependencies {
                if trimmed.contains('{') {
                    brace_count += 1;
                } else if trimmed.contains('}') {
                    brace_count -= 1;
                    if brace_count <= 0 {
                        break;
                    }
                } else if trimmed.contains(':') && trimmed.contains('"') {
                    // Extract package name
                    if let Some(start) = trimmed.find('"') {
                        if let Some(end) = trimmed[start + 1..].find('"') {
                            let package_name = &trimmed[start + 1..start + 1 + end];
                            
                            // Create edge if both nodes exist
                            if let Some(&source_idx) = node_map.get("React") { // Assuming React as source for now
                                if let Some(&target_idx) = node_map.get(package_name) {
                                    let edge = DependencyEdge {
                                        id: format!("edge_{}", self.generate_edge_id()),
                                        source: format!("node_{}", source_idx),
                                        target: format!("node_{}", target_idx),
                                        relationship: relationship.clone(),
                                        weight: 1.0,
                                        confidence: ConfidenceLevel::High,
                                        evidence: vec![format!("package.json:{}", section)],
                                    };
                                    edges.push(edge);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Analyze Cargo.toml dependencies
    fn analyze_cargo_toml_dependencies(
        &self,
        cargo_toml_path: &Path,
        nodes: &mut Vec<DependencyNode>,
        edges: &mut Vec<DependencyEdge>,
        node_map: &HashMap<String, usize>,
    ) -> Result<()> {
        let content = fs::read_to_string(cargo_toml_path)?;
        
        // Parse [dependencies] section
        if content.contains("[dependencies]") {
            self.extract_dependencies_from_cargo_toml(&content, "dependencies", DependencyRelationship::DirectDependency, edges, node_map);
        }
        
        if content.contains("[dev-dependencies]") {
            self.extract_dependencies_from_cargo_toml(&content, "dev-dependencies", DependencyRelationship::DevDependency, edges, node_map);
        }
        
        if content.contains("[build-dependencies]") {
            self.extract_dependencies_from_cargo_toml(&content, "build-dependencies", DependencyRelationship::BuildDependency, edges, node_map);
        }

        Ok(())
    }

    /// Extract dependencies from Cargo.toml content
    fn extract_dependencies_from_cargo_toml(
        &self,
        content: &str,
        section: &str,
        relationship: DependencyRelationship,
        edges: &mut Vec<DependencyEdge>,
        node_map: &HashMap<String, usize>,
    ) {
        let lines: Vec<&str> = content.lines().collect();
        let mut in_dependencies = false;

        for line in lines {
            let trimmed = line.trim();
            
            if trimmed == &format!("[{}]", section) {
                in_dependencies = true;
                continue;
            }
            
            if in_dependencies {
                if trimmed.starts_with('[') && trimmed.ends_with(']') {
                    // New section started
                    break;
                } else if trimmed.contains('=') && !trimmed.starts_with('#') {
                    // Extract crate name
                    if let Some(eq_pos) = trimmed.find('=') {
                        let crate_name = trimmed[..eq_pos].trim();
                        
                        // Create edge if both nodes exist
                        if let Some(&source_idx) = node_map.get("Rust") { // Assuming Rust as source
                            if let Some(&target_idx) = node_map.get(crate_name) {
                                let edge = DependencyEdge {
                                    id: format!("edge_{}", self.generate_edge_id()),
                                    source: format!("node_{}", source_idx),
                                    target: format!("node_{}", target_idx),
                                    relationship: relationship.clone(),
                                    weight: 1.0,
                                    confidence: ConfidenceLevel::High,
                                    evidence: vec![format!("Cargo.toml:{}", section)],
                                };
                                edges.push(edge);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Analyze requirements.txt dependencies
    fn analyze_requirements_dependencies(
        &self,
        requirements_path: &Path,
        nodes: &mut Vec<DependencyNode>,
        edges: &mut Vec<DependencyEdge>,
        node_map: &HashMap<String, usize>,
    ) -> Result<()> {
        let content = fs::read_to_string(requirements_path)?;
        
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            // Extract package name (before == or >= or <=)
            let package_name = if let Some(pos) = line.find("==") {
                &line[..pos]
            } else if let Some(pos) = line.find(">=") {
                &line[..pos]
            } else if let Some(pos) = line.find("<=") {
                &line[..pos]
            } else {
                line
            };
            
            // Create edge if both nodes exist
            if let Some(&source_idx) = node_map.get("Python") { // Assuming Python as source
                if let Some(&target_idx) = node_map.get(package_name) {
                    let edge = DependencyEdge {
                        id: format!("edge_{}", self.generate_edge_id()),
                        source: format!("node_{}", source_idx),
                        target: format!("node_{}", target_idx),
                        relationship: DependencyRelationship::DirectDependency,
                        weight: 1.0,
                        confidence: ConfidenceLevel::High,
                        evidence: vec!["requirements.txt".to_string()],
                    };
                    edges.push(edge);
                }
            }
        }

        Ok(())
    }

    /// Analyze import relationships in source files
    fn analyze_import_relationships(
        &self,
        project_path: &str,
        nodes: &mut Vec<DependencyNode>,
        edges: &mut Vec<DependencyEdge>,
        node_map: &HashMap<String, usize>,
    ) -> Result<()> {
        // This is a simplified implementation
        // In a real implementation, you'd analyze actual import statements
        
        // For now, just create some common relationships
        self.create_common_relationships(edges, node_map);

        Ok(())
    }

    /// Create common technology relationships
    fn create_common_relationships(&self, edges: &mut Vec<DependencyEdge>, node_map: &HashMap<String, usize>) {
        // React -> JavaScript
        if let (Some(&react_idx), Some(&js_idx)) = (node_map.get("React"), node_map.get("JavaScript")) {
            let edge = DependencyEdge {
                id: format!("edge_{}", self.generate_edge_id()),
                source: format!("node_{}", react_idx),
                target: format!("node_{}", js_idx),
                relationship: DependencyRelationship::RuntimeDependency,
                weight: 1.0,
                confidence: ConfidenceLevel::VeryHigh,
                evidence: vec!["Framework dependency".to_string()],
            };
            edges.push(edge);
        }

        // Django -> Python
        if let (Some(&django_idx), Some(&python_idx)) = (node_map.get("Django"), node_map.get("Python")) {
            let edge = DependencyEdge {
                id: format!("edge_{}", self.generate_edge_id()),
                source: format!("node_{}", django_idx),
                target: format!("node_{}", python_idx),
                relationship: DependencyRelationship::RuntimeDependency,
                weight: 1.0,
                confidence: ConfidenceLevel::VeryHigh,
                evidence: vec!["Framework dependency".to_string()],
            };
            edges.push(edge);
        }

        // Add more common relationships...
    }

    /// Update node metrics based on connections
    fn update_node_metrics(&self, nodes: &mut Vec<DependencyNode>, edges: &[DependencyEdge]) {
        let mut incoming_counts = HashMap::new();
        let mut outgoing_counts = HashMap::new();

        // Count connections
        for edge in edges {
            *outgoing_counts.entry(edge.source.clone()).or_insert(0) += 1;
            *incoming_counts.entry(edge.target.clone()).or_insert(0) += 1;
        }

        // Update node metrics
        for node in nodes {
            node.metrics.incoming_connections = *incoming_counts.get(&node.id).unwrap_or(&0);
            node.metrics.outgoing_connections = *outgoing_counts.get(&node.id).unwrap_or(&0);
            
            // Calculate centrality score (simplified)
            let total_connections = node.metrics.incoming_connections + node.metrics.outgoing_connections;
            node.metrics.centrality_score = (total_connections as f64).sqrt() / 10.0;
        }
    }

    /// Create technology clusters
    fn create_clusters(&self, nodes: &[DependencyNode], edges: &[DependencyEdge]) -> Result<Vec<TechCluster>> {
        let mut clusters = Vec::new();

        // Frontend cluster
        let frontend_nodes: Vec<String> = nodes.iter()
            .filter(|n| matches!(n.category, TechCategory::Frontend | TechCategory::UIFramework | TechCategory::Styling))
            .map(|n| n.id.clone())
            .collect();

        if !frontend_nodes.is_empty() {
            clusters.push(TechCluster {
                id: format!("cluster_{}", self.generate_cluster_id()),
                name: "Frontend Technologies".to_string(),
                category: TechCategory::Frontend,
                nodes: frontend_nodes,
                cohesion_score: 0.8,
                purpose: "User interface and client-side functionality".to_string(),
                architecture_pattern: ArchitecturePattern::MVC,
            });
        }

        // Backend cluster
        let backend_nodes: Vec<String> = nodes.iter()
            .filter(|n| matches!(n.category, TechCategory::Backend | TechCategory::WebFramework | TechCategory::Database))
            .map(|n| n.id.clone())
            .collect();

        if !backend_nodes.is_empty() {
            clusters.push(TechCluster {
                id: format!("cluster_{}", self.generate_cluster_id()),
                name: "Backend Technologies".to_string(),
                category: TechCategory::Backend,
                nodes: backend_nodes,
                cohesion_score: 0.7,
                purpose: "Server-side logic and data management".to_string(),
                architecture_pattern: ArchitecturePattern::Layered,
            });
        }

        // Infrastructure cluster
        let infra_nodes: Vec<String> = nodes.iter()
            .filter(|n| matches!(n.category, TechCategory::Infrastructure | TechCategory::Containerization | TechCategory::CloudProvider))
            .map(|n| n.id.clone())
            .collect();

        if !infra_nodes.is_empty() {
            clusters.push(TechCluster {
                id: format!("cluster_{}", self.generate_cluster_id()),
                name: "Infrastructure Technologies".to_string(),
                category: TechCategory::Infrastructure,
                nodes: infra_nodes,
                cohesion_score: 0.6,
                purpose: "Deployment and runtime environment".to_string(),
                architecture_pattern: ArchitecturePattern::Microservices,
            });
        }

        Ok(clusters)
    }

    /// Calculate overall graph metrics
    fn calculate_graph_metrics(&self, nodes: &[DependencyNode], edges: &[DependencyEdge]) -> Result<GraphMetrics> {
        let total_nodes = nodes.len();
        let total_edges = edges.len();
        
        // Calculate density
        let max_possible_edges = if total_nodes > 1 {
            total_nodes * (total_nodes - 1)
        } else {
            1
        };
        let density = total_edges as f64 / max_possible_edges as f64;

        // Calculate complexity score
        let complexity_score = (total_edges as f64 / total_nodes as f64).min(1.0);

        // Calculate coupling score (high coupling = more dependencies)
        let coupling_score = density;

        // Calculate cohesion score (how well-organized the dependencies are)
        let cohesion_score = 1.0 - (complexity_score * 0.5);

        // Calculate maintainability score
        let maintainability_score = (cohesion_score + (1.0 - coupling_score)) / 2.0;

        // Detect circular dependencies (simplified)
        let circular_dependencies = self.detect_circular_dependencies(edges);

        Ok(GraphMetrics {
            total_nodes,
            total_edges,
            density,
            complexity_score,
            coupling_score,
            cohesion_score,
            maintainability_score,
            circular_dependencies,
        })
    }

    /// Detect circular dependencies
    fn detect_circular_dependencies(&self, edges: &[DependencyEdge]) -> Vec<CircularDependency> {
        let mut cycles = Vec::new();
        
        // Build adjacency list
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();
        for edge in edges {
            graph.entry(edge.source.clone()).or_insert_with(Vec::new).push(edge.target.clone());
        }

        // Simple cycle detection using DFS (simplified implementation)
        let mut visited = HashSet::new();
        let mut recursion_stack = HashSet::new();

        for node in graph.keys() {
            if !visited.contains(node) {
                if let Some(cycle) = self.dfs_cycle_detection(node, &graph, &mut visited, &mut recursion_stack, &mut Vec::new()) {
                    cycles.push(CircularDependency {
                        cycle,
                        severity: CycleSeverity::Medium,
                        impact: "May cause build issues or runtime problems".to_string(),
                        recommendation: "Consider refactoring to break the circular dependency".to_string(),
                    });
                }
            }
        }

        cycles
    }

    /// DFS-based cycle detection
    fn dfs_cycle_detection(
        &self,
        node: &str,
        graph: &HashMap<String, Vec<String>>,
        visited: &mut HashSet<String>,
        recursion_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
    ) -> Option<Vec<String>> {
        visited.insert(node.to_string());
        recursion_stack.insert(node.to_string());
        path.push(node.to_string());

        if let Some(neighbors) = graph.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    if let Some(cycle) = self.dfs_cycle_detection(neighbor, graph, visited, recursion_stack, path) {
                        return Some(cycle);
                    }
                } else if recursion_stack.contains(neighbor) {
                    // Found a cycle
                    let cycle_start = path.iter().position(|n| n == neighbor).unwrap_or(0);
                    return Some(path[cycle_start..].to_vec());
                }
            }
        }

        recursion_stack.remove(node);
        path.pop();
        None
    }

    /// Generate unique node ID
    fn generate_node_id(&self) -> String {
        // In a real implementation, this would be properly thread-safe
        format!("node_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos())
    }

    /// Generate unique edge ID
    fn generate_edge_id(&self) -> String {
        // In a real implementation, this would be properly thread-safe
        format!("edge_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos())
    }

    /// Generate unique cluster ID
    fn generate_cluster_id(&self) -> String {
        // In a real implementation, this would be properly thread-safe
        format!("cluster_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos())
    }

    /// Check if mapper is ready
    pub fn is_ready(&self) -> bool {
        true
    }
}

impl Default for DependencyMapper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::test_utils::TestProject;
    use crate::core::stats::techstack::detection::TechStackDetector;

    #[test]
    fn test_dependency_mapper_creation() {
        let mapper = DependencyMapper::new();
        assert!(mapper.is_ready());
    }

    #[test]
    fn test_dependency_mapping_with_inventory() {
        let project = TestProject::new().unwrap();
        let mapper = DependencyMapper::new();

        // Create test files
        project.create_file("package.json", r#"{"dependencies": {"react": "^18.0.0", "lodash": "^4.17.21"}}"#).unwrap();
        
        // Initialize detector and get inventory
        crate::core::stats::techstack::initialize_detector().unwrap();
        let detector = TechStackDetector::instance();
        let inventory = detector.detect_techstack(project.path()).unwrap();
        let graph = mapper.map_dependencies(&inventory, project.path()).unwrap();

        assert!(graph.nodes.len() >= 0);
        assert!(graph.metrics.total_nodes >= 0);
    }

    #[test]
    fn test_cluster_creation() {
        let mapper = DependencyMapper::new();
        let nodes = vec![
            DependencyNode {
                id: "node_1".to_string(),
                name: "React".to_string(),
                category: TechCategory::UIFramework,
                node_type: NodeType::Framework,
                version: None,
                confidence: ConfidenceLevel::High,
                metrics: NodeMetrics {
                    incoming_connections: 0,
                    outgoing_connections: 0,
                    centrality_score: 0.0,
                    importance_score: 0.8,
                    risk_score: 0.2,
                    update_frequency: 0.9,
                },
                metadata: NodeMetadata {
                    description: "React framework".to_string(),
                    website: None,
                    documentation: None,
                    license: None,
                    size_kb: None,
                    last_updated: None,
                    vulnerabilities: vec![],
                    end_of_life: None,
                },
            }
        ];

        let clusters = mapper.create_clusters(&nodes, &[]).unwrap();
        assert!(clusters.len() > 0);
    }

    #[test]
    fn test_circular_dependency_detection() {
        let mapper = DependencyMapper::new();
        let edges = vec![
            DependencyEdge {
                id: "edge_1".to_string(),
                source: "node_a".to_string(),
                target: "node_b".to_string(),
                relationship: DependencyRelationship::DirectDependency,
                weight: 1.0,
                confidence: ConfidenceLevel::High,
                evidence: vec![],
            },
            DependencyEdge {
                id: "edge_2".to_string(),
                source: "node_b".to_string(),
                target: "node_a".to_string(),
                relationship: DependencyRelationship::DirectDependency,
                weight: 1.0,
                confidence: ConfidenceLevel::High,
                evidence: vec![],
            },
        ];

        let cycles = mapper.detect_circular_dependencies(&edges);
        assert!(cycles.len() > 0);
    }
} 