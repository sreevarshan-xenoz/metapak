//! Package dependency visualization module
//! 
//! This module provides functionality for visualizing package dependencies
//! in a tree-like structure to help users understand relationships between packages.

use crate::models::Package;
use std::collections::{HashMap, HashSet};

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

    /// Checks if the node has any children
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }
}

/// Service for building dependency trees
pub struct DependencyVisualizationService;

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
        Self::build_dependency_tree_recursive(package, max_depth, &mut HashSet::new())
    }

    /// Recursively builds the dependency tree
    fn build_dependency_tree_recursive(
        package: &Package,
        remaining_depth: usize,
        visited: &mut HashSet<String>,
    ) -> DependencyNode {
        let mut node = DependencyNode::new(
            package.name.clone(),
            package.version.clone(),
            package.is_installed,
        );

        // Prevent circular dependencies by tracking visited packages
        if visited.contains(&package.name) {
            return node;
        }

        visited.insert(package.name.clone());

        if remaining_depth == 0 {
            return node;
        }

        // Add direct dependencies as children
        for dep_name in &package.depends_on {
            // In a real implementation, we would fetch the actual package info
            // For now, we'll create placeholder nodes
            let child_node = DependencyNode::new(
                dep_name.clone(),
                "unknown".to_string(),
                false, // We don't know if it's installed
            );
            node.add_child(child_node);
        }

        // Remove from visited set when backtracking
        visited.remove(&package.name);

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