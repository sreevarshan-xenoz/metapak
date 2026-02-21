//! Package dependency visualization module
//!
//! This module provides functionality for visualizing package dependencies
//! in a tree-like structure to help users understand relationships between packages.

use crate::errors::AppError;
use crate::models::Package;
use std::collections::HashSet;
use std::process::Command;

/// Represents a dependency relationship between packages
#[derive(Debug, Clone)]
pub struct DependencyNode {
    /// Name of the package
    pub name: String,
    /// Version of the package
    pub version: String,
    /// Whether the package is installed
    pub is_installed: bool,
    /// Child dependencies
    pub children: Vec<DependencyNode>,
}

impl DependencyNode {
    /// Creates a new dependency node
    pub fn new(name: String, version: String, is_installed: bool) -> Self {
        Self {
            name,
            version,
            is_installed,
            children: Vec::new(),
        }
    }

    /// Adds a child dependency node
    pub fn add_child(&mut self, child: DependencyNode) {
        self.children.push(child);
    }
}

/// Service for building dependency trees
pub struct DependencyVisualizationService;

#[derive(Debug, Clone)]
struct PacmanPackageInfo {
    version: String,
    depends_on: Vec<String>,
    is_installed: bool,
}

struct PacmanDependencyResolver;

impl PacmanDependencyResolver {
    fn resolve(&self, package_name: &str) -> Result<Option<PacmanPackageInfo>, AppError> {
        let installed = self.query_installed(package_name)?;
        if installed.is_some() {
            return Ok(installed);
        }
        self.query_sync_database(package_name)
    }

    fn query_installed(&self, package_name: &str) -> Result<Option<PacmanPackageInfo>, AppError> {
        let output = Command::new("pacman")
            .arg("-Qi")
            .arg(package_name)
            .output()
            .map_err(|e| {
                AppError::Dependency(format!(
                    "Failed to query installed package '{}': {}",
                    package_name, e
                ))
            })?;

        if !output.status.success() {
            return Ok(None);
        }

        let stdout = String::from_utf8(output.stdout)
            .map_err(|e| AppError::Dependency(format!("Invalid UTF-8 from pacman -Qi: {}", e)))?;
        Self::parse_pacman_info(&stdout, true).map(Some)
    }

    fn query_sync_database(
        &self,
        package_name: &str,
    ) -> Result<Option<PacmanPackageInfo>, AppError> {
        let output = Command::new("pacman")
            .arg("-Si")
            .arg(package_name)
            .output()
            .map_err(|e| {
                AppError::Dependency(format!(
                    "Failed to query sync package '{}': {}",
                    package_name, e
                ))
            })?;

        if !output.status.success() {
            return Ok(None);
        }

        let stdout = String::from_utf8(output.stdout)
            .map_err(|e| AppError::Dependency(format!("Invalid UTF-8 from pacman -Si: {}", e)))?;
        Self::parse_pacman_info(&stdout, false).map(Some)
    }

    fn parse_pacman_info(info: &str, is_installed: bool) -> Result<PacmanPackageInfo, AppError> {
        if info.trim().is_empty() {
            return Err(AppError::Dependency(
                "Empty pacman metadata output".to_string(),
            ));
        }

        let mut version = "unknown".to_string();
        let mut depends_on = Vec::new();
        let mut collecting_depends = false;

        for line in info.lines() {
            if line.trim().is_empty() {
                continue;
            }

            if let Some((key, value)) = line.split_once(':') {
                collecting_depends = false;
                let key = key.trim();
                let value = value.trim();

                match key {
                    "Version" => {
                        if !value.is_empty() {
                            version = value.to_string();
                        }
                    }
                    "Depends On" => {
                        collecting_depends = true;
                        Self::append_dependencies(&mut depends_on, value);
                    }
                    _ => {}
                }
            } else if collecting_depends {
                Self::append_dependencies(&mut depends_on, line.trim());
            }
        }

        Ok(PacmanPackageInfo {
            version,
            depends_on,
            is_installed,
        })
    }

