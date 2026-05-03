//! Operation queue for batch package operations.
//!
//! This module provides functionality for managing multiple package operations
//! in a queue, including preview, dependency checking, and transaction handling.

use crate::models::{Package, PackageSource};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationType {
    Install,
    Remove,
    Update,
    Reinstall,
}

#[derive(Debug, Clone)]
pub struct Operation {
    pub id: u64,
    pub package_name: String,
    pub operation_type: OperationType,
    pub version: Option<String>,
    pub size: Option<u64>,
    pub reason: Option<String>,
    pub is_dep: bool,
}

impl Operation {
    pub fn install(package: &Package) -> Self {
        Self {
            id: generate_id(),
            package_name: package.name.clone(),
            operation_type: OperationType::Install,
            version: Some(package.version.clone()),
            size: package.download_size,
            reason: None,
            is_dep: false,
        }
    }

    pub fn remove(package: &Package) -> Self {
        Self {
            id: generate_id(),
            package_name: package.name.clone(),
            operation_type: OperationType::Remove,
            version: Some(package.version.clone()),
            size: package.installed_size,
            reason: None,
            is_dep: false,
        }
    }

    pub fn update(package: &Package) -> Self {
        Self {
            id: generate_id(),
            package_name: package.name.clone(),
            operation_type: OperationType::Update,
            version: Some(package.version.clone()),
            size: package.download_size,
            reason: None,
            is_dep: false,
        }
    }

    pub fn new_install(name: String, size: Option<u64>) -> Self {
        Self {
            id: generate_id(),
            package_name: name,
            operation_type: OperationType::Install,
            version: None,
            size,
            reason: None,
            is_dep: false,
        }
    }

    pub fn new_remove(name: String, size: Option<u64>) -> Self {
        Self {
            id: generate_id(),
            package_name: name,
            operation_type: OperationType::Remove,
            version: None,
            size,
            reason: None,
            is_dep: false,
        }
    }

    pub fn summary(&self) -> String {
        match self.operation_type {
            OperationType::Install => format!("+{}", self.package_name),
            OperationType::Remove => format!("-{}", self.package_name),
            OperationType::Update => format!("~{}", self.package_name),
            OperationType::Reinstall => format!("#{}", self.package_name),
        }
    }
}

fn generate_id() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}

#[derive(Debug, Clone, Default)]
pub struct OperationQueue {
    operations: Vec<Operation>,
    confirmed: bool,
}

