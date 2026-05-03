use crate::app::{App, InputMode};
use crossterm::event::{Event, KeyCode, KeyEventKind};

pub fn handle_event(app: &mut App, event: Event) {
    // Handle mouse events
    if let Event::Mouse(mouse) = event {
        handle_mouse_event(app, mouse);
        return;
    }

    if let Event::Key(key) = event {
        // Only handle key press events, not release or repeat
        if key.kind != KeyEventKind::Press {
            return;
        }

        // Global: Help screen
        if app.show_help {
            if key.code == KeyCode::Esc || key.code == KeyCode::Char('?') {
                app.hide_help();
            }
            return;
        }

        // Updates View
        if app.show_updates_view {
            match key.code {
                KeyCode::Esc => app.hide_updates_view(),
                KeyCode::Char('a') => app.select_all_updates(),
                KeyCode::Char('n') => app.deselect_all_updates(),
                KeyCode::Enter => {
                    if app.selected_updates.is_empty() {
                        app.error_message = Some("No packages selected".to_string());
                    } else {
                        // Start update process for selected packages
                    }
                }
                KeyCode::Char(' ') | KeyCode::Char('s') => {
                    if let Some(cursor) = app.updates_cursor {
                        app.toggle_update_selection(cursor);
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if let Some(current) = app.updates_cursor {
                        if current > 0 {
                            app.updates_cursor = Some(current - 1);
                        }
                    } else {
                        app.updates_cursor = Some(0);
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let max = app.outdated_packages.len().saturating_sub(1);
                    if let Some(current) = app.updates_cursor {
                        if current < max {
                            app.updates_cursor = Some(current + 1);
                        }
                    } else {
                        app.updates_cursor = Some(0);
                    }
                }
                _ => {}
            }
            return;
        }

        // Global: Transaction History
        if app.show_history {
            match key.code {
                KeyCode::Esc | KeyCode::Char('t') => app.show_history = false,
                KeyCode::Char('R') => trigger_rollback(app),
                _ => {}
            }
            return;
        }

        if app.show_diagnostics {
            if key.code == KeyCode::Esc || key.code == KeyCode::Char('h') {
                app.show_diagnostics = false;
            }
            return;
        }

        if app.show_system_info {
            if key.code == KeyCode::Esc || key.code == KeyCode::Char('I') {
                app.show_system_info = false;
            }
            return;
        }

        if app.show_orphans {
            if key.code == KeyCode::Esc || key.code == KeyCode::Char('O') {
                app.show_orphans = false;
            }
            return;
        }

        if app.show_package_sizes {
            if key.code == KeyCode::Esc || key.code == KeyCode::Char('P') {
                app.show_package_sizes = false;
            }
            return;
        }

        if app.show_cache {
            if key.code == KeyCode::Esc || key.code == KeyCode::Char('C') {
                app.show_cache = false;
            }
            return;
        }

        if app.show_foreign {
            if key.code == KeyCode::Esc || key.code == KeyCode::Char('F') {
                app.show_foreign = false;
            }
            return;
        }

        if app.show_groups {
            if key.code == KeyCode::Esc || key.code == KeyCode::Char('G') {
                app.show_groups = false;
                app.selected_group = None;
            }
            return;
        }

        // Global: Password Prompt Handling
        if app.show_password_prompt {
            match key.code {
                KeyCode::Enter if !app.password_input.is_empty() => {
                    if let Some(tx) = &app.action_tx {
                        let password = crate::utils::PasswordInput::from_string(
                            app.password_input.expose_secret().to_string(),
                        );
                        let _ = tx.send(crate::action::Action::new(crate::action::ActionInner::InitSudo(
                            password.get_secret().clone(),
                        )));
                    }
                    app.is_loading = true;
                }
                KeyCode::Char(c) => {
                    app.password_input.push(c);
                }
                KeyCode::Backspace => {
                    app.password_input.pop();
                }
                KeyCode::Esc => {
                    app.should_quit = true;
                }
                _ => {}
            }
            return;
        }

        // Global: Console handling
        if app.show_console {
            match key.code {
                KeyCode::Esc => {
                    app.show_console = false;
                }
                KeyCode::Char('q') if app.command_stdin_tx.is_none() => {
                    app.show_console = false;
                }
                KeyCode::Enter => {
                    if let Some(tx) = &app.command_stdin_tx {
                        let line = app.console_input.clone();
                        if tx.send(line.clone()).is_ok() {
                            app.add_console_output(format!("> {}", line));
                        }
                        app.console_input.clear();
                    }
                }
                KeyCode::Backspace => {
                    app.console_input.pop();
                }
                KeyCode::Char(c) if app.command_stdin_tx.is_some() => {
                    app.console_input.push(c);
                }
                _ => {}
            }
            return;
        }

        // Global: Package Details
        if app.show_package_details {
            match key.code {
                KeyCode::Esc => app.hide_package_details(),
                KeyCode::Char('j') | KeyCode::Down => {} // Could add scrolling
                KeyCode::Char('k') | KeyCode::Up => {}
                _ => {}
            }
            return;
        }

        // Global: Dependency Visualization
        if app.show_dependency_visualization {
            match key.code {
                KeyCode::Esc => app.hide_dependency_visualization(),
                KeyCode::Char('j') | KeyCode::Down => {} // Could add scrolling
                KeyCode::Char('k') | KeyCode::Up => {}
                _ => {}
            }
            return;
        }

        // Confirmation Popup
        if app.show_confirm_prompt {
            match key.code {
                KeyCode::Char('y') | KeyCode::Enter => {
                    app.show_confirm_prompt = false;
                    app.show_console = true;
                    app.clear_console();

                    let packages = std::mem::take(&mut app.packages_pending_confirmation);
                    if !packages.is_empty() {
                        execute_confirmation_action(app, &packages);
                    }
                }
                KeyCode::Char('n') | KeyCode::Esc => {
                    app.show_confirm_prompt = false;
                    app.packages_pending_confirmation.clear();
                    app.confirmation_commands.clear();
                }
                _ => {}
            }
            return;
        }

        // Robustness: Simulation Modal
        if app.show_simulation {
            match key.code {
                KeyCode::Enter => {
                    app.show_simulation = false;
                    // Proceed with original confirmation logic if needed
                }
                KeyCode::Esc => {
                    app.show_simulation = false;
                    app.simulation_result = None;
                }
                _ => {}
            }
            return;
        }

        // Robustness: Rollback Dialog
        if app.show_rollback_confirm {
            match key.code {
                KeyCode::Char('y') | KeyCode::Enter => {
                    if let Some(rollback_id) = app.pending_rollback_id.take() {
                        if let Some(tx) = &app.action_tx {
                            let _ = tx.send(crate::action::Action::new(
                                crate::action::ActionInner::Rollback(rollback_id),
                            ));
                        }
                    }
                    app.show_rollback_confirm = false;
                }
                KeyCode::Char('n') | KeyCode::Esc => {
                    app.show_rollback_confirm = false;
                    app.pending_rollback_id = None;
                }
                _ => {}
            }
            return;
        }

        // Main input handling based on mode
        match app.input_mode {
            InputMode::Normal => handle_normal_mode(app, key.code),
            InputMode::Editing => handle_editing_mode(app, key.code),
        }
    }
}

fn handle_normal_mode(app: &mut App, key: KeyCode) {
    match key {
        // Quit
        KeyCode::Char('q') => app.should_quit = true,

        // Help
        KeyCode::Char('?') => app.toggle_help(),
        KeyCode::Char('t') => app.toggle_history(),
        KeyCode::Char('h') => {
            app.diagnostics = crate::diagnostics::run_diagnostics();
            app.toggle_diagnostics();
        }
        KeyCode::Char('I') => app.toggle_system_info(),
        KeyCode::Char('O') => app.toggle_orphans(),
        KeyCode::Char('P') => app.toggle_package_sizes(),
        KeyCode::Char('C') => app.toggle_cache(),
        KeyCode::Char('F') => app.toggle_foreign(),
        KeyCode::Char('G') => app.toggle_groups(),

        // Search
        KeyCode::Char('/') | KeyCode::Char('i') => {
            app.input_mode = InputMode::Editing;
        }

        // Navigation
        KeyCode::Down | KeyCode::Char('j') => app.next(),
        KeyCode::Up | KeyCode::Char('k') => app.previous(),

        // Pagination
        KeyCode::Char('n') => app.next_page(),
        KeyCode::Char('p') => app.previous_page(),
        KeyCode::Home => {
            app.current_page = 0;
            app.selected_index = Some(0);
        }
        KeyCode::End => {
            app.current_page = app.total_pages().saturating_sub(1);
            app.selected_index = Some(0);
        }

        // Selection
        KeyCode::Tab => app.toggle_selection(),
        KeyCode::Char('u') => app.undo_last_selection(),

        // Filter and Sort
        KeyCode::Char('f') => app.cycle_filter(),
        KeyCode::Char('s') => app.cycle_sort(),

        // Actions
        KeyCode::Enter => {
            if app.is_operation_running {
                app.error_message = Some("An operation is already running.".to_string());
                return;
            }
            if !app.selected_packages.is_empty() {
                app.packages_pending_confirmation =
                    app.selected_packages.values().cloned().collect();
                app.confirmation_commands = crate::services::plan_package_transaction(
                    &app.packages_pending_confirmation,
                    &app.config,
                );
                app.show_confirm_prompt = true;
            } else if let Some(pkg) = app.get_selected_package() {
                app.packages_pending_confirmation = vec![pkg.clone()];
                app.confirmation_commands = crate::services::plan_package_transaction(
                    &app.packages_pending_confirmation,
                    &app.config,
                );
                app.show_confirm_prompt = true;
            }
        }

        // System Update
        KeyCode::Char('U') => {
            if app.is_operation_running {
                app.error_message = Some("An operation is already running.".to_string());
                return;
            }
            app.show_console = true;
            app.clear_console();
            app.is_operation_running = true;
            if let Some(tx) = &app.action_tx {
                let _ = tx.send(crate::action::Action::new(crate::action::ActionInner::SystemUpdate));
            }
        }

        // Rollback last successful transaction
        KeyCode::Char('R') => {
            trigger_rollback(app);
        }

        // Package Details
        KeyCode::Char('d') => app.show_package_details(),

        // Dependency Visualization
        KeyCode::Char('v') => app.show_dependency_visualization(),

        // Open in browser
        KeyCode::Char('o') => {
            if let Some(pkg) = app.get_selected_package() {
                let url = match pkg.source {
                    crate::models::PackageSource::Pacman => {
                        format!("https://archlinux.org/packages/?q={}", pkg.name)
                    }
                    crate::models::PackageSource::Aur => {
                        format!("https://aur.archlinux.org/packages/{}/", pkg.name)
                    }
                    _ => "https://archlinux.org/".to_string(),
                };
                if let Err(e) = open::that(&url) {
                    app.error_message = Some(format!("Failed to open browser: {}", e));
                }
            }
        }

        // Refresh/Clear
        KeyCode::Char('r') => {
            app.results.clear();
            app.filtered_results.clear();
            app.search_input.clear();
            app.selected_packages.clear();
            app.selected_index = None;
        }

        // Cancel operation
        KeyCode::Char('c') if key == KeyCode::Char('c') => {
            if let Some(tx) = &app.action_tx {
                let _ = tx.send(crate::action::Action::new(crate::action::ActionInner::CancelOperation));
            }
        }

        _ => {}
    }
}

fn trigger_rollback(app: &mut App) {
    if app.is_operation_running {
        app.error_message = Some("An operation is already running.".to_string());
        return;
    }
    if let Some(last) = app
        .transaction_history
        .iter()
        .find(|tx| tx.status == crate::transaction_history::TransactionStatus::Success)
    {
        let commands = crate::services::plan_rollback_transaction(
            &last.installed_packages,
            &last.removed_packages,
            &app.config,
        );
        if commands.is_empty() {
            app.error_message =
                Some("No rollback plan available for last transaction.".to_string());
            return;
        }
        app.show_history = false;
        app.show_console = true;
        app.clear_console();
        app.is_operation_running = true;
        if let Some(tx) = &app.action_tx {
            let _ = tx.send(crate::action::Action::new(crate::action::ActionInner::RunCommands(commands)));
        }
    } else {
        app.error_message = Some("No successful transaction found to rollback.".to_string());
    }
}

fn handle_editing_mode(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Esc => {
            app.input_mode = InputMode::Normal;
            app.history_index = None;
        }

        KeyCode::Enter => {
            app.input_mode = InputMode::Normal;
            let query = app.search_input.trim().to_string();
            if !query.is_empty() {
                app.execute_search_now(query);
            }
        }

        KeyCode::Char(c) => {
            app.search_input.push(c);
            app.history_index = None;
            // Trigger live search as user types
            let query = app.search_input.trim().to_string();
            if !query.is_empty() {
                app.trigger_search(query);
            }
        }

        KeyCode::Backspace => {
            app.search_input.pop();
            app.history_index = None;
            // Trigger live search as user types
            let query = app.search_input.trim().to_string();
            if !query.is_empty() {
                app.trigger_search(query);
            }
        }

        // History navigation
        KeyCode::Up => app.navigate_history_up(),
        KeyCode::Down => app.navigate_history_down(),

        _ => {}
    }
}

fn execute_confirmation_action(app: &mut App, packages: &[crate::models::Package]) {
    let commands = crate::services::plan_package_transaction(packages, &app.config);
    app.confirmation_commands.clear();

    // Execute commands
    if !commands.is_empty() {
        let installed_packages = packages
            .iter()
            .filter(|p| !p.is_installed)
            .map(|p| p.name.clone())
            .collect();
        let removed_packages = packages
            .iter()
            .filter(|p| p.is_installed)
            .map(|p| p.name.clone())
            .collect();
        app.current_transaction = Some(crate::transaction_history::new_record(
            installed_packages,
            removed_packages,
            &commands,
        ));
        app.selected_packages.clear();
        app.is_operation_running = true;

        if let Some(tx) = &app.action_tx {
            let _ = tx.send(crate::action::Action::new(crate::action::ActionInner::RunCommands(commands)));
        }
    }
}

fn handle_mouse_event(app: &mut App, event: crossterm::event::MouseEvent) {
    let results_count = app.get_paginated_results().len();
    if results_count == 0 {
        return;
    }

    if let InputMode::Editing = app.input_mode {
        return;
    }

    let clicked_row = event.row.saturating_sub(1) as usize;
    if clicked_row < results_count {
        app.selected_index = Some(clicked_row);
    }
}
