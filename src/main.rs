mod action;
mod app;
mod config;
mod dependency_visualization;
mod errors;
mod i18n;
mod input;
mod models;
mod services;
mod theme;
mod traits;
mod ui;
mod ui_utils;
mod utils;

use crossterm::{
    event::{self},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use secrecy::ExposeSecret;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::{io, time::Duration};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::sync::Mutex;
use tracing_subscriber::{fmt, EnvFilter};

use crate::action::{Action, ActionResult};
use crate::app::App;
use crate::errors::Result;
use crate::services::{AurHelperCommand, CommandSpec, PackageService};

enum CommandRunResult {
    Finished,
    Cancelled,
}

async fn read_output_lines<R>(
    reader: R,
    is_stderr: bool,
    tx: tokio::sync::mpsc::UnboundedSender<ActionResult>,
) where
    R: tokio::io::AsyncRead + Unpin,
{
    let mut lines = BufReader::new(reader).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        let line = if is_stderr {
            format!("[stderr] {}", line)
        } else {
            line
        };
        let _ = tx.send(ActionResult::CommandOutput(line));
    }
}

async fn run_single_command(
    command: &CommandSpec,
    tx: tokio::sync::mpsc::UnboundedSender<ActionResult>,
    active_pid: Arc<Mutex<Option<u32>>>,
    cancel_requested: Arc<AtomicBool>,
) -> std::result::Result<CommandRunResult, String> {
    let mut cmd = Command::new(&command.prog);
    cmd.args(&command.args);
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
    cmd.stdin(std::process::Stdio::piped());

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Command failed to start: {}", e))?;

    {
        let mut pid_guard = active_pid.lock().await;
        *pid_guard = child.id();
    }

    let stdout_task = child
        .stdout
        .take()
        .map(|stdout| tokio::spawn(read_output_lines(stdout, false, tx.clone())));
    let stderr_task = child
        .stderr
        .take()
        .map(|stderr| tokio::spawn(read_output_lines(stderr, true, tx.clone())));

    let stdin_task = child.stdin.take().map(|mut stdin| {
        let (stdin_tx, mut stdin_rx) = tokio::sync::mpsc::unbounded_channel::<String>();
        let _ = tx.send(ActionResult::CommandInputReady(stdin_tx));
        let tx_for_errors = tx.clone();

        tokio::spawn(async move {
            while let Some(line) = stdin_rx.recv().await {
                let payload = format!("{}\n", line);
                if let Err(e) = stdin.write_all(payload.as_bytes()).await {
                    let _ = tx_for_errors.send(ActionResult::Error(format!(
                        "Failed to send input to command: {}",
                        e
                    )));
                    break;
                }
                let _ = stdin.flush().await;
            }
        })
    });

    let status = child
        .wait()
        .await
        .map_err(|e| format!("Command execution failed: {}", e))?;

    if let Some(task) = stdout_task {
        let _ = task.await;
    }
    if let Some(task) = stderr_task {
        let _ = task.await;
    }
    if let Some(task) = stdin_task {
        task.abort();
    }

    let _ = tx.send(ActionResult::CommandInputClosed);

    {
        let mut pid_guard = active_pid.lock().await;
        *pid_guard = None;
    }

    if cancel_requested.swap(false, Ordering::SeqCst) {
        return Ok(CommandRunResult::Cancelled);
    }

    if status.success() {
        Ok(CommandRunResult::Finished)
    } else {
        Err(format!(
            "Command exited with status: {}",
            status
                .code()
                .map(|c| c.to_string())
                .unwrap_or_else(|| "terminated by signal".to_string())
        ))
    }
}