impl OperationQueue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, operation: Operation) {
        if !self.operations.iter().any(|o| o.package_name == operation.package_name && o.operation_type == operation.operation_type) {
            self.operations.push(operation);
        }
    }

    pub fn add_install(&mut self, package: &Package) {
        self.add(Operation::install(package));
    }

    pub fn add_remove(&mut self, package: &Package) {
        self.add(Operation::remove(package));
    }

    pub fn add_update(&mut self, package: &Package) {
        self.add(Operation::update(package));
    }

    pub fn remove(&mut self, id: u64) -> Option<Operation> {
        if let Some(pos) = self.operations.iter().position(|o| o.id == id) {
            Some(self.operations.remove(pos))
        } else {
            None
        }
    }

    pub fn remove_by_name(&mut self, name: &str) -> Vec<Operation> {
        let mut removed = Vec::new();
        self.operations.retain(|op| {
            if op.package_name == name {
                removed.push(op.clone());
                false
            } else {
                true
            }
        });
        removed
    }

    pub fn move_up(&mut self, id: u64) -> bool {
        if let Some(pos) = self.operations.iter().position(|o| o.id == id) {
            if pos > 0 {
                self.operations.swap(pos, pos - 1);
                return true;
            }
        }
        false
    }

    pub fn move_down(&mut self, id: u64) -> bool {
        if let Some(pos) = self.operations.iter().position(|o| o.id == id) {
            if pos < self.operations.len() - 1 {
                self.operations.swap(pos, pos + 1);
                return true;
            }
        }
        false
    }

    pub fn clear(&mut self) {
        self.operations.clear();
        self.confirmed = false;
    }

    pub fn operations(&self) -> &[Operation] {
        &self.operations
    }

    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }

    pub fn len(&self) -> usize {
        self.operations.len()
    }

    pub fn confirm(&mut self) {
        self.confirmed = true;
    }

    pub fn is_confirmed(&self) -> bool {
        self.confirmed
    }

    pub fn preview(&self) -> OperationPreview {
        let mut to_install = Vec::new();
        let mut to_remove = Vec::new();
        let mut to_update = Vec::new();
        let mut total_download: u64 = 0;
        let mut total_remove_size: u64 = 0;

        for op in &self.operations {
            match op.operation_type {
                OperationType::Install => {
                    to_install.push(op.package_name.clone());
                    if let Some(size) = op.size {
                        total_download += size;
                    }
                }
                OperationType::Remove => {
                    to_remove.push(op.package_name.clone());
                    if let Some(size) = op.size {
                        total_remove_size += size;
                    }
                }
                OperationType::Update => {
                    to_update.push(op.package_name.clone());
                    if let Some(size) = op.size {
                        total_download += size;
                    }
                }
                OperationType::Reinstall => {
                    to_update.push(op.package_name.clone());
                }
            }
        }

        OperationPreview {
            to_install,
            to_remove,
            to_update,
            total_download_size_kb: total_download,
            total_removed_size_kb: total_remove_size,
        }
    }

    pub fn get_package_names(&self) -> Vec<&str> {
        self.operations.iter().map(|op| op.package_name.as_str()).collect()
    }
}

#[derive(Debug, Clone)]
pub struct OperationPreview {
    pub to_install: Vec<String>,
    pub to_remove: Vec<String>,
    pub to_update: Vec<String>,
    pub total_download_size_kb: u64,
    pub total_removed_size_kb: u64,
}

impl OperationPreview {
    pub fn format_summary(&self) -> String {
        let mut parts = Vec::new();

        if !self.to_install.is_empty() {
            parts.push(format!("Install: {}", self.to_install.len()));
        }
        if !self.to_remove.is_empty() {
            parts.push(format!("Remove: {}", self.to_remove.len()));
        }
        if !self.to_update.is_empty() {
            parts.push(format!("Update: {}", self.to_update.len()));
        }

        if parts.is_empty() {
            "No operations".to_string()
        } else {
            parts.join(", ")
        }
    }

    pub fn has_operations(&self) -> bool {
        !self.to_install.is_empty() || !self.to_remove.is_empty() || !self.to_update.is_empty()
    }
}

pub struct DependencyChecker;

impl DependencyChecker {
    pub fn check_conflicts(
        operations: &[Operation],
        installed_packages: &[Package],
    ) -> Vec<Conflict> {
        let mut conflicts = Vec::new();
        let mut packages_to_remove: HashSet<&str> = operations
            .iter()
            .filter(|op| op.operation_type == OperationType::Remove)
            .map(|op| op.package_name.as_str())
            .collect();

        let packages_to_install: HashSet<&str> = operations
            .iter()
            .filter(|op| op.operation_type == OperationType::Install)
            .map(|op| op.package_name.as_str())
            .collect();

        for pkg in installed_packages {
            if packages_to_remove.contains(pkg.name.as_str()) {
                for dep in &pkg.depends_on {
                    if packages_to_install.contains(dep.as_str()) {
                        conflicts.push(Conflict {
                            package1: pkg.name.clone(),
                            package2: dep.clone(),
                            conflict_type: ConflictType::ReverseDep,
                        });
                    }
                }
            }
        }

        conflicts
    }

