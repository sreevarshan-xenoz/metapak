//! Package dependency visualization module
//!
//! This module provides functionality for visualizing package dependencies
//! in a tree-like structure to help users understand relationships between packages.
#![allow(dead_code)]
#![allow(clippy::too_many_arguments)]

use crate::errors::AppError;
use crate::models::Package;
use std::collections::HashSet;
use std::process::Command;
use std::thread;
use std::time::Duration;

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

/// Represents an interactive dependency relationship with UI state
#[derive(Debug, Clone)]
pub struct InteractiveDependencyNode {
    pub name: String,
    pub version: String,
    pub is_installed: bool,
    pub is_expanded: bool,
    pub is_orphan: bool,
    pub children: Vec<InteractiveDependencyNode>,
}

impl From<DependencyNode> for InteractiveDependencyNode {
    fn from(node: DependencyNode) -> Self {
        Self {
            name: node.name,
            version: node.version,
            is_installed: node.is_installed,
            is_expanded: true, // Default to expanded
            is_orphan: false,   // Will be calculated later
            children: node.children.into_iter().map(InteractiveDependencyNode::from).collect(),
        }
    }
}

/// A flattened representation of a dependency node for UI rendering
#[derive(Debug, Clone)]
pub struct FlattenedDependencyItem {
    pub name: String,
    pub version: String,
    pub is_installed: bool,
    pub is_expanded: bool,
    pub is_orphan: bool,
    pub has_children: bool,
    pub depth: usize,
    pub prefix: String,
    pub full_display_name: String,
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
    const MAX_RETRIES: usize = 3;

    fn resolve(&self, package_name: &str) -> Result<Option<PacmanPackageInfo>, AppError> {
        let installed = self.query_installed(package_name)?;
        if installed.is_some() {
            return Ok(installed);
        }
        self.query_sync_database(package_name)
    }

    fn query_installed(&self, package_name: &str) -> Result<Option<PacmanPackageInfo>, AppError> {
        let output = self.run_pacman_with_retry(&["-Qi", package_name], "installed package")?;

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
        let output = self.run_pacman_with_retry(&["-Si", package_name], "sync package")?;

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
                    "Version" if !value.is_empty() => {
                        version = value.to_string();
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

    fn run_pacman_with_retry(
        &self,
        args: &[&str],
        context: &str,
    ) -> Result<std::process::Output, AppError> {
        let mut last_err = None;
        for attempt in 0..Self::MAX_RETRIES {
            match Command::new("pacman").args(args).output() {
                Ok(output) => return Ok(output),
                Err(e) => {
                    last_err = Some(e.to_string());
                    if attempt + 1 < Self::MAX_RETRIES {
                        thread::sleep(Duration::from_millis(150 * (attempt as u64 + 1)));
                    }
                }
            }
        }

        Err(AppError::Dependency(format!(
            "Failed to query {} after {} retries: {}",
            context,
            Self::MAX_RETRIES,
            last_err.unwrap_or_else(|| "unknown error".to_string())
        )))
    }
}

impl DependencyVisualizationService {
    /// Flattens an interactive tree into a list of items for UI rendering
    pub fn flatten_interactive_tree(node: &InteractiveDependencyNode) -> Vec<FlattenedDependencyItem> {
        let mut items = Vec::new();
        Self::flatten_interactive_recursive(node, 0, &[], true, &mut items);
        items
    }

    fn flatten_interactive_recursive(
        node: &InteractiveDependencyNode,
        depth: usize,
        parent_prefixes: &[bool],
        is_last: bool,
        items: &mut Vec<FlattenedDependencyItem>
    ) {
        let mut prefix = String::new();
        for &has_sibling in parent_prefixes {
            prefix.push_str(if has_sibling { "│   " } else { "    " });
        }
        
        if depth > 0 {
            prefix.push_str(if is_last { "└── " } else { "├── " });
        }

        let expand_icon = if node.children.is_empty() {
            "  "
        } else if node.is_expanded {
            "▼ "
        } else {
            "▶ "
        };

        let status = if node.is_installed { "✓" } else { "○" };
        let orphan_tag = if node.is_orphan { " [Orphan]" } else { "" };
        
        items.push(FlattenedDependencyItem {
            name: node.name.clone(),
            version: node.version.clone(),
            is_installed: node.is_installed,
            is_expanded: node.is_expanded,
            is_orphan: node.is_orphan,
            has_children: !node.children.is_empty(),
            depth,
            prefix: prefix.clone(),
            full_display_name: format!("{}{}{} {} ({}){}", prefix, expand_icon, status, node.name, node.version, orphan_tag),
        });

        if node.is_expanded {
            let mut child_prefixes = parent_prefixes.to_vec();
            if depth > 0 {
                child_prefixes.push(!is_last);
            }
            let child_count = node.children.len();
            for (i, child) in node.children.iter().enumerate() {
                Self::flatten_interactive_recursive(child, depth + 1, &child_prefixes, i == child_count - 1, items);
            }
        }
    }

    pub fn toggle_node_expansion(node: &mut InteractiveDependencyNode, name: &str, target_depth: usize, current_depth: usize) -> bool {
        if node.name == name && target_depth == current_depth {
            node.is_expanded = !node.is_expanded;
            return true;
        }
        for child in &mut node.children {
            if Self::toggle_node_expansion(child, name, target_depth, current_depth + 1) {
                return true;
            }
        }
        false
    }

    /// Marks nodes in the tree as orphans if they are present in the provided set
    pub fn mark_orphans(node: &mut InteractiveDependencyNode, orphans: &HashSet<String>) {
        if orphans.contains(&node.name) {
            node.is_orphan = true;
        }
        for child in &mut node.children {
            Self::mark_orphans(child, orphans);
        }
    }

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

    /// Formats the dependency tree as a string with box-drawing characters
    pub fn format_tree(
        node: &DependencyNode,
        indent_level: usize,
        is_last: bool,
        is_root: bool,
    ) -> String {
        if is_root {
            let status = if node.is_installed { "✓" } else { "○" };
            let mut result = format!("{} {} ({})\n", status, node.name, node.version);

            let child_count = node.children.len();
            for (i, child) in node.children.iter().enumerate() {
                let child_is_last = i == child_count - 1;
                let prefixes: Vec<bool> = vec![];
                result.push_str(&Self::format_node(child, &prefixes, child_is_last));
            }
            result
        } else {
            let prefixes: Vec<bool> = vec![false; indent_level];
            Self::format_node(node, &prefixes, is_last)
        }
    }

    fn format_node(node: &DependencyNode, parent_prefixes: &[bool], is_last: bool) -> String {
        let status = if node.is_installed { "✓" } else { "○" };
        let mut result = String::new();

        for &has_sibling in parent_prefixes.iter() {
            if has_sibling {
                result.push_str("│   ");
            } else {
                result.push_str("    ");
            }
        }

        if is_last {
            result.push_str("└── ");
        } else {
            result.push_str("├── ");
        }

        result.push_str(&format!("{} {} ({})\n", status, node.name, node.version));

        let mut child_prefixes = parent_prefixes.to_vec();
        child_prefixes.push(!is_last);

        let child_count = node.children.len();
        for (i, child) in node.children.iter().enumerate() {
            let child_is_last = i == child_count - 1;
            result.push_str(&Self::format_node(child, &child_prefixes, child_is_last));
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
        let mut package = Package::new("test-package", "1.0.0");
        package.source = PackageSource::Pacman;
        package.is_installed = true;
        package.depends_on = vec!["dependency1".to_string(), "dependency2".to_string()];

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

        let formatted = DependencyVisualizationService::format_tree(&root, 0, true, true);
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