async fn run_command_sequence(
    commands: Vec<CommandSpec>,
    tx: tokio::sync::mpsc::UnboundedSender<ActionResult>,
    active_pid: Arc<Mutex<Option<u32>>>,
    cancel_requested: Arc<AtomicBool>,
) -> CommandRunResult {
    let _ = tx.send(ActionResult::CommandStarted);

    for command in &commands {
        match run_single_command(
            command,
            tx.clone(),
            active_pid.clone(),
            cancel_requested.clone(),
        )
        .await
        {
            Ok(CommandRunResult::Finished) => {}
            Ok(CommandRunResult::Cancelled) => return CommandRunResult::Cancelled,
            Err(err) => {
                let _ = tx.send(ActionResult::Error(err));
                return CommandRunResult::Finished;
            }
        }
    }

    CommandRunResult::Finished
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    fmt().with_env_filter(EnvFilter::from_default_env()).init();

    tracing::info!("Starting Arch TUI");

    // Load configuration
    let app_config =
        crate::config::AppConfig::load().map_err(|e| crate::errors::AppError::Config(e))?;

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
    let active_pid = Arc::new(Mutex::new(None));
    let cancel_requested = Arc::new(AtomicBool::new(false));

    // Create app
    let aur_helper = app_config.aur_helper.clone();
    let mut app = App::new();
    app.config = app_config;
    app.items_per_page = app.config.ui.items_per_page;
    app.search_debounce_duration = Duration::from_millis(app.config.ui.search_debounce_ms);
    app.max_history_size = app.config.ui.max_search_history;
    app.max_undo_history = app.config.ui.max_undo_history;
    app.theme = app.config.get_theme();
    app.set_sender(action_tx.clone());

    // Initial check for updates
    let _ = action_tx.send(Action::CheckUpdates);

    // Spawn Background Task
    let aur_helper_for_spawn = aur_helper.clone();
    let active_pid_for_spawn = active_pid.clone();
    let cancel_requested_for_spawn = cancel_requested.clone();
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
                                let _ = result_tx_clone
                                    .send(ActionResult::Error(format!("Search failed: {}", e)));
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
                Action::RunCommands(commands) => {
                    let result_tx_clone = result_tx.clone();
                    let active_pid_clone = active_pid_for_spawn.clone();
                    let cancel_requested_clone = cancel_requested_for_spawn.clone();

                    tokio::spawn(async move {
                        let sequence_result = run_command_sequence(
                            commands,
                            result_tx_clone.clone(),
                            active_pid_clone,
                            cancel_requested_clone,
                        )
                        .await;

                        match sequence_result {
                            CommandRunResult::Finished => {
                                let _ = result_tx_clone.send(ActionResult::CommandFinished);
                            }
                            CommandRunResult::Cancelled => {
                                let _ = result_tx_clone.send(ActionResult::CommandCancelled);
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
                    let active_pid_clone = active_pid_for_spawn.clone();
                    let cancel_requested_clone = cancel_requested_for_spawn.clone();

                    tokio::spawn(async move {
                        let helper_cmd = AurHelperCommand::new(&crate::config::AppConfig {
                            aur_helper: aur_helper_value,
                            ..Default::default()
                        });

                        let sequence_result = run_command_sequence(
                            vec![helper_cmd.update_command()],
                            result_tx_clone.clone(),
                            active_pid_clone,
                            cancel_requested_clone,
                        )
                        .await;

                        match sequence_result {
                            CommandRunResult::Finished => {
                                let _ = result_tx_clone.send(ActionResult::CommandFinished);
                                let _ = action_tx_clone.send(Action::CheckUpdates);
                            }
                            CommandRunResult::Cancelled => {
                                let _ = result_tx_clone.send(ActionResult::CommandCancelled);
                            }
                        }
                    });
                }
                Action::CancelOperation => {
                    cancel_requested_for_spawn.store(true, Ordering::SeqCst);
                    if let Some(pid) = *active_pid_for_spawn.lock().await {
                        let _ = std::process::Command::new("kill")
                            .arg("-TERM")
                            .arg(pid.to_string())
                            .status();
                        tokio::time::sleep(Duration::from_millis(800)).await;
                        let still_alive = std::process::Command::new("kill")
                            .arg("-0")
                            .arg(pid.to_string())
                            .status()
                            .map(|s| s.success())
                            .unwrap_or(false);
                        if still_alive {
                            let _ = std::process::Command::new("kill")
                                .arg("-KILL")
                                .arg(pid.to_string())
                                .status();
                        }
                    }
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
                    app.selected_index = if app.get_paginated_results().is_empty() {
                        None
                    } else {
                        Some(0)
                    };
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
                ActionResult::CommandStarted => {
                    app.is_operation_running = true;
                }
                ActionResult::CommandInputReady(tx) => {
                    app.command_stdin_tx = Some(tx);
                }
                ActionResult::CommandInputClosed => {
                    app.command_stdin_tx = None;
                    app.console_input.clear();
                }
                ActionResult::CommandFinished => {
                    app.is_operation_running = false;
                    app.command_stdin_tx = None;
                    app.console_input.clear();
                    app.add_console_output(
                        "----- Process Finished (Press 'q' or 'Esc' to close) -----".to_string(),
                    );
                }
                ActionResult::CommandCancelled => {
                    app.is_operation_running = false;
                    app.command_stdin_tx = None;
                    app.console_input.clear();
                    app.add_console_output("----- Operation Cancelled -----".to_string());
                }
                ActionResult::UpdateCount(count) => {
                    app.available_updates = Some(count);
                }
                ActionResult::Cancelled => {
                    app.is_operation_running = false;
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
    execute!(terminal.backend_mut(), LeaveAlternateScreen)
        .map_err(|e| crate::errors::AppError::Io(e))?;
    terminal
        .show_cursor()
        .map_err(|e| crate::errors::AppError::Io(e))?;

    Ok(())
}
