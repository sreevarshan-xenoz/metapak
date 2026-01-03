use crossterm::event::{Event, KeyCode};
use crate::app::{App, InputMode};

pub fn handle_event(app: &mut App, event: Event) {
    if let Event::Key(key) = event {
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
            match key.code {
                KeyCode::Char('y') | KeyCode::Enter => {
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
                            let helper = crate::utils::get_aur_helper();
                            
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
                }
                KeyCode::Char('n') | KeyCode::Esc => {
                    app.show_confirm_prompt = false;
                    app.packages_pending_confirmation.clear();
                }
                _ => {}
            }
            return;
        }

        if app.show_console {
             match key.code {
                 KeyCode::Esc | KeyCode::Char('q') => {
                     app.show_console = false;
                 }
                 _ => {}
             }
             return;
        }

        match app.input_mode {
            InputMode::Normal => match key.code {
                KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
                KeyCode::Char('/') | KeyCode::Char('i') => app.input_mode = InputMode::Editing,
                KeyCode::Down | KeyCode::Char('j') => app.next(),
                KeyCode::Up | KeyCode::Char('k') => app.previous(),
                KeyCode::Tab => {
                    app.toggle_selection();
                },
                KeyCode::Enter => {
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
                _ => {}
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
