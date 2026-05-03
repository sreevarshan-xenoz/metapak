//! Application state management for Arch TUI
//!
//! This module contains the main application state and logic for managing
//! the TUI application's state, including search results, selections,
//! and UI modes.

use std::collections::{HashMap, VecDeque};
use crate::constants::ui::CONSOLE_BUFFER_MAX_LINES;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::UnboundedSender;

use crate::action::Action;
use crate::animations::Toast;
use crate::config::AppConfig;

use crate::models::Package;
use crate::models::OutdatedPackage;
use crate::services::CommandSpec;
use crate::transaction_history::TransactionRecord;
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
    Group(String),
}

impl Default for FilterOption {
    fn default() -> Self {
        Self::All
    }
}

/// Sort options
#[derive(Debug, Clone, PartialEq)]
pub enum SortOption {
    NameAsc,
    NameDesc,
    Source,
    SizeAsc,
    SizeDesc,
    Group,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UpdatesSortOption {
    NameAsc,
    NameDesc,
    SizeAsc,
    SizeDesc,
    Repository,
    SecurityFirst,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UpdatesFilter {
    All,
    SecurityOnly,
    Repository(String),
    AurOnly,
}

impl Default for UpdatesSortOption {
    fn default() -> Self {
        Self::SecurityFirst
    }
}

impl Default for UpdatesFilter {
    fn default() -> Self {
        Self::All
    }
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
    pub immediate_search: Option<String>,

    // State
    pub should_quit: bool,
    pub is_loading: bool,
    pub action_tx: Option<UnboundedSender<crate::action::Action>>,
    pub selected_index: Option<usize>,
    pub pending_command: Option<(String, Vec<String>)>,
    pub error_message: Option<String>,
    pub available_updates: Option<usize>,
    pub is_operation_running: bool,

    // Updates View
    pub show_updates_view: bool,
    pub outdated_packages: Vec<OutdatedPackage>,
    pub updates_cursor: Option<usize>,
    pub updates_sort: UpdatesSortOption,
    pub updates_filter: UpdatesFilter,
    pub updates_group_by_repo: bool,
    pub selected_updates: Vec<String>,
    pub updates_changelog_package: Option<String>,
    pub partial_update_warning_shown: bool,

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
    pub confirmation_commands: Vec<CommandSpec>,

    // Console
    pub show_console: bool,
    pub console_buffer: Vec<String>,
    pub command_stdin_tx: Option<UnboundedSender<String>>,
    pub console_input: String,
    pub command_progress: Option<CommandProgress>,

    // Install progress tracking
    pub install_total: usize,
    pub install_current: usize,
    pub install_current_package: String,

    // Configuration
    pub config: AppConfig,
    pub theme: crate::theme::Theme,

    // Views
    pub show_package_details: bool,
    pub show_dependency_visualization: bool,
    pub dependency_tree_text: Option<String>,
    pub show_help: bool,
    pub show_history: bool,
    pub show_diagnostics: bool,
    pub diagnostics: Vec<crate::diagnostics::DiagnosticItem>,
    pub show_system_info: bool,
    pub system_info: Vec<crate::diagnostics::DiagnosticItem>,
    pub show_orphans: bool,
    pub orphan_packages: Vec<crate::diagnostics::OrphanPackage>,
    pub show_package_sizes: bool,
    pub package_sizes: Vec<crate::diagnostics::PackageSize>,
    pub show_cache: bool,
    pub cache_info: Vec<crate::diagnostics::CacheInfo>,

    // Localization
    pub localizer: crate::i18n::Localizer,

    // Filter and Sort
    pub current_filter: FilterOption,
    pub current_sort: SortOption,

    // Transaction history
    pub transaction_history: VecDeque<TransactionRecord>,
    pub current_transaction: Option<TransactionRecord>,

    // Visual overhaul - sidebar, animations, toasts, scroll states
    pub show_sidebar: bool,
    pub animation_state: crate::animations::AnimationState,
    pub toasts: Vec<Toast>,
    pub results_scroll_state: ratatui::widgets::ScrollbarState,
    pub history_scroll_state: Option<ratatui::widgets::ScrollbarState>,
    pub dependency_scroll_state: Option<ratatui::widgets::ScrollbarState>,
    pub console_scroll_state: Option<ratatui::widgets::ScrollbarState>,
    pub diagnostics_scroll_state: Option<ratatui::widgets::ScrollbarState>,

    // Fuzzy search
    pub fuzzy_matcher: crate::search::FuzzySearch,
    pub fuzzy_scores: std::collections::HashMap<String, i64>,
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
            search_debounce_duration: Duration::from_millis(50),
            pending_search: None,
            immediate_search: None,

            should_quit: false,
            is_loading: false,
            action_tx: None,
            selected_index: None,
            pending_command: None,
            error_message: None,
            available_updates: None,
            is_operation_running: false,

            // Updates View
            show_updates_view: false,
            outdated_packages: Vec::new(),
            updates_cursor: None,
            updates_sort: UpdatesSortOption::default(),
            updates_filter: UpdatesFilter::default(),
            updates_group_by_repo: true,
            selected_updates: Vec::new(),
            updates_changelog_package: None,
            partial_update_warning_shown: false,

            show_password_prompt: true,
            password_input: PasswordInput::new(),

            selected_packages: HashMap::new(),
            selection_history: VecDeque::new(),
            max_undo_history: 20,

            show_confirm_prompt: false,
            packages_pending_confirmation: Vec::new(),
            confirmation_commands: Vec::new(),

            show_console: false,
            console_buffer: VecDeque::new(),
            command_stdin_tx: None,
            console_input: String::new(),
            command_progress: None,

            install_total: 0,
            install_current: 0,
            install_current_package: String::new(),

            config: AppConfig::default(),
            theme: crate::theme::Theme::default(),

            show_package_details: false,
            show_dependency_visualization: false,
            dependency_tree_text: None,
            show_help: false,
            show_history: false,
            show_diagnostics: false,
            diagnostics: Vec::new(),
            show_system_info: false,
            system_info: Vec::new(),
            show_orphans: false,
            orphan_packages: Vec::new(),
            show_package_sizes: false,
            package_sizes: Vec::new(),
            show_cache: false,
            cache_info: Vec::new(),

            localizer: crate::i18n::Localizer::new(),

            current_filter: FilterOption::All,
            current_sort: SortOption::NameAsc,

            transaction_history: VecDeque::new(),
            current_transaction: None,

            show_sidebar: false,
            animation_state: crate::animations::AnimationState::new(),
            toasts: Vec::new(),
            results_scroll_state: ratatui::widgets::ScrollbarState::new(0),
            history_scroll_state: Some(ratatui::widgets::ScrollbarState::new(0)),
            dependency_scroll_state: Some(ratatui::widgets::ScrollbarState::new(0)),
            console_scroll_state: Some(ratatui::widgets::ScrollbarState::new(0)),
            diagnostics_scroll_state: Some(ratatui::widgets::ScrollbarState::new(0)),

            fuzzy_matcher: crate::search::FuzzySearch::new(),
            fuzzy_scores: std::collections::HashMap::new(),
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
        if let Some(idx) = self.history_index {
            if idx == 0 {
                self.history_index = None;
                self.search_input.clear();
            } else {
                self.history_index = Some(idx - 1);
                if let Some(query) = self.search_history.get(idx - 1) {
                    self.search_input = query.clone();
                }
            }
        }
    }

    // Debounced Search
    pub fn trigger_search(&mut self, query: String) {
        self.pending_search = Some(query.clone());
        self.last_search_time = Some(Instant::now());
    }

    /// Execute search immediately (bypass debounce - used when user presses Enter)
    pub fn execute_search_now(&mut self, query: String) {
        self.immediate_search = Some(query);
    }

    pub fn should_execute_search(&self) -> Option<String> {
        if let (Some(query), Some(last_time)) = (&self.pending_search, self.last_search_time) {
            if last_time.elapsed() >= self.search_debounce_duration {
                return Some(query.clone());
            }
        }
        None
    }

    /// Get search suggestions based on current input
    pub fn get_search_suggestions(&self, input: &str, limit: usize) -> Vec<String> {
        if input.is_empty() {
            return self.search_history.iter().take(limit).cloned().collect();
        }
        
        let input_lower = input.to_lowercase();
        self.search_history
            .iter()
            .filter(|q| q.to_lowercase().contains(&input_lower))
            .take(limit)
            .cloned()
            .collect()
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

        count.div_ceil(self.items_per_page)
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
            FilterOption::Group(ref g) => pkg.groups.iter().any(|gr| gr == g),
        });

