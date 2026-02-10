use crate::app::{App, InputMode};
use crossterm::event::{Event, KeyCode, KeyEventKind};

pub fn handle_event(app: &mut App, event: Event) {
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

        // Global: Password Prompt Handling
        if app.show_password_prompt {
            match key.code {
                KeyCode::Enter => {
                    if !app.password_input.is_empty() {
                        if let Some(tx) = &app.action_tx {
                            let password = crate::utils::PasswordInput::from_string(
                                app.password_input.expose_secret().to_string(),
                            );
                            let _ = tx.send(crate::action::Action::InitSudo(
                                password.get_secret().clone(),
                            ));
                        }
                        app.is_loading = true;
                    }
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
            if key.code == KeyCode::Esc || key.code == KeyCode::Char('q') {
                app.show_console = false;
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
            if !app.selected_packages.is_empty() {
                app.packages_pending_confirmation =
                    app.selected_packages.values().cloned().collect();
                app.show_confirm_prompt = true;
            } else if let Some(pkg) = app.get_selected_package() {
                app.packages_pending_confirmation = vec![pkg.clone()];
                app.show_confirm_prompt = true;
            }
        }

        // System Update
        KeyCode::Char('U') => {
            app.show_console = true;
            app.clear_console();
            if let Some(tx) = &app.action_tx {
                let _ = tx.send(crate::action::Action::SystemUpdate);
            }
        }

        // Package Details
        KeyCode::Char('d') => app.show_package_details(),

        // Dependency Visualization
        KeyCode::Char('v') => app.show_dependency_visualization(),

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
                let _ = tx.send(crate::action::Action::CancelOperation);
            }
        }

        _ => {}
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
                app.trigger_search(query);
            }
        }

        KeyCode::Char(c) => {
            app.search_input.push(c);
            app.history_index = None;
        }

        KeyCode::Backspace => {
            app.search_input.pop();
            app.history_index = None;
        }

        // History navigation
        KeyCode::Up => app.navigate_history_up(),
        KeyCode::Down => app.navigate_history_down(),

        _ => {}
    }
}

fn execute_confirmation_action(app: &mut App, packages: &[crate::models::Package]) {
    use crate::models::PackageSource;
    use crate::services::AurHelperCommand;

    // Partition into installs and removes
    let (removes, installs): (Vec<_>, Vec<_>) = packages.iter().partition(|p| p.is_installed);

    let mut commands = Vec::new();

    // Handle Removes
    if !removes.is_empty() {
        let names: Vec<String> = removes.iter().map(|p| p.name.clone()).collect();
        commands.push(format!("sudo pacman -Rns --noconfirm {}", names.join(" ")));
    }

    // Handle Installs
    if !installs.is_empty() {
        let use_aur_helper = installs
            .iter()
            .any(|p| matches!(p.source, PackageSource::Aur));
        let helper = AurHelperCommand::new(&app.config);

        let names: Vec<String> = installs.iter().map(|p| p.name.clone()).collect();
        let names_ref: Vec<&str> = names.iter().map(|s| s.as_str()).collect();

        if use_aur_helper {
            commands.push(helper.install_command(&names_ref));
        } else {
            commands.push(format!("sudo pacman -S --noconfirm {}", names.join(" ")));
        }
    }

    // Execute commands
    if !commands.is_empty() {
        let full_cmd = commands.join(" && ");
        app.selected_packages.clear();

        if let Some(tx) = &app.action_tx {
            let _ = tx.send(crate::action::Action::RunCommand {
                prog: "sh".to_string(),
                args: vec!["-c".to_string(), full_cmd],
            });
        }
    }
}
