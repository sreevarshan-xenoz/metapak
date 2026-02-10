//! Application state management for Arch TUI
//!
//! This module contains the main application state and logic for managing
//! the TUI application's state, including search results, selections,
//! and UI modes.

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use tokio::sync::mpsc::UnboundedSender;

use crate::action::Action;
use crate::config::AppConfig;

use crate::models::Package;
use crate::utils::PasswordInput;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Editing,
}

/// Search filter options
#[derive(Debug, Clone, PartialEq)]
pub enum FilterOption {
    All,
    Installed,
    NotInstalled,
    RepoOnly,
    AurOnly,
}

/// Sort options
#[derive(Debug, Clone, PartialEq)]
pub enum SortOption {
    NameAsc,
    NameDesc,
    Source, // Repo first, then AUR
}

/// Main application state
pub struct App {
    // Search
    pub search_input: String,
    pub input_mode: InputMode,
    pub results: Vec<Package>,
    pub filtered_results: Vec<Package>,

    // Pagination
    pub current_page: usize,
    pub items_per_page: usize,

    // Search history
    pub search_history: VecDeque<String>,
    pub history_index: Option<usize>,
    pub max_history_size: usize,

    // Debouncing
    pub last_search_time: Option<Instant>,
    pub search_debounce_duration: Duration,
    pub pending_search: Option<String>,

    // State
    pub should_quit: bool,
    pub is_loading: bool,
    pub action_tx: Option<UnboundedSender<Action>>,
    pub selected_index: Option<usize>,
    pub pending_command: Option<(String, Vec<String>)>,
    pub error_message: Option<String>,
    pub available_updates: Option<usize>,

    // Password - using secure input
    pub show_password_prompt: bool,
    pub password_input: PasswordInput,

    // Selection with undo
    pub selected_packages: HashMap<String, Package>,
    pub selection_history: VecDeque<SelectionAction>,
    pub max_undo_history: usize,

    // Confirmation
    pub show_confirm_prompt: bool,
    pub packages_pending_confirmation: Vec<Package>,

    // Console
    pub show_console: bool,
    pub console_buffer: Vec<String>,
    pub command_stdin_tx: Option<UnboundedSender<String>>,
    pub command_progress: Option<CommandProgress>,

    // Configuration
    pub config: AppConfig,

    // Views
    pub show_package_details: bool,
    pub show_dependency_visualization: bool,
    pub show_help: bool,

    // Localization
    pub localizer: crate::i18n::Localizer,

    // Filter and Sort
    pub current_filter: FilterOption,
    pub current_sort: SortOption,
}

/// Represents a selection action for undo functionality
#[derive(Debug, Clone)]
pub enum SelectionAction {
    Select(Package),
    Deselect(Package),
}

/// Progress information for running commands
#[derive(Debug, Clone)]
pub struct CommandProgress {
    pub current: usize,
    pub total: usize,
    pub current_package: String,
    pub status: ProgressStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProgressStatus {
    Downloading,
    Building,
    Installing,
    Complete,
    Error(String),
}

impl App {
    pub fn new() -> Self {
        Self {
            search_input: String::new(),
            input_mode: InputMode::Normal,
            results: Vec::new(),
            filtered_results: Vec::new(),

            // Pagination - 20 items per page
            current_page: 0,
            items_per_page: 20,

            // Search history
            search_history: VecDeque::new(),
            history_index: None,
            max_history_size: 50,

            // Debouncing - 300ms delay
            last_search_time: None,
            search_debounce_duration: Duration::from_millis(300),
            pending_search: None,

            should_quit: false,
            is_loading: false,
            action_tx: None,
            selected_index: None,
            pending_command: None,
            error_message: None,
            available_updates: None,

            show_password_prompt: true,
            password_input: PasswordInput::new(),

            selected_packages: HashMap::new(),
            selection_history: VecDeque::new(),
            max_undo_history: 20,

            show_confirm_prompt: false,
            packages_pending_confirmation: Vec::new(),

            show_console: false,
            console_buffer: Vec::new(),
            command_stdin_tx: None,
            command_progress: None,

            config: AppConfig::default(),

            show_package_details: false,
            show_dependency_visualization: false,
            show_help: false,

            localizer: crate::i18n::Localizer::new(),

            current_filter: FilterOption::All,
            current_sort: SortOption::NameAsc,
        }
    }