    pub fn check_orphans(
        operations: &[Operation],
        installed_packages: &[Package],
    ) -> Vec<String> {
        let mut potential_orphans = Vec::new();
        let packages_to_remove: HashSet<&str> = operations
            .iter()
            .filter(|op| op.operation_type == OperationType::Remove)
            .map(|op| op.package_name.as_str())
            .collect();

        for pkg in installed_packages {
            if packages_to_remove.contains(pkg.name.as_str()) {
                continue;
            }

            let mut all_deps_satisfied = true;
            for dep in &pkg.depends_on {
                let dep_installed = installed_packages.iter().any(|p| {
                    p.name == *dep && !packages_to_remove.contains(p.name.as_str())
                });
                if !dep_installed {
                    all_deps_satisfied = false;
                    break;
                }
            }

            if all_deps_satisfied {
                let is_required_by_others = installed_packages.iter().any(|p| {
                    !packages_to_remove.contains(p.name.as_str())
                        && p.depends_on.iter().any(|d| d == &pkg.name)
                });
                if !is_required_by_others {
                    potential_orphans.push(pkg.name.clone());
                }
            }
        }

        potential_orphans
    }
}

#[derive(Debug, Clone)]
pub struct Conflict {
    pub package1: String,
    pub package2: String,
    pub conflict_type: ConflictType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConflictType {
    ReverseDep,
    Conflicts,
    Provides,
}

impl Conflict {
    pub fn description(&self) -> String {
        match self.conflict_type {
            ConflictType::ReverseDep => format!(
                "{} is required by {}",
                self.package2, self.package1
            ),
            ConflictType::Conflicts => format!(
                "{} conflicts with {}",
                self.package1, self.package2
            ),
            ConflictType::Provides => format!(
                "{} provides {}",
                self.package1, self.package2
            ),
        }
    }
}

pub struct TransactionLog {
    entries: Vec<TransactionEntry>,
}

impl TransactionLog {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    pub fn log(&mut self, operation: &Operation, status: TransactionStatus) {
        self.entries.push(TransactionEntry {
            timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            package_name: operation.package_name.clone(),
            operation_type: operation.operation_type.clone(),
            version: operation.version.clone(),
            status,
        });
    }

    pub fn entries(&self) -> &[TransactionEntry] {
        &self.entries
    }

    pub fn successful_count(&self) -> usize {
        self.entries.iter().filter(|e| e.status == TransactionStatus::Success).count()
    }

    pub fn failed_count(&self) -> usize {
        self.entries.iter().filter(|e| e.status == TransactionStatus::Failed).count()
    }
}

impl Default for TransactionLog {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct TransactionEntry {
    pub timestamp: String,
    pub package_name: String,
    pub operation_type: OperationType,
    pub version: Option<String>,
    pub status: TransactionStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransactionStatus {
    Pending,
    Success,
    Failed,
    Skipped,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_queue_add() {
        let mut queue = OperationQueue::new();
        queue.add(Operation::new_install("vim".to_string(), Some(1000)));
        queue.add(Operation::new_install("neovim".to_string(), Some(2000)));

        assert_eq!(queue.len(), 2);
    }

    #[test]
    fn test_operation_queue_no_duplicates() {
        let mut queue = OperationQueue::new();
        queue.add(Operation::new_install("vim".to_string(), Some(1000)));
        queue.add(Operation::new_install("vim".to_string(), Some(1000)));

        assert_eq!(queue.len(), 1);
    }

    #[test]
    fn test_operation_preview() {
        let mut queue = OperationQueue::new();
        queue.add(Operation::new_install("vim".to_string(), Some(1000)));
        queue.add(Operation::new_remove("emacs".to_string(), Some(5000)));

        let preview = queue.preview();
        assert!(preview.to_install.contains(&"vim".to_string()));
        assert!(preview.to_remove.contains(&"emacs".to_string()));
    }

    #[test]
    fn test_move_operations() {
        let mut queue = OperationQueue::new();
        queue.add(Operation::new_install("a".to_string(), None));
        queue.add(Operation::new_install("b".to_string(), None));
        queue.add(Operation::new_install("c".to_string(), None));

        let id = queue.operations()[1].id;
        queue.move_up(id);

        assert_eq!(queue.operations()[0].package_name, "b");
    }
}