mod app;
mod ui;
mod ui_utils;
mod input;
mod models;
mod pacman;
mod aur;
mod action;
mod utils;
mod errors;
mod config;
mod services;
mod i18n;
mod dependency_visualization;
mod traits;
mod theme;

use crossterm::{
    event::{self},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, time::Duration};
use std::process::Stdio;
use std::io::{BufRead, BufReader};
use tracing_subscriber::{EnvFilter, fmt};
use secrecy::ExposeSecret;

use crate::app::App;
use crate::action::{Action, ActionResult};
use crate::errors::Result;
use crate::services::{PackageService, AurHelperCommand};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    tracing::info!("Starting Arch TUI");

    // Load configuration
    let app_config = crate::config::AppConfig::load()
        .map_err(|e| crate::errors::AppError::Config(e))?;

    tracing::info!("Configuration loaded successfully");

    // Setup terminal
    enable_raw_mode().map_err(|e| crate::errors::AppError::Io(e))?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).map_err(|e| crate::errors::AppError::Io(e))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(|e| crate::errors::AppError::Io(e))?;

    // Channels
    let (action_tx, mut action_rx) = tokio::sync::mpsc::unbounded_channel();
    let (result_tx, mut result_rx) = tokio::sync::mpsc::unbounded_channel();

    // Create app
    let aur_helper = app_config.aur_helper.clone();
    let mut app = App::new();
    app.config = app_config;
    app.set_sender(action_tx.clone());

    // Initial check for updates
    let _ = action_tx.send(Action::CheckUpdates);

    // Spawn Background Task
    let aur_helper_for_spawn = aur_helper.clone();
    tokio::spawn(async move {
        while let Some(action) = action_rx.recv().await {
            match action {
                Action::Search(query) => {
                    let result_tx_clone = result_tx.clone();
                    
                    tokio::spawn(async move {
                        let package_service = PackageService::new();
                        match package_service.search_all(&query).await {
                            Ok(results) => {
                                let _ = result_tx_clone.send(ActionResult::SearchResults(results));
                            }
                            Err(e) => {
                                tracing::error!("Search failed: {}", e);
                                let _ = result_tx_clone.send(ActionResult::Error(format!("Search failed: {}", e)));
                            }
                        }
                    });
                }
                Action::InitSudo(password) => {
                    let result_tx_clone = result_tx.clone();
                    let password_str = password.expose_secret().to_string();
                    
                    tokio::task::spawn_blocking(move || {
                        // Re-create SecretString for secure handling
                        let pwd = secrecy::SecretString::new(password_str);
                        let success = crate::utils::check_sudo_password(&pwd);
                        let _ = result_tx_clone.send(ActionResult::SudoResult(success));
                    });
                }
                Action::RunCommand { prog, args } => {
                    let result_tx_clone = result_tx.clone();
                    
                    tokio::task::spawn_blocking(move || {
                        let mut cmd = std::process::Command::new(&prog);
                        cmd.args(&args);
                        cmd.stdout(Stdio::piped());
                        cmd.stderr(Stdio::piped());

                        match cmd.spawn() {
                            Ok(mut child) => {
                                // Read stdout
                                if let Some(stdout) = child.stdout.take() {
                                    let reader = BufReader::new(stdout);
                                    for line in reader.lines() {
                                        if let Ok(l) = line {
                                            let _ = result_tx_clone.send(ActionResult::CommandOutput(l));
                                        }
                                    }
                                }

                                // Read stderr
                                if let Some(stderr) = child.stderr.take() {
                                    let reader = BufReader::new(stderr);
                                    for line in reader.lines() {
                                        if let Ok(l) = line {
                                            let _ = result_tx_clone.send(ActionResult::CommandOutput(format!("[stderr] {}", l)));
                                        }
                                    }
                                }

                                let _ = child.wait();
                                let _ = result_tx_clone.send(ActionResult::CommandFinished);
                            }
                            Err(e) => {
                                tracing::error!("Command execution failed: {}", e);
                                let _ = result_tx_clone.send(ActionResult::Error(format!("Command failed: {}", e)));
                            }
                        }
                    });
                }
                Action::CheckUpdates => {
                    let result_tx_clone = result_tx.clone();
                    
                    tokio::spawn(async move {
                        let package_service = PackageService::new();
                        match package_service.check_updates().await {
                            Ok(count) => {
                                let _ = result_tx_clone.send(ActionResult::UpdateCount(count));
                            }
                            Err(e) => {
                                tracing::warn!("Failed to check updates: {}", e);
                                // Don't show error to user for background update check
                            }
                        }
                    });
                }
                Action::SystemUpdate => {
                    let result_tx_clone = result_tx.clone();
                    let action_tx_clone = action_tx.clone();
                    let aur_helper_value = aur_helper_for_spawn.clone();
                    
                    tokio::task::spawn_blocking(move || {
                        let helper_cmd = AurHelperCommand::new(&crate::config::AppConfig {
                            aur_helper: aur_helper_value,
                            ..Default::default()
                        });
                        
                        let full_cmd = helper_cmd.update_command();

                        let mut cmd = std::process::Command::new("sh");
                        cmd.arg("-c").arg(&full_cmd);
                        cmd.stdout(Stdio::piped());
                        cmd.stderr(Stdio::piped());

                        match cmd.spawn() {
                            Ok(mut child) => {
                                if let Some(stdout) = child.stdout.take() {
                                    let reader = BufReader::new(stdout);
                                    for line in reader.lines() {
                                        if let Ok(l) = line {
                                            let _ = result_tx_clone.send(ActionResult::CommandOutput(l));
                                        }
                                    }
                                }
                                
                                if let Some(stderr) = child.stderr.take() {
                                    let reader = BufReader::new(stderr);
                                    for line in reader.lines() {
                                        if let Ok(l) = line {
                                            let _ = result_tx_clone.send(ActionResult::CommandOutput(format!("[stderr] {}", l)));
                                        }
                                    }
                                }
                                
                                let _ = child.wait();
                                let _ = result_tx_clone.send(ActionResult::CommandFinished);
                                // After update, check again
                                let _ = action_tx_clone.send(Action::CheckUpdates);
                            }
                            Err(e) => {
                                tracing::error!("System update failed: {}", e);
                                let _ = result_tx_clone.send(ActionResult::Error(format!("Update failed: {}", e)));
                            }
                        }
                    });
                }
                Action::CancelOperation => {
                    // Acknowledge cancellation
                    let _ = result_tx.send(ActionResult::Cancelled);
                }
            }
        }
    });

    // Main application loop
    let mut last_update = std::time::Instant::now();
    
    loop {
        if app.should_quit {
            break;
        }
        
        // Handle debounced search
        if let Some(query) = app.should_execute_search() {
            app.clear_pending_search();
            app.is_loading = true;
            app.results.clear();
            app.add_to_history(query.clone());
            
            if let Some(tx) = &app.action_tx {
                let _ = tx.send(Action::Search(query));
            }
        }
        
        // Handle Pending Command (Foreground)
        if let Some((prog, args)) = app.pending_command.take() {
             disable_raw_mode()?;
             execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
             terminal.show_cursor()?;
             
             println!("Executing: {} {}", prog, args.join(" "));
             
             match std::process::Command::new(&prog).args(&args).status() {
                 Ok(status) => {
                     if status.success() {
                         println!("\n✓ Command completed successfully");
                     } else {
                         println!("\n✗ Command failed with status: {:?}", status.code());
                     }
                 }
                 Err(e) => {
                     println!("\n✗ Failed to execute command: {}", e);
                 }
             }
             
             println!("\nPress Enter to return to TUI...");
             let mut _input = String::new();
             let _ = std::io::stdin().read_line(&mut _input);
             
             enable_raw_mode()?;
             execute!(terminal.backend_mut(), EnterAlternateScreen)?;
             terminal.hide_cursor()?;
             terminal.clear()?;
        }

        terminal.draw(|f| ui::render(&app, f))?;

        // Handle Input with shorter timeout for responsiveness
        if event::poll(Duration::from_millis(50))? {
             let event = event::read()?;
             input::handle_event(&mut app, event);
        }

        // Handle Async Results
        while let Ok(res) = result_rx.try_recv() {
            match res {
                ActionResult::SearchResults(pkgs) => {
                    app.results = pkgs;
                    app.apply_filter_and_sort();
                    app.is_loading = false;
                    app.error_message = None;
                    app.current_page = 0;
                    app.selected_index = if app.get_paginated_results().is_empty() { None } else { Some(0) };
                }
                ActionResult::Error(msg) => {
                    tracing::error!("Error received: {}", msg);
                    app.error_message = Some(msg);
                    app.is_loading = false;
                }
                ActionResult::SudoResult(success) => {
                    app.is_loading = false;
                    if success {
                        app.show_password_prompt = false;
                        tracing::info!("Sudo authentication successful");
                    } else {
                        app.error_message = Some("Incorrect password. Try again.".to_string());
                        app.password_input.clear();
                    }
                }
                ActionResult::CommandOutput(line) => {
                    app.add_console_output(line);
                }
                ActionResult::CommandFinished => {
                    app.add_console_output("----- Process Finished (Press 'q' or 'Esc' to close) -----".to_string());
                }
                ActionResult::CommandCancelled => {
                    app.add_console_output("----- Operation Cancelled -----".to_string());
                }
                ActionResult::UpdateCount(count) => {
                    app.available_updates = Some(count);
                }
                ActionResult::Cancelled => {
                    app.is_loading = false;
                }
            }
        }
        
        // Periodic cleanup (every 30 seconds)
        if last_update.elapsed() > Duration::from_secs(30) {
            PackageService::clear_expired_cache();
            last_update = std::time::Instant::now();
        }
    }

    // Restore terminal
    tracing::info!("Shutting down Arch TUI");
    disable_raw_mode().map_err(|e| crate::errors::AppError::Io(e))?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen).map_err(|e| crate::errors::AppError::Io(e))?;
    terminal.show_cursor().map_err(|e| crate::errors::AppError::Io(e))?;

    Ok(())
}