    pub fn set_sender(&mut self, tx: UnboundedSender<Action>) {
        self.action_tx = Some(tx);
    }

    // Search History Management
    pub fn add_to_history(&mut self, query: String) {
        if query.trim().is_empty() {
            return;
        }

        // Remove if already exists (move to front)
        self.search_history.retain(|q| q != &query);

        // Add to front
        self.search_history.push_front(query);

        // Trim to max size
        while self.search_history.len() > self.max_history_size {
            self.search_history.pop_back();
        }

        self.history_index = None;
    }

    pub fn navigate_history_up(&mut self) {
        if self.search_history.is_empty() {
            return;
        }

        let new_index = match self.history_index {
            None => 0,
            Some(idx) if idx + 1 < self.search_history.len() => idx + 1,
            Some(_) => return, // At end of history
        };

        self.history_index = Some(new_index);
        if let Some(query) = self.search_history.get(new_index) {
            self.search_input = query.clone();
        }
    }

    pub fn navigate_history_down(&mut self) {
        match self.history_index {
            None => return,
            Some(0) => {
                self.history_index = None;
                self.search_input.clear();
            }
            Some(idx) => {
                self.history_index = Some(idx - 1);
                if let Some(query) = self.search_history.get(idx - 1) {
                    self.search_input = query.clone();
                }
            }
        }
    }

    // Debounced Search
    pub fn trigger_search(&mut self, query: String) {
        self.pending_search = Some(query);
        self.last_search_time = Some(Instant::now());
    }

    pub fn should_execute_search(&self) -> Option<String> {
        if let (Some(query), Some(last_time)) = (&self.pending_search, self.last_search_time) {
            if last_time.elapsed() >= self.search_debounce_duration {
                return Some(query.clone());
            }
        }
        None
    }

    pub fn clear_pending_search(&mut self) {
        self.pending_search = None;
        self.last_search_time = None;
    }

    // Selection with Undo
    pub fn toggle_selection(&mut self) {
        if let Some(_idx) = self.selected_index {
            if let Some(pkg) = self.get_current_package().cloned() {
                if self.selected_packages.contains_key(&pkg.name) {
                    // Deselect
                    self.selected_packages.remove(&pkg.name);
                    self.add_selection_history(SelectionAction::Deselect(pkg));
                } else {
                    // Select
                    self.selected_packages.insert(pkg.name.clone(), pkg.clone());
                    self.add_selection_history(SelectionAction::Select(pkg));
                }
            }
        }
    }

    fn add_selection_history(&mut self, action: SelectionAction) {
        self.selection_history.push_front(action);
        while self.selection_history.len() > self.max_undo_history {
            self.selection_history.pop_back();
        }
    }

    pub fn undo_last_selection(&mut self) {
        if let Some(action) = self.selection_history.pop_front() {
            match action {
                SelectionAction::Select(pkg) => {
                    self.selected_packages.remove(&pkg.name);
                }
                SelectionAction::Deselect(pkg) => {
                    self.selected_packages.insert(pkg.name.clone(), pkg);
                }
            }
        }
    }

    // Pagination
    pub fn get_paginated_results(&self) -> Vec<&Package> {
        let results = if self.filtered_results.is_empty() {
            &self.results
        } else {
            &self.filtered_results
        };

        let start = self.current_page * self.items_per_page;
        let end = ((self.current_page + 1) * self.items_per_page).min(results.len());

        if start >= results.len() {
            return Vec::new();
        }

        results[start..end].iter().collect()
    }

    pub fn total_pages(&self) -> usize {
        let count = if self.filtered_results.is_empty() {
            self.results.len()
        } else {
            self.filtered_results.len()
        };

        (count + self.items_per_page - 1) / self.items_per_page
    }

    pub fn next_page(&mut self) {
        if self.current_page + 1 < self.total_pages() {
            self.current_page += 1;
            self.selected_index = Some(0);
        }
    }

    pub fn previous_page(&mut self) {
        if self.current_page > 0 {
            self.current_page -= 1;
            self.selected_index = Some(0);
        }
    }

