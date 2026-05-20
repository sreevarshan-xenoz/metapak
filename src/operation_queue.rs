//! Operation queue for batch package operations.
//!
//! This module provides functionality for managing multiple package operations
//! in a queue, including preview, dependency checking, and transaction handling.

use crate::errors::Result;
use crate::models::Package;
use crate::transaction_manager::TransactionManager;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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
    #[allow(dead_code)]
    pub size: Option<u64>,
    #[allow(dead_code)]
    pub reason: Option<String>,
    #[allow(dead_code)]
    pub is_dep: bool,
}

impl Operation {
    #[allow(dead_code)]
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

    #[allow(dead_code)]
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

    #[allow(dead_code)]
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

    #[allow(dead_code)]
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

    #[allow(dead_code)]
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

    #[allow(dead_code)]
    pub fn summary(&self) -> String {
        match self.operation_type {
            OperationType::Install => format!("+{}", self.package_name),
            OperationType::Remove => format!("-{}", self.package_name),
            OperationType::Update => format!("~{}", self.package_name),
            OperationType::Reinstall => format!("#{}", self.package_name),
        }
    }
}

#[allow(dead_code)]
fn generate_id() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}

#[derive(Clone, Default)]
pub struct OperationQueue {
    operations: Vec<Operation>,
    #[allow(dead_code)]
    confirmed: bool,
    #[allow(dead_code)]
    transaction_manager: Option<Arc<TransactionManager>>,
}

impl OperationQueue {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(dead_code)]
    pub fn with_manager(manager: Arc<TransactionManager>) -> Self {
        Self {
            operations: Vec::new(),
            confirmed: false,
            transaction_manager: Some(manager),
        }
    }

    #[allow(dead_code)]
    pub async fn execute_safe<F, Fut, T>(
        &self,
        action_name: &str,
        commands: Option<&[crate::services::CommandSpec]>,
        action: F,
    ) -> Result<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        if let Some(manager) = &self.transaction_manager {
            manager
                .run_safe_transaction(action_name, commands, action)
                .await
        } else {
            tracing::warn!("No TransactionManager configured, running directly");
            action().await
        }
    }

    #[allow(dead_code)]
    pub fn add(&mut self, operation: Operation) {
        if !self.operations.iter().any(|o| {
            o.package_name == operation.package_name && o.operation_type == operation.operation_type
        }) {
            self.operations.push(operation);
        }
    }

    #[allow(dead_code)]
    pub fn add_install(&mut self, package: &Package) {
        self.add(Operation::install(package));
    }

    #[allow(dead_code)]
    pub fn add_remove(&mut self, package: &Package) {
        self.add(Operation::remove(package));
    }

    #[allow(dead_code)]
    pub fn add_update(&mut self, package: &Package) {
        self.add(Operation::update(package));
    }

    #[allow(dead_code)]
    pub fn remove(&mut self, id: u64) -> Option<Operation> {
        if let Some(pos) = self.operations.iter().position(|o| o.id == id) {
            Some(self.operations.remove(pos))
        } else {
            None
        }
    }

    #[allow(dead_code)]
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

    #[allow(dead_code)]
    pub fn move_up(&mut self, id: u64) -> bool {
        if let Some(pos) = self.operations.iter().position(|o| o.id == id) {
            if pos > 0 {
                self.operations.swap(pos, pos - 1);
                return true;
            }
        }
        false
    }

    #[allow(dead_code)]
    pub fn move_down(&mut self, id: u64) -> bool {
        if let Some(pos) = self.operations.iter().position(|o| o.id == id) {
            if pos < self.operations.len() - 1 {
                self.operations.swap(pos, pos + 1);
                return true;
            }
        }
        false
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.operations.clear();
        self.confirmed = false;
    }

    #[allow(dead_code)]
    pub fn operations(&self) -> &[Operation] {
        &self.operations
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.operations.len()
    }

    #[allow(dead_code)]
    pub fn confirm(&mut self) {
        self.confirmed = true;
    }

    #[allow(dead_code)]
    pub fn is_confirmed(&self) -> bool {
        self.confirmed
    }

    #[allow(dead_code)]
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

    #[allow(dead_code)]
    pub fn get_package_names(&self) -> Vec<&str> {
        self.operations
            .iter()
            .map(|op| op.package_name.as_str())
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct OperationPreview {
    pub to_install: Vec<String>,
    pub to_remove: Vec<String>,
    pub to_update: Vec<String>,
    #[allow(dead_code)]
    pub total_download_size_kb: u64,
    #[allow(dead_code)]
    pub total_removed_size_kb: u64,
}

impl OperationPreview {
    #[allow(dead_code)]
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

    #[allow(dead_code)]
    pub fn has_operations(&self) -> bool {
        !self.to_install.is_empty() || !self.to_remove.is_empty() || !self.to_update.is_empty()
    }
}

#[allow(dead_code)]
pub struct TransactionLog {
    entries: Vec<TransactionEntry>,
}

impl TransactionLog {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn log(&mut self, operation: &Operation, status: TransactionStatus) {
        self.entries.push(TransactionEntry {
            timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            package_name: operation.package_name.clone(),
            operation_type: operation.operation_type.clone(),
            version: operation.version.clone(),
            status,
        });
    }

    #[allow(dead_code)]
    pub fn entries(&self) -> &[TransactionEntry] {
        &self.entries
    }

    #[allow(dead_code)]
    pub fn successful_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|e| e.status == TransactionStatus::Success)
            .count()
    }

    #[allow(dead_code)]
    pub fn failed_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|e| e.status == TransactionStatus::Failed)
            .count()
    }
}

impl Default for TransactionLog {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct TransactionEntry {
    #[allow(dead_code)]
    pub timestamp: String,
    #[allow(dead_code)]
    pub package_name: String,
    #[allow(dead_code)]
    pub operation_type: OperationType,
    #[allow(dead_code)]
    pub version: Option<String>,
    pub status: TransactionStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransactionStatus {
    #[allow(dead_code)]
    Pending,
    Success,
    Failed,
    #[allow(dead_code)]
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
