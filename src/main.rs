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

use crate::app::App;
use crate::action::{Action, ActionResult};
use crate::errors::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // Load configuration
    let app_config = crate::config::AppConfig::load()
        .map_err(|e| crate::errors::AppError::Config(e))?;

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
    let aur_helper = app_config.aur_helper.clone(); // Extract aur_helper before moving app_config
    let mut app = App::new();
    app.config = app_config;
    app.set_sender(action_tx.clone());

    // Initial check for updates
    let _ = action_tx.send(Action::CheckUpdates);

    // Spawn Background Task
    let aur_helper_for_spawn = aur_helper.clone(); // Clone for the spawned task
    tokio::spawn(async move {
        while let Some(action) = action_rx.recv().await {
            match action {
                Action::Search(query) => {
                    let result_tx_clone = result_tx.clone();
                    let query_clone = query.clone();

                    tokio::spawn(async move {
                        let mut results = Vec::new();

                        // Pacman - using blocking for now since it's not async yet
                        if let Ok(pkgs_result) = tokio::task::spawn_blocking({
                            let query_clone = query_clone.clone();
                            move || crate::pacman::search(&query_clone)
                        }).await {
                            if let Ok(mut pkgs) = pkgs_result {
                                results.append(&mut pkgs);
                            }
                        }

                        // AUR - now async
                        if let Ok(aur_pkgs) = crate::aur::search(&query_clone).await {
                            // Update installation status for AUR packages
                            let mut updated_aur_pkgs = Vec::new();
                            for pkg in aur_pkgs {
                                let mut pkg = pkg;
                                if tokio::task::spawn_blocking({
                                    let pkg_name = pkg.name.clone();
                                    move || crate::pacman::is_installed(&pkg_name)
                                }).await.unwrap_or(false) {
                                    pkg.is_installed = true;
                                }
                                updated_aur_pkgs.push(pkg);
                            }
                            results.extend(updated_aur_pkgs);
                        }

                        let _ = result_tx_clone.send(ActionResult::SearchResults(results));
                    });
                }
                Action::InitSudo(password) => {
                    let result_tx_clone = result_tx.clone();
                    tokio::task::spawn_blocking(move || {
                        let success = crate::utils::check_sudo_password(&password);
                        let _ = result_tx_clone.send(ActionResult::SudoResult(success));
                    });
                }

// ... inside async move loop
                Action::RunCommand { prog, args } => {
                    let result_tx_clone = result_tx.clone();
                    tokio::task::spawn_blocking(move || {
                        let mut cmd = std::process::Command::new(&prog);
                        cmd.args(&args);
                        cmd.stdout(Stdio::piped());
                        cmd.stderr(Stdio::piped()); // Capture stderr too

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

                                // Ideally we read stderr concurrently but doing it sequentially is okay for basic logs
                                // or mix them. For now let's just wait.
                                let _ = child.wait();
                                let _ = result_tx_clone.send(ActionResult::CommandFinished);
                            }
                            Err(e) => {
                                let _ = result_tx_clone.send(ActionResult::Error(e.to_string()));
                            }
                        }
                    });
                }
                Action::CommandInput(_) => {}
                Action::CheckUpdates => {
                    let result_tx_clone = result_tx.clone();
                    tokio::task::spawn_blocking(move || {
                        if let Ok(count) = crate::pacman::check_updates() {
                            let _ = result_tx_clone.send(ActionResult::UpdateCount(count));
                        }
                    });
                }
                Action::SystemUpdate => {
                    let result_tx_clone = result_tx.clone();
                    let action_tx_clone = action_tx.clone();
                    let aur_helper_value = aur_helper_for_spawn.clone(); // Use the cloned value
                    tokio::task::spawn_blocking(move || {
                        let helper = crate::utils::get_aur_helper(Some(&aur_helper_value));
                        // For system update, we use -Syu
                        // If it's a helper like paru/yay, it handles both repo and AUR
                        let mut cmd = std::process::Command::new("sh");
                        let full_cmd = if helper == "sudo pacman" {
                            "sudo pacman -Syu --noconfirm".to_string()
                        } else {
                            format!("{} -Syu --noconfirm", helper)
                        };

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
                                let _ = child.wait();
                                let _ = result_tx_clone.send(ActionResult::CommandFinished);
                                // After update, check again
                                let _ = action_tx_clone.send(Action::CheckUpdates);
                            }
                            Err(e) => {
                                let _ = result_tx_clone.send(ActionResult::Error(e.to_string()));
                            }
                        }
                    });
                }
            }
        }
    });

    // Run the application
    loop {
        if app.should_quit {
            break;
        }
        
        // Handle Pending Command (Foreground)
        if let Some((prog, args)) = app.pending_command.take() {
             disable_raw_mode()?;
             execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
             terminal.show_cursor()?;
             
             println!("Executing: {} {}", prog, args.join(" "));
             let _ = std::process::Command::new(prog).args(args).status();
             
             println!("\nPress Enter to return to TUI...");
             let mut _input = String::new();
             let _ = std::io::stdin().read_line(&mut _input);
             
             enable_raw_mode()?;
             execute!(terminal.backend_mut(), EnterAlternateScreen)?;
             terminal.hide_cursor()?;
             terminal.clear()?;
        }

        terminal.draw(|f| ui::render(&app, f))?;

        // 1. Handle Input
        if event::poll(Duration::from_millis(10))? {
             let event = event::read()?;
             input::handle_event(&mut app, event);
        }

        // 2. Handle Async Results
        while let Ok(res) = result_rx.try_recv() {
            match res {
                ActionResult::SearchResults(pkgs) => {
                    app.results = pkgs;
                    app.is_loading = false;
                    app.error_message = None; 
                }
                ActionResult::Error(msg) => {
                    app.error_message = Some(msg);
                    app.is_loading = false;
                }
                ActionResult::SudoResult(success) => {
                    app.is_loading = false;
                    if success {
                        app.show_password_prompt = false; 
                    } else {
                        app.error_message = Some("Incorrect password. Try again.".to_string());
                        app.password_input.clear();
                    }
                }
                ActionResult::CommandOutput(line) => {
                    app.console_buffer.push(line);
                }
                ActionResult::CommandFinished => {
                    app.console_buffer.push("----- Process Finished (Press 'q' or 'Esc' to close) -----".to_string());
                }
                ActionResult::UpdateCount(count) => {
                    app.available_updates = Some(count);
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode().map_err(|e| crate::errors::AppError::Io(e))?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen).map_err(|e| crate::errors::AppError::Io(e))?;
    terminal.show_cursor().map_err(|e| crate::errors::AppError::Io(e))?;

    Ok(())
}