    // Navigation
    pub fn next(&mut self) {
        let page_items = self.get_paginated_results();
        if page_items.is_empty() {
            return;
        }

        let i = match self.selected_index {
            Some(i) if i + 1 < page_items.len() => i + 1,
            Some(_) => {
                // Try next page
                if self.current_page + 1 < self.total_pages() {
                    self.next_page();
                    return;
                }
                0 // Wrap to first item
            }
            None => 0,
        };
        self.selected_index = Some(i);
    }

    pub fn previous(&mut self) {
        let page_items = self.get_paginated_results();
        if page_items.is_empty() {
            return;
        }

        let i = match self.selected_index {
            Some(0) => {
                // Try previous page
                if self.current_page > 0 {
                    self.previous_page();
                    // Select last item on previous page
                    let prev_items = self.get_paginated_results();
                    if !prev_items.is_empty() {
                        self.selected_index = Some(prev_items.len() - 1);
                    }
                    return;
                }
                page_items.len() - 1 // Wrap to last item
            }
            Some(i) => i - 1,
            None => page_items.len() - 1,
        };
        self.selected_index = Some(i);
    }

    pub fn get_selected_package(&self) -> Option<&Package> {
        self.selected_index
            .and_then(|idx| self.get_paginated_results().get(idx).copied())
    }

    pub fn get_current_package(&self) -> Option<&Package> {
        self.get_selected_package()
    }

    // Filtering and Sorting
    pub fn apply_filter_and_sort(&mut self) {
        let mut filtered: Vec<Package> = self.results.clone();

        // Apply filter
        filtered.retain(|pkg| match self.current_filter {
            FilterOption::All => true,
            FilterOption::Installed => pkg.is_installed,
            FilterOption::NotInstalled => !pkg.is_installed,
            FilterOption::RepoOnly => matches!(pkg.source, crate::models::PackageSource::Pacman),
            FilterOption::AurOnly => matches!(pkg.source, crate::models::PackageSource::Aur),
        });

        // Apply sort
        filtered.sort_by(|a, b| match self.current_sort {
            SortOption::NameAsc => a.name.cmp(&b.name),
            SortOption::NameDesc => b.name.cmp(&a.name),
            SortOption::Source => {
                let a_val = if matches!(a.source, crate::models::PackageSource::Pacman) {
                    0
                } else {
                    1
                };
                let b_val = if matches!(b.source, crate::models::PackageSource::Pacman) {
                    0
                } else {
                    1
                };
                a_val.cmp(&b_val).then_with(|| a.name.cmp(&b.name))
            }
        });

        self.filtered_results = filtered;
        self.current_page = 0;
        self.selected_index = if self.filtered_results.is_empty() {
            None
        } else {
            Some(0)
        };
    }

    pub fn cycle_filter(&mut self) {
        self.current_filter = match self.current_filter {
            FilterOption::All => FilterOption::Installed,
            FilterOption::Installed => FilterOption::NotInstalled,
            FilterOption::NotInstalled => FilterOption::RepoOnly,
            FilterOption::RepoOnly => FilterOption::AurOnly,
            FilterOption::AurOnly => FilterOption::All,
        };
        self.apply_filter_and_sort();
    }

    pub fn cycle_sort(&mut self) {
        self.current_sort = match self.current_sort {
            SortOption::NameAsc => SortOption::NameDesc,
            SortOption::NameDesc => SortOption::Source,
            SortOption::Source => SortOption::NameAsc,
        };
        self.apply_filter_and_sort();
    }

    // View Management
    pub fn show_package_details(&mut self) {
        if self.selected_index.is_some() {
            self.show_package_details = true;
        }
    }

    pub fn hide_package_details(&mut self) {
        self.show_package_details = false;
    }

    pub fn show_dependency_visualization(&mut self) {
        if self.selected_index.is_some() {
            self.show_dependency_visualization = true;
        }
    }