    fn append_dependencies(buffer: &mut Vec<String>, raw: &str) {
        if raw.is_empty() || raw == "None" {
            return;
        }

        for token in raw.split_whitespace() {
            let name = Self::sanitize_dependency_name(token);
            if !name.is_empty() {
                buffer.push(name);
            }
        }
    }

    fn sanitize_dependency_name(raw: &str) -> String {
        raw.split(['<', '>', '='])
            .next()
            .unwrap_or(raw)
            .trim()
            .to_string()
    }
}

impl DependencyVisualizationService {
    /// Builds a dependency tree for a given package
    ///
    /// # Arguments
    /// * `package` - The package to visualize dependencies for
    /// * `max_depth` - Maximum depth to traverse (to prevent infinite loops)
    ///
    /// # Returns
    /// A DependencyNode representing the root of the dependency tree
    pub fn build_dependency_tree(package: &Package, max_depth: usize) -> DependencyNode {
        let (tree, _warnings) = Self::build_dependency_tree_safe(package, max_depth);
        tree
    }

    /// Builds a dependency tree while collecting non-fatal resolution warnings.
    pub fn build_dependency_tree_safe(
        package: &Package,
        max_depth: usize,
    ) -> (DependencyNode, Vec<String>) {
        let resolver = PacmanDependencyResolver;
        let mut warnings = Vec::new();
        let tree = Self::build_dependency_tree_recursive(
            &resolver,
            &package.name,
            &package.version,
            package.is_installed,
            &package.depends_on,
            max_depth,
            &mut HashSet::new(),
            &mut warnings,
        );
        (tree, warnings)
    }

    /// Recursively builds the dependency tree
    fn build_dependency_tree_recursive(
        resolver: &PacmanDependencyResolver,
        package_name: &str,
        version: &str,
        is_installed: bool,
        fallback_depends: &[String],
        remaining_depth: usize,
        visited: &mut HashSet<String>,
        warnings: &mut Vec<String>,
    ) -> DependencyNode {
        let mut node =
            DependencyNode::new(package_name.to_string(), version.to_string(), is_installed);

        // Prevent circular dependencies by tracking visited packages
        if !visited.insert(package_name.to_string()) {
            return node;
        }

        if remaining_depth == 0 {
            visited.remove(package_name);
            return node;
        }

        let dependencies = match resolver.resolve(package_name) {
            Ok(Some(meta)) => meta.depends_on,
            Ok(None) => fallback_depends.to_vec(),
            Err(e) => {
                warnings.push(format!("{}: {}", package_name, e));
                fallback_depends.to_vec()
            }
        };

        for dep_name in dependencies {
            match resolver.resolve(&dep_name) {
                Ok(Some(meta)) => {
                    let child = Self::build_dependency_tree_recursive(
                        resolver,
                        &dep_name,
                        &meta.version,
                        meta.is_installed,
                        &meta.depends_on,
                        remaining_depth.saturating_sub(1),
                        visited,
                        warnings,
                    );
                    node.add_child(child);
                }
                Ok(None) => {
                    node.add_child(DependencyNode::new(dep_name, "unknown".to_string(), false));
                }
                Err(e) => {
                    warnings.push(format!("{}: {}", dep_name, e));
                    node.add_child(DependencyNode::new(dep_name, "unknown".to_string(), false));
                }
            }
        }

        // Remove from visited set when backtracking
        visited.remove(package_name);

        node
    }

    /// Formats the dependency tree as a string for display
    pub fn format_tree(node: &DependencyNode, indent_level: usize) -> String {
        let indent = "  ".repeat(indent_level);
        let status = if node.is_installed { "✓" } else { "○" };
        let mut result = format!("{}{} {} ({})\n", indent, status, node.name, node.version);

        for child in &node.children {
            result.push_str(&Self::format_tree(child, indent_level + 1));
        }

        result
    }

    /// Gets a flat list of all dependencies (direct and indirect)
    pub fn get_all_dependencies(root: &DependencyNode) -> Vec<String> {
        let mut deps = Vec::new();
        Self::collect_dependencies_recursive(root, &mut deps);
        deps
    }

