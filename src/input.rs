use crossterm::event::{Event, KeyCode, KeyEventKind};
use crate::app::{App, InputMode};

pub fn handle_event(app: &mut App, event: Event) {
    if let Event::Key(key) = event {
        // Only handle key press events, not release or repeat
        if key.kind != KeyEventKind::Press {
            return;
        }
        
        // Global: Password Prompt Handling
        if app.show_password_prompt {
           match key.code {
               KeyCode::Enter => {
                   // Send password verification action
                   if let Some(tx) = &app.action_tx {
                       let _ = tx.send(crate::action::Action::InitSudo(app.password_input.clone()));
                   }
                   app.is_loading = true; // Show loading while checking
               }
               KeyCode::Char(c) => {
                   app.password_input.push(c);
               }
               KeyCode::Backspace => {
                   app.password_input.pop();
               }
               KeyCode::Esc => {
                   app.should_quit = true; // Quit if they refuse to give pass
               }
               _ => {}
           }
           return;
        }

        if app.show_confirm_prompt {
            // Use 'y' for yes and 'n' for no by default, but could be configurable
            if matches!(key.code, KeyCode::Char('y')) || key.code == KeyCode::Enter {
                app.show_confirm_prompt = false;
                app.show_console = true;
                app.console_buffer.clear();

                let packages = std::mem::take(&mut app.packages_pending_confirmation);
                if !packages.is_empty() {
                    // Partition into installs and removes
                    let (removes, installs): (Vec<&crate::models::Package>, Vec<&crate::models::Package>) =
                        packages.iter().partition(|p| p.is_installed);

                    let mut commands = Vec::new();

                    // Handle Removes
                    if !removes.is_empty() {
                        let mut args = vec!["pacman".to_string(), "-Rns".to_string(), "--noconfirm".to_string()];
                        for p in removes {
                            args.push(p.name.clone());
                        }
                        commands.push(format!("sudo {}", args.join(" ")));
                    }

                    // Handle Installs
                    if !installs.is_empty() {
                        // Check if any is AUR
                        let use_aur_helper = installs.iter().any(|p| matches!(p.source, crate::models::PackageSource::Aur));
                        let helper = crate::utils::get_aur_helper(Some(&app.config.aur_helper));

                        let prog = if use_aur_helper { helper } else { "sudo pacman" };
                        let mut args = vec!["-S".to_string(), "--noconfirm".to_string()];

                        for p in installs {
                            args.push(p.name.clone());
                        }
                        commands.push(format!("{} {}", prog, args.join(" ")));
                    }

                    // Combine commands
                    if !commands.is_empty() {
                        let full_cmd = commands.join(" && ");
                        // Clear selection after action
                        app.selected_packages.clear();

                        if let Some(tx) = &app.action_tx {
                             let _ = tx.send(crate::action::Action::RunCommand {
                                 prog: "sh".to_string(),
                                 args: vec!["-c".to_string(), full_cmd]
                             });
                        }
                    }
                }
            } else if matches!(key.code, KeyCode::Char('n')) || key.code == KeyCode::Esc {
                app.show_confirm_prompt = false;
                app.packages_pending_confirmation.clear();
            }
            return;
        }

        if app.show_package_details {
            // Use Esc to close package details view
            if key.code == KeyCode::Esc {
                app.hide_package_details();
            }
            return;
        }

        if app.show_dependency_visualization {
            // Use Esc to close dependency visualization
            if key.code == KeyCode::Esc {
                app.hide_dependency_visualization();
            }
            return;
        }

        if app.show_console {
             // Use Esc or 'q' to close console by default
             if key.code == KeyCode::Esc || matches!(key.code, KeyCode::Char('q')) {
                 app.show_console = false;
             }
             return;
        }

        match app.input_mode {
            InputMode::Normal => {
                // Handle quit action
                if matches!(key.code, KeyCode::Char(c) if c == app.config.keyboard.quit.chars().next().unwrap_or('q')) || key.code == KeyCode::Esc {
                    app.should_quit = true;
                }
                // Handle search action
                else if matches!(key.code, KeyCode::Char(c) if c == app.config.keyboard.search.chars().next().unwrap_or('/')) || matches!(key.code, KeyCode::Char('i')) {
                    app.input_mode = InputMode::Editing;
                }
                // Handle navigation
                else if key.code == KeyCode::Down || matches!(key.code, KeyCode::Char(c) if c == 'j') {
                    app.next();
                }
                else if key.code == KeyCode::Up || matches!(key.code, KeyCode::Char(c) if c == 'k') {
                    app.previous();
                }
                // Handle selection toggle
                else if key.code == KeyCode::Tab {
                    app.toggle_selection();
                }
                // Handle system update
                else if matches!(key.code, KeyCode::Char(c) if c == app.config.keyboard.install.chars().next().unwrap_or('u')) {
                    app.show_console = true;
                    app.console_buffer.clear();
                    if let Some(tx) = &app.action_tx {
                        let _ = tx.send(crate::action::Action::SystemUpdate);
                    }
                }
                // Show package details
                else if key.code == KeyCode::Char('d') {
                    // Show package details for the selected package
                    if app.selected_index.is_some() {
                        app.show_package_details = true;
                    }
                }
                // Show dependency visualization
                else if key.code == KeyCode::Char('v') {
                    // Show dependency visualization for the selected package
                    if app.selected_index.is_some() {
                        app.show_dependency_visualization = true;
                    }
                }
                // Handle install/remove action
                else if key.code == KeyCode::Enter {
                    // Check batch first
                    if !app.selected_packages.is_empty() {
                         app.packages_pending_confirmation = app.selected_packages.values().cloned().collect();
                         app.show_confirm_prompt = true;
                    } else {
                        // Single item fallback
                        if let Some(index) = app.selected_index {
                             if let Some(pkg) = app.results.get(index) {
                                 app.packages_pending_confirmation = vec![pkg.clone()];
                                 app.show_confirm_prompt = true;
                             }
                        }
                    }
                }
            },
            InputMode::Editing => match key.code {
                KeyCode::Esc => app.input_mode = InputMode::Normal,
                KeyCode::Enter => {
                    app.input_mode = InputMode::Normal;
                    app.is_loading = true;

                    app.results.clear();
                    let query = app.search_input.clone();

                    if let Some(tx) = &app.action_tx {
                        let _ = tx.send(crate::action::Action::Search(query));
                    }
                }
                KeyCode::Char(c) => {
                    app.search_input.push(c);
                }
                KeyCode::Backspace => {
                    app.search_input.pop();
                }
                _ => {}
            },
        }
    }
}