    pub fn hide_dependency_visualization(&mut self) {
        self.show_dependency_visualization = false;
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    pub fn hide_help(&mut self) {
        self.show_help = false;
    }

    // Console Management
    pub fn add_console_output(&mut self, line: String) {
        // Parse progress before pushing to avoid borrow issues
        let line_clone = line.clone();
        self.console_buffer.push(line);
        self.parse_progress(&line_clone);

        // Keep buffer size manageable
        if self.console_buffer.len() > 1000 {
            self.console_buffer.remove(0);
        }
    }

    fn parse_progress(&mut self, line: &str) {
        // Try to detect package installation progress
        if line.contains("(1/") {
            // Parse "(1/10)" format
            if let Some(start) = line.find('(') {
                if let Some(end) = line.find(')') {
                    let progress = &line[start + 1..end];
                    let parts: Vec<&str> = progress.split('/').collect();
                    if parts.len() == 2 {
                        if let (Ok(current), Ok(total)) = (
                            parts[0].trim().parse::<usize>(),
                            parts[1].trim().parse::<usize>(),
                        ) {
                            self.command_progress = Some(CommandProgress {
                                current,
                                total,
                                current_package: line
                                    .split_whitespace()
                                    .next()
                                    .unwrap_or("")
                                    .to_string(),
                                status: ProgressStatus::Installing,
                            });
                        }
                    }
                }
            }
        }
    }

    pub fn clear_console(&mut self) {
        self.console_buffer.clear();
        self.command_progress = None;
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_package(
        name: &str,
        source: crate::models::PackageSource,
        installed: bool,
    ) -> Package {
        Package {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            description: format!("Test package {}", name),
            source,
            is_installed: installed,
            installed_size: None,
            download_size: None,
            groups: vec![],
            licenses: vec![],
            maintainers: vec![],
            keywords: vec![],
            url: None,
            depends_on: vec![],
            required_by: vec![],
            opt_depends: vec![],
            conflicts: vec![],
            replaces: vec![],
            provides: vec![],
        }
    }

    #[test]
    fn test_search_history() {
        let mut app = App::new();

        app.add_to_history("firefox".to_string());
        app.add_to_history("vlc".to_string());
        app.add_to_history("firefox".to_string()); // Duplicate should move to front

        assert_eq!(app.search_history.len(), 2);
        assert_eq!(app.search_history[0], "firefox");

        app.navigate_history_up();
        assert_eq!(app.search_input, "firefox");
    }

    #[test]
    fn test_selection_undo() {
        let mut app = App::new();
        let pkg = create_test_package("test", crate::models::PackageSource::Pacman, false);

        app.results.push(pkg.clone());
        app.selected_index = Some(0);

        // Select
        app.toggle_selection();
        assert_eq!(app.selected_packages.len(), 1);

        // Undo
        app.undo_last_selection();
        assert_eq!(app.selected_packages.len(), 0);
    }

    #[test]
    fn test_pagination() {
        let mut app = App::new();
        app.items_per_page = 2;

        // Add 5 test packages
        for i in 0..5 {
            app.results.push(create_test_package(
                &format!("pkg{}", i),
                crate::models::PackageSource::Pacman,
                false,
            ));
        }

        assert_eq!(app.total_pages(), 3);

        let page1 = app.get_paginated_results();
        assert_eq!(page1.len(), 2);

        app.next_page();
        let page2 = app.get_paginated_results();
        assert_eq!(page2.len(), 2);

        app.next_page();
        let page3 = app.get_paginated_results();
        assert_eq!(page3.len(), 1);
    }

    #[test]
    fn test_filtering() {
        let mut app = App::new();

        app.results.push(create_test_package(
            "installed-pkg",
            crate::models::PackageSource::Pacman,
            true,
        ));
        app.results.push(create_test_package(
            "not-installed",
            crate::models::PackageSource::Pacman,
            false,
        ));
        app.results.push(create_test_package(
            "aur-pkg",
            crate::models::PackageSource::Aur,
            false,
        ));

        app.current_filter = FilterOption::Installed;
        app.apply_filter_and_sort();
        assert_eq!(app.filtered_results.len(), 1);

        app.current_filter = FilterOption::AurOnly;
        app.apply_filter_and_sort();
        assert_eq!(app.filtered_results.len(), 1);
        assert_eq!(app.filtered_results[0].name, "aur-pkg");
    }
}