    /// Recursively collects all dependencies
    fn collect_dependencies_recursive(node: &DependencyNode, deps: &mut Vec<String>) {
        for child in &node.children {
            if !deps.contains(&child.name) {
                deps.push(child.name.clone());
            }
            Self::collect_dependencies_recursive(child, deps);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Package, PackageSource};

    #[test]
    fn test_build_simple_dependency_tree() {
        let package = Package {
            name: "test-package".to_string(),
            version: "1.0.0".to_string(),
            description: "A test package".to_string(),
            source: PackageSource::Pacman,
            is_installed: true,
            installed_size: None,
            download_size: None,
            groups: vec![],
            licenses: vec![],
            maintainers: vec![],
            keywords: vec![],
            url: None,
            depends_on: vec!["dependency1".to_string(), "dependency2".to_string()],
            required_by: vec![],
            opt_depends: vec![],
            conflicts: vec![],
            replaces: vec![],
            provides: vec![],
        };

        let tree = DependencyVisualizationService::build_dependency_tree(&package, 3);
        assert_eq!(tree.name, "test-package");
        assert_eq!(tree.children.len(), 2);
        assert_eq!(tree.children[0].name, "dependency1");
        assert_eq!(tree.children[1].name, "dependency2");
    }

    #[test]
    fn test_sanitize_dependency_name() {
        assert_eq!(
            PacmanDependencyResolver::sanitize_dependency_name("glibc>=2.39"),
            "glibc"
        );
        assert_eq!(
            PacmanDependencyResolver::sanitize_dependency_name("openssl=3.2.0"),
            "openssl"
        );
        assert_eq!(
            PacmanDependencyResolver::sanitize_dependency_name("zlib"),
            "zlib"
        );
    }

    #[test]
    fn test_parse_pacman_info_dependencies() {
        let info = r#"Name            : foo
Version         : 1.2.3-1
Depends On      : glibc>=2.39 gcc-libs
                  zlib>=1.3
Description     : Test package
"#;

        let parsed = PacmanDependencyResolver::parse_pacman_info(info, true).unwrap();
        assert_eq!(parsed.version, "1.2.3-1");
        assert!(parsed.is_installed);
        assert_eq!(parsed.depends_on, vec!["glibc", "gcc-libs", "zlib"]);
    }

    #[test]
    fn test_parse_pacman_info_empty_returns_error() {
        let parsed = PacmanDependencyResolver::parse_pacman_info("", true);
        assert!(parsed.is_err());
    }

    #[test]
    fn test_format_tree() {
        let mut root = DependencyNode::new("root".to_string(), "1.0".to_string(), true);
        let child1 = DependencyNode::new("child1".to_string(), "1.0".to_string(), false);
        let mut child2 = DependencyNode::new("child2".to_string(), "2.0".to_string(), true);
        let grandchild = DependencyNode::new("grandchild".to_string(), "1.5".to_string(), false);

        child2.add_child(grandchild);
        root.add_child(child1);
        root.add_child(child2);

        let formatted = DependencyVisualizationService::format_tree(&root, 0);
        assert!(formatted.contains("root"));
        assert!(formatted.contains("child1"));
        assert!(formatted.contains("child2"));
        assert!(formatted.contains("grandchild"));
    }

    #[test]
    fn test_get_all_dependencies() {
        let mut root = DependencyNode::new("root".to_string(), "1.0".to_string(), true);
        let child1 = DependencyNode::new("child1".to_string(), "1.0".to_string(), false);
        let mut child2 = DependencyNode::new("child2".to_string(), "2.0".to_string(), true);
        let grandchild = DependencyNode::new("grandchild".to_string(), "1.5".to_string(), false);

        child2.add_child(grandchild);
        root.add_child(child1);
        root.add_child(child2);

        let all_deps = DependencyVisualizationService::get_all_dependencies(&root);
        assert!(all_deps.contains(&"child1".to_string()));
        assert!(all_deps.contains(&"child2".to_string()));
        assert!(all_deps.contains(&"grandchild".to_string()));
        assert_eq!(all_deps.len(), 3);
    }
}