        // Apply fuzzy scoring for search
        self.fuzzy_scores.clear();
        if !self.search_input.is_empty() {
            let items: Vec<(String, String)> = filtered
                .iter()
                .map(|p| (p.name.clone(), p.description.clone()))
                .collect();

            let results = self
                .fuzzy_matcher
                .filter_and_sort(&items, &self.search_input);
            for (name, score, _) in results {
                self.fuzzy_scores.insert(name.to_string(), score);
            }
        }

        // Apply sort (fuzzy score first if searching)
        filtered.sort_by(|a, b| {
            if !self.search_input.is_empty() {
                let a_score = self.fuzzy_scores.get(&a.name).copied().unwrap_or(0);
                let b_score = self.fuzzy_scores.get(&b.name).copied().unwrap_or(0);
                if a_score != b_score {
                    return b_score.cmp(&a_score);
                }
            }

            match self.current_sort {
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
                SortOption::SizeAsc => a.installed_size.cmp(&b.installed_size),
                SortOption::SizeDesc => b.installed_size.cmp(&a.installed_size),
                SortOption::Group => {
                    let a_groups = a.groups.join("");
                    let b_groups = b.groups.join("");
                    a_groups.cmp(&b_groups).then_with(|| a.name.cmp(&b.name))
                }
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
            FilterOption::Group(_) => FilterOption::All,
        };
        self.apply_filter_and_sort();
    }

    pub fn get_available_groups(&self) -> Vec<String> {
        let mut groups: Vec<String> = self.results
            .iter()
            .flat_map(|p| p.groups.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        groups.sort();
        groups
    }

    pub fn filter_by_group(&mut self, group: &str) {
        self.current_filter = FilterOption::Group(group.to_string());
        self.apply_filter_and_sort();
    }

    pub fn cycle_sort(&mut self) {
        self.current_sort = match self.current_sort {
            SortOption::NameAsc => SortOption::NameDesc,
            SortOption::NameDesc => SortOption::Source,
            SortOption::Source => SortOption::SizeDesc,
            SortOption::SizeDesc => SortOption::SizeAsc,
            SortOption::SizeAsc => SortOption::Group,
            SortOption::Group => SortOption::NameAsc,
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
        if let Some(pkg) = self.get_selected_package().cloned() {
            let (tree, warnings) =
                crate::dependency_visualization::DependencyVisualizationService::build_dependency_tree_safe(
                    &pkg, 3,
                );
            let mut text =
                crate::dependency_visualization::DependencyVisualizationService::format_tree(
                    &tree, 0, true, true,
                );
            if !warnings.is_empty() {
                text.push_str("\nWarnings:\n");
                for warning in warnings.iter().take(5) {
                    text.push_str(&format!("- {}\n", warning));
                }
                if warnings.len() > 5 {
                    text.push_str(&format!(
                        "- ...and {} more warning(s)\n",
                        warnings.len() - 5
                    ));
                }
            }
            self.dependency_tree_text = Some(text);
            self.show_dependency_visualization = true;
        }
    }

    pub fn hide_dependency_visualization(&mut self) {
        self.show_dependency_visualization = false;
        self.dependency_tree_text = None;
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    pub fn hide_help(&mut self) {
        self.show_help = false;
    }

    pub fn toggle_history(&mut self) {
        self.show_history = !self.show_history;
    }

    pub fn toggle_diagnostics(&mut self) {
        self.show_diagnostics = !self.show_diagnostics;
    }

    pub fn toggle_system_info(&mut self) {
        if !self.show_system_info {
            self.system_info = crate::diagnostics::get_system_info();
        }
        self.show_system_info = !self.show_system_info;
    }

    pub fn toggle_orphans(&mut self) {
        if !self.show_orphans {
            self.orphan_packages = crate::diagnostics::find_orphan_packages();
        }
        self.show_orphans = !self.show_orphans;
    }

    pub fn toggle_package_sizes(&mut self) {
        if !self.show_package_sizes {
            self.package_sizes = crate::diagnostics::get_package_sizes();
            self.package_sizes.truncate(30);
        }
        self.show_package_sizes = !self.show_package_sizes;
    }

    pub fn toggle_cache(&mut self) {
        if !self.show_cache {
            self.cache_info = crate::diagnostics::get_cache_info();
        }
        self.show_cache = !self.show_cache;
    }

    pub fn toggle_updates_view(&mut self) {
        self.show_updates_view = !self.show_updates_view;
        if self.show_updates_view {
            self.hide_package_details();
            self.hide_dependency_visualization();
            self.hide_help();
        }
    }

    pub fn hide_updates_view(&mut self) {
        self.show_updates_view = false;
    }

    pub fn get_filtered_outdated_packages(&self) -> Vec<&OutdatedPackage> {
        let mut packages = self.outdated_packages.iter().collect::<Vec<_>>();

        packages.sort_by(|a, b| {
            match self.updates_sort {
                UpdatesSortOption::NameAsc => a.name.cmp(&b.name),
                UpdatesSortOption::NameDesc => b.name.cmp(&a.name),
                UpdatesSortOption::SizeAsc => a.download_size.cmp(&b.download_size),
                UpdatesSortOption::SizeDesc => b.download_size.cmp(&a.download_size),
                UpdatesSortOption::Repository => a.repository.cmp(&b.repository),
                UpdatesSortOption::SecurityFirst => {
                    match (a.is_security_update, b.is_security_update) {
                        (true, false) => std::cmp::Ordering::Less,
                        (false, true) => std::cmp::Ordering::Greater,
                        _ => a.name.cmp(&b.name),
                    }
                }
            }
        });

        if !matches!(self.updates_filter, UpdatesFilter::All) {
            packages.retain(|p| match &self.updates_filter {
                UpdatesFilter::SecurityOnly => p.is_security_update,
                UpdatesFilter::Repository(repo) => &p.repository == repo,
                UpdatesFilter::AurOnly => p.is_aur,
                UpdatesFilter::All => true,
            });
        }

        packages
    }

    pub fn toggle_update_selection(&mut self, idx: usize) {
        if idx >= self.outdated_packages.len() {
            return;
        }
        let pkg = &mut self.outdated_packages[idx];
        pkg.is_selected = !pkg.is_selected;

        if pkg.is_selected {
            if !self.selected_updates.contains(&pkg.name) {
                self.selected_updates.push(pkg.name.clone());
            }
        } else {
            self.selected_updates.retain(|n| n != &pkg.name);
        }
    }

    pub fn select_all_updates(&mut self) {
        for pkg in &mut self.outdated_packages {
            pkg.is_selected = true;
        }
        self.selected_updates = self.outdated_packages.iter().map(|p| p.name.clone()).collect();
    }

    pub fn deselect_all_updates(&mut self) {
        for pkg in &mut self.outdated_packages {
            pkg.is_selected = false;
        }
        self.selected_updates.clear();
    }

    pub fn get_selected_outdated_packages(&self) -> Vec<&OutdatedPackage> {
        self.outdated_packages.iter().filter(|p| p.is_selected).collect()
    }

    pub fn get_total_update_size(&self) -> u64 {
        self.outdated_packages.iter().map(|p| p.download_size).sum()
    }

    pub fn get_selected_update_size(&self) -> u64 {
        self.get_selected_outdated_packages().iter().map(|p| p.download_size).sum()
    }

    pub fn has_aur_needing_rebuild(&self) -> bool {
        self.outdated_packages.iter().any(|p| p.needs_rebuild)
    }

    pub fn get_security_updates_count(&self) -> usize {
        self.outdated_packages.iter().filter(|p| p.is_security_update).count()
    }

    pub fn get_aur_updates_count(&self) -> usize {
        self.outdated_packages.iter().filter(|p| p.is_aur).count()
    }

    pub fn get_repo_updates_count(&self, repo: &str) -> usize {
        self.outdated_packages.iter().filter(|p| p.repository == repo).count()
    }

    pub fn show_changelog_for_package(&mut self, name: String) {
        self.updates_changelog_package = Some(name);
    }

    pub fn hide_changelog(&mut self) {
        self.updates_changelog_package = None;
    }

    // Console Management
    pub fn add_console_output(&mut self, line: String) {
        let line_clone = line.clone();
        self.console_buffer.push_back(line);
        self.parse_progress(&line_clone);

        if self.console_buffer.len() > CONSOLE_BUFFER_MAX_LINES {
            self.console_buffer.pop_front();
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
                            });
                        }
                    }
                }
            }
        }
    }

    pub fn clear_console(&mut self) {
        self.console_buffer.clear();
        self.console_input.clear();
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
        let mut pkg = Package::new(name, "1.0.0");
        pkg.source = source;
        pkg.is_installed = installed;
        pkg
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

impl App {
    pub fn add_toast(&mut self, message: String, style: crate::animations::ToastStyle) {
        let truncated = if message.chars().count() > 60 {
            let truncated: String = message.chars().take(57).collect();
            format!("{}...", truncated)
        } else {
            message
        };

        self.toasts
            .push(crate::animations::Toast::new(truncated, style));

        if self.toasts.len() > 3 {
            self.toasts.remove(0);
        }
    }

    pub fn start_install_progress(&mut self, total: usize) {
        self.install_total = total;
        self.install_current = 0;
        self.install_current_package = String::new();
    }

    pub fn update_install_progress(&mut self, current: usize, package_name: &str) {
        self.install_current = current;
        self.install_current_package = package_name.to_string();
    }

    pub fn finish_install_progress(&mut self) {
        self.install_total = 0;
        self.install_current = 0;
        self.install_current_package.clear();
    }

    pub fn get_progress_percentage(&self) -> f64 {
        if self.install_total == 0 {
            0.0
        } else {
            (self.install_current as f64 / self.install_total as f64) * 100.0
        }
    }

    pub fn expire_toasts(&mut self) {
        self.toasts.retain(|t| !t.is_expired());
    }

    pub fn toggle_sidebar(&mut self) {
        self.show_sidebar = !self.show_sidebar;
        if self.show_sidebar && self.selected_index.is_none() && !self.results.is_empty() {
            self.selected_index = Some(0);
        }
    }

    pub fn tick(&mut self, delta_ms: u64) {
        self.animation_state.tick(delta_ms);
        self.expire_toasts();

        self.results_scroll_state = self
            .results_scroll_state
            .position(self.get_paginated_results().len());
    }
}
