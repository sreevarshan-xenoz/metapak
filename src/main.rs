mod action;
mod animations;
mod app;
mod backends;
mod config;
mod constants;
mod dependency_visualization;
mod diagnostics;
mod errors;
mod hooks;
mod i18n;
mod input;
mod models;
mod notifications;
mod operation_queue;
mod platform;
mod search;
mod services;
mod simulation;
mod telemetry;
mod theme;
mod traits;
mod transaction_history;
mod transaction_manager;
mod ui;
mod ui_utils;
mod utils;
mod watchdog;

// CLI arguments disabled - cargo not available for testing
// use clap::{Arg, Command as ClapCommand};

use crossterm::{
    event::{self},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use secrecy::ExposeSecret;
use std::collections::VecDeque;
use std::io;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::sync::Mutex;
use tracing_subscriber::{fmt, EnvFilter};

use crate::constants::retry::{
    GENERAL_RETRY_DELAY_SECS, LOCK_RETRY_DELAY_SECS, MAX_ATTEMPTS, NETWORK_RETRY_DELAY_SECS,
};
use crate::constants::ui::{
    CAPTURED_OUTPUT_MAX_LINES, CLEANUP_INTERVAL_SECS, INPUT_POLL_TIMEOUT_MS,
    UPDATE_CHECK_INTERVAL_SECS,
};

use crate::action::{Action, ActionInner, ActionResult};
use crate::app::{App, ViewMode};
// CommandRunResult is defined locally below
pub enum CommandRunResult {
    Finished,
    Cancelled,
}
use crate::errors::Result;
use crate::notifications::DesktopNotifier;
use crate::services::{AurHelperCommand, CommandSpec, PackageService};
use crate::traits::PackageSimulator;
use crate::transaction_history::{save_history, TransactionStatus};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "metapak")]
#[command(about = "metapak Package Manager", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Initial search query (opens TUI with search pre-filled)
    #[arg(short, long)]
    search: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Search for packages
    Search {
        /// Package name to search for
        query: String,
    },
    /// Check for updates
    Check,
    /// Install a package
    Install {
        /// Package name(s) to install
        package: Vec<String>,
    },
    /// Remove a package
    Remove {
        /// Package name(s) to remove
        package: Vec<String>,
    },
}

async fn read_output_lines<R>(
    reader: R,
    is_stderr: bool,
    tx: tokio::sync::mpsc::UnboundedSender<ActionResult>,
    captured: Arc<Mutex<VecDeque<String>>>,
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
        {
            let mut buf = captured.lock().await;
            buf.push_back(line.clone());
            if buf.len() > CAPTURED_OUTPUT_MAX_LINES {
                let _ = buf.pop_front();
            }
        }
        let _ = tx.send(ActionResult::CommandOutput(line));
    }
}

fn output_contains_dependency_error(output: &[String]) -> bool {
    let haystack = output.join("\n").to_lowercase();
    haystack.contains("could not satisfy dependencies")
        || haystack.contains("unable to satisfy dependency")
        || haystack.contains("breaks dependency")
}

fn output_contains_lock_error(output: &[String]) -> bool {
    let haystack = output.join("\n").to_lowercase();
    haystack.contains("unable to lock database")
        || haystack.contains("database is locked")
        || haystack.contains("failed to init transaction")
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

    let captured_output = Arc::new(Mutex::new(VecDeque::<String>::new()));
    let stdout_task = child.stdout.take().map(|stdout| {
        tokio::spawn(read_output_lines(
            stdout,
            false,
            tx.clone(),
            captured_output.clone(),
        ))
    });
    let stderr_task = child.stderr.take().map(|stderr| {
        tokio::spawn(read_output_lines(
            stderr,
            true,
            tx.clone(),
            captured_output.clone(),
        ))
    });

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
        let output_deque = captured_output.lock().await;
        let output: Vec<String> = output_deque.iter().cloned().collect();
        let mut context = format!(
            "Command exited with status: {}",
            status
                .code()
                .map(|c| c.to_string())
                .unwrap_or_else(|| "terminated by signal".to_string())
        );
        if output_contains_dependency_error(&output) {
            context.push_str(" [dependency-error]");
        }
        if output_contains_lock_error(&output) {
            context.push_str(" [db-lock-error]");
        }
        Err(context)
    }
}

async fn run_command_sequence(
    commands: Vec<CommandSpec>,
    tx: tokio::sync::mpsc::UnboundedSender<ActionResult>,
    active_pid: Arc<Mutex<Option<u32>>>,
    cancel_requested: Arc<AtomicBool>,
) -> Result<CommandRunResult> {
    let _ = tx.send(ActionResult::CommandStarted);

    for command in &commands {
        let mut attempts = 0usize;

        loop {
            attempts += 1;
            match run_single_command(
                command,
                tx.clone(),
                active_pid.clone(),
                cancel_requested.clone(),
            )
            .await
            {
                Ok(CommandRunResult::Finished) => break,
                Ok(CommandRunResult::Cancelled) => return Ok(CommandRunResult::Cancelled),
                Err(err) => {
                    let error_lower = err.to_lowercase();
                    let is_dependency_error = error_lower.contains("dependency")
                        || error_lower.contains("[dependency-error]");
                    let is_lock_error =
                        error_lower.contains("lock") || error_lower.contains("[db-lock-error]");
                    let is_conflict_error =
                        error_lower.contains("conflict") || error_lower.contains("::");
                    let is_signature_error =
                        error_lower.contains("signature") || error_lower.contains("pgp");
                    let is_disk_space_error =
                        error_lower.contains("disk") || error_lower.contains("space");
                    let is_network_error =
                        error_lower.contains("network") || error_lower.contains("connection");
                    let is_cache_error =
                        error_lower.contains("cache") || error_lower.contains("invalid");

                    // Handle database lock - retry with backoff
                    if is_lock_error && attempts < MAX_ATTEMPTS {
                        let _ = tx.send(ActionResult::CommandOutput(format!(
                            "Detected pacman DB lock. Attempt {}/3. Waiting {}s...",
                            attempts, LOCK_RETRY_DELAY_SECS
                        )));
                        tokio::time::sleep(Duration::from_secs(LOCK_RETRY_DELAY_SECS)).await;
                        continue;
                    }

                    // Handle dependency errors - run system upgrade first
                    if is_dependency_error {
                        let _ = tx.send(ActionResult::CommandOutput(
                            "Dependency issue detected. Running system upgrade...".to_string(),
                        ));

                        let fix = if cfg!(windows) {
                            CommandSpec::new_no_sudo(
                                "winget",
                                vec!["upgrade".to_string(), "--all".to_string()],
                            )
                        } else {
                            CommandSpec {
                                prog: "sudo".to_string(),
                                args: vec![
                                    "pacman".to_string(),
                                    "-Syu".to_string(),
                                    "--noconfirm".to_string(),
                                ],
                            }
                        };

                        match run_single_command(
                            &fix,
                            tx.clone(),
                            active_pid.clone(),
                            cancel_requested.clone(),
                        )
                        .await
                        {
                            Ok(CommandRunResult::Finished) => {
                                let _ = tx.send(ActionResult::CommandOutput(
                                    "System upgrade complete. Retrying...".to_string(),
                                ));
                                if attempts < MAX_ATTEMPTS {
                                    continue;
                                }
                            }
                            Ok(CommandRunResult::Cancelled) => {
                                return Ok(CommandRunResult::Cancelled)
                            }
                            Err(fix_err) => {
                                return Err(crate::errors::AppError::Backend(format!(
                                    "System upgrade failed: {}",
                                    fix_err
                                )));
                            }
                        }
                    }

                    // Handle conflict errors - report to user instead of blindly overwriting
                    if is_conflict_error {
                        // The error string already contains relevant output from the command
                        let conflict_details: Vec<String> = err
                            .lines()
                            .filter(|line| {
                                let lower = line.to_lowercase();
                                lower.contains("conflict")
                                    || lower.contains("exists in filesystem")
                                    || lower.contains("are in conflict")
                            })
                            .map(|s| s.to_string())
                            .collect();

                        let detail_msg = if conflict_details.is_empty() {
                            format!("Package conflicts detected: {}", err)
                        } else {
                            format!(
                                "Package conflicts detected:\n{}",
                                conflict_details.join("\n")
                            )
                        };

                        let _ = tx.send(ActionResult::CommandOutput(detail_msg.clone()));
                        let _ = tx.send(ActionResult::CommandOutput(
                            "Tip: Use 'pacman -Syu --overwrite <specific-file>' to resolve individual conflicts.".to_string(),
                        ));

                        return Err(crate::errors::AppError::Backend(format!(
                            "Transaction aborted due to package conflicts: {}",
                            err
                        )));
                    }

                    // Handle signature errors - refresh keys
                    if is_signature_error && !cfg!(windows) {
                        let _ = tx.send(ActionResult::CommandOutput(
                            "PGP signature issue detected. Attempting to refresh keys..."
                                .to_string(),
                        ));
                        let fix = CommandSpec {
                            prog: "sudo".to_string(),
                            args: vec!["pacman-key".to_string(), "--init".to_string()],
                        };
                        let _ = run_single_command(
                            &fix,
                            tx.clone(),
                            active_pid.clone(),
                            cancel_requested.clone(),
                        )
                        .await;

                        let fix2 = CommandSpec {
                            prog: "sudo".to_string(),
                            args: vec![
                                "pacman".to_string(),
                                "-Sy".to_string(),
                                "--noconfirm".to_string(),
                            ],
                        };
                        if let Ok(CommandRunResult::Finished) = run_single_command(
                            &fix2,
                            tx.clone(),
                            active_pid.clone(),
                            cancel_requested.clone(),
                        )
                        .await
                        {
                            let _ = tx.send(ActionResult::CommandOutput(
                                "Keys refreshed. Retrying...".to_string(),
                            ));
                            if attempts < MAX_ATTEMPTS {
                                continue;
                            }
                        }
                    }

                    // Handle disk space errors
                    if is_disk_space_error {
                        let msg = if cfg!(windows) {
                            "Disk space issue. Please free up some space and try again.".to_string()
                        } else {
                            "Disk space issue. Try: sudo pacman -Scc to clean cache".to_string()
                        };
                        let _ = tx.send(ActionResult::CommandOutput(msg));
                        return Err(crate::errors::AppError::ResourceExhausted(err));
                    }

                    // Handle network errors - retry
                    if is_network_error && attempts < 2 {
                        let _ = tx.send(ActionResult::CommandOutput(format!(
                            "Network error. Retrying in {}s...",
                            NETWORK_RETRY_DELAY_SECS
                        )));
                        tokio::time::sleep(Duration::from_secs(NETWORK_RETRY_DELAY_SECS)).await;
                        continue;
                    }

                    // Handle cache errors
                    if is_cache_error && !cfg!(windows) {
                        let _ = tx.send(ActionResult::CommandOutput(
                            "Cache issue. Cleaning local cache...".to_string(),
                        ));
                        let fix = CommandSpec {
                            prog: "rm".to_string(),
                            args: vec!["-rf".to_string(), "/var/cache/pacman/pkg/*".to_string()],
                        };
                        let _ = run_single_command(
                            &fix,
                            tx.clone(),
                            active_pid.clone(),
                            cancel_requested.clone(),
                        )
                        .await;
                    }

                    // Max attempts reached
                    if attempts >= MAX_ATTEMPTS {
                        return Err(crate::errors::AppError::Backend(format!(
                            "Failed after {} attempts: {}",
                            attempts, err
                        )));
                    }

                    // Retry other errors
                    let _ = tx.send(ActionResult::CommandOutput(format!(
                        "Error (attempt {}/{}): {}. Retrying...",
                        attempts, MAX_ATTEMPTS, err
                    )));
                    tokio::time::sleep(Duration::from_secs(GENERAL_RETRY_DELAY_SECS)).await;
                }
            }
        }
    }

    Ok(CommandRunResult::Finished)
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    fmt().with_env_filter(EnvFilter::from_default_env()).init();

    // Migrate configuration if needed
    if let Err(e) = crate::config::migrate_config() {
        tracing::warn!("Failed to migrate configuration: {}", e);
    }

    // Parse CLI arguments
    let cli = Cli::parse();

    // Handle CLI-only commands (non-interactive)
    if let Some(command) = cli.command {
        match command {
            Commands::Search { query } => {
                eprintln!("Searching for: {}", query);
                let config = crate::config::AppConfig::default();
                let service = PackageService::new(config);
                match tokio::runtime::Runtime::new()?.block_on(service.search_all(&query)) {
                    Ok(results) => {
                        eprintln!("Found {} packages:", results.len());
                        for pkg in results.iter().take(20) {
                            eprintln!("  {} - {}", pkg.name, pkg.version);
                        }
                        if results.len() > 20 {
                            eprintln!("  ... and {} more", results.len() - 20);
                        }
                    }
                    Err(e) => eprintln!("Search failed: {}", e),
                }
                return Ok(());
            }
            Commands::Check => {
                let config = crate::config::AppConfig::default();
                let service = PackageService::new(config);
                match tokio::runtime::Runtime::new()?.block_on(service.check_updates()) {
                    Ok(count) => {
                        if count > 0 {
                            eprintln!("{} updates available", count);
                        } else {
                            eprintln!("System is up to date");
                        }
                    }
                    Err(e) => eprintln!("Check failed: {}", e),
                }
                return Ok(());
            }
            Commands::Install { package } => {
                eprintln!("Installing: {:?}", package);
                let helper = AurHelperCommand::new(&crate::config::AppConfig::default());
                let cmd =
                    helper.install_command(&package.iter().map(|s| s.as_str()).collect::<Vec<_>>());
                if cfg!(windows) {
                    eprintln!("Run: {} {}", cmd.prog, cmd.args.join(" "));
                } else {
                    eprintln!("Run: sudo {} {}", cmd.prog, cmd.args.join(" "));
                }
                return Ok(());
            }
            Commands::Remove { package } => {
                eprintln!("Removing: {:?}", package);
                let helper = AurHelperCommand::new(&crate::config::AppConfig::default());
                let cmd =
                    helper.remove_command(&package.iter().map(|s| s.as_str()).collect::<Vec<_>>());
                if cfg!(windows) {
                    eprintln!("Run: {} {}", cmd.prog, cmd.args.join(" "));
                } else {
                    eprintln!("Run: sudo {} {}", cmd.prog, cmd.args.join(" "));
                }
                return Ok(());
            }
        }
    }

    // Display platform and package manager info
    let platform_info = crate::platform::get_platform_info();
    tracing::info!("Starting Universal TUI on {}", platform_info);
    eprintln!("Platform: {}", platform_info);

    // Load configuration
    let app_config = crate::config::AppConfig::load().map_err(crate::errors::AppError::Config)?;

    tracing::info!("Configuration loaded successfully");

    // Setup terminal
    enable_raw_mode().map_err(crate::errors::AppError::Io)?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )
    .map_err(crate::errors::AppError::Io)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(crate::errors::AppError::Io)?;

    // Channels
    let (action_tx, mut action_rx) = tokio::sync::mpsc::unbounded_channel();
    let (result_tx, mut result_rx) = tokio::sync::mpsc::unbounded_channel();
    let active_pid = Arc::new(Mutex::new(None));
    let cancel_requested = Arc::new(AtomicBool::new(false));
    let shutdown_requested = Arc::new(AtomicBool::new(false));

    // Initialize robustness suite
    let watchdog = Arc::new(crate::watchdog::HealthWatchdog::new(Duration::from_secs(5)));

    let sim_backend = if app_config.robustness.simulation_backend == "auto" {
        crate::platform::get_default_package_manager()
            .name()
            .to_string()
    } else {
        app_config.robustness.simulation_backend.clone()
    };
    let simulator = Arc::new(crate::simulation::SimulationEngine::new(sim_backend));

    // Detect and initialize snapshot provider
    let root_path = if cfg!(windows) { "C:\\" } else { "/" };
    let snapshots_dir = if cfg!(windows) {
        "C:\\.snapshots"
    } else {
        "/.snapshots"
    };
    let snapshot_provider: Option<Arc<dyn crate::traits::SnapshotProvider>> = if cfg!(windows) {
        // Windows-specific snapshot provider could be added here (e.g. VSS)
        None
    } else if std::path::Path::new("/usr/bin/btrfs").exists() {
        tracing::info!("Detected btrfs, using BtrfsProvider");
        Some(Arc::new(
            crate::backends::snapshots::btrfs::BtrfsProvider::new(root_path, snapshots_dir),
        ))
    } else if std::path::Path::new("/usr/bin/timeshift").exists() {
        tracing::info!("Detected timeshift, using TimeshiftProvider");
        Some(Arc::new(
            crate::backends::snapshots::timeshift::TimeshiftProvider::new(),
        ))
    } else {
        tracing::warn!("No supported snapshot provider (btrfs, timeshift) detected");
        None
    };

    let transaction_manager = crate::transaction_manager::TransactionManager::new(
        snapshot_provider,
        simulator.clone(),
        watchdog.clone(),
        app_config.robustness.snapshot_keep_count,
    );

    // Attach hooks from config
    let hook_runner = crate::hooks::HookRunner::from_config(&app_config.hooks);
    let transaction_manager = Arc::new(transaction_manager.with_hooks(hook_runner));

    // Create app
    let aur_helper = app_config.aur_helper.clone();
    let mut app = App::new();
    app.config = app_config.clone();
    app.operation_queue =
        crate::operation_queue::OperationQueue::with_manager(transaction_manager.clone());
    app.items_per_page = app.config.ui.items_per_page;
    app.search_debounce_duration = Duration::from_millis(app.config.ui.search_debounce_ms);
    app.max_history_size = app.config.ui.max_search_history;
    app.max_undo_history = app.config.ui.max_undo_history;
    app.theme = app.config.get_theme();
    app.localizer = crate::i18n::Localizer::with_config(&app.config.i18n.language);
    app.set_sender(action_tx.clone());
    if let Ok(history) = crate::transaction_history::load_history() {
        app.transaction_history = history.into();
    }

    // Wire --search flag to pre-fill search input
    if let Some(query) = cli.search {
        app.search_input = query;
        app.input_mode = crate::app::InputMode::Editing;
    }

    // Rotate telemetry logs if needed
    if app.config.telemetry.enabled {
        crate::telemetry::rotate_logs(
            app.config.telemetry.max_log_size_mb,
            app.config.telemetry.max_log_files,
        );
    }

    // Validate theme contrast ratios on startup
    let contrast_failures = app.theme.validate_contrast();
    if !contrast_failures.is_empty() {
        for (name, ratio) in &contrast_failures {
            tracing::warn!(
                "WCAG AA contrast warning: '{}' has ratio {:.2}:1 (minimum 4.5:1)",
                name,
                ratio
            );
        }
    }

    // Initial check for updates (only if auto_update_on_startup is enabled)
    if app.config.ui.auto_update_on_startup {
        let _ = action_tx.send(Action::new(ActionInner::CheckUpdates));
    }

    // Start auto-update checker if enabled
    let auto_check_enabled = app.config.ui.auto_check_updates;
    let update_interval = app.config.ui.update_check_interval_minutes;

    // Spawn background auto-update checker
    if auto_check_enabled {
        let action_tx_clone = action_tx.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(update_interval * 60));
            loop {
                interval.tick().await;
                let _ = action_tx_clone.send(Action::new(ActionInner::CheckUpdates));
            }
        });
    }

    // Spawn Background Task
    let aur_helper_for_spawn = aur_helper.clone();
    let active_pid_for_spawn = active_pid.clone();
    let cancel_requested_for_spawn = cancel_requested.clone();
    let transaction_manager_for_spawn = transaction_manager.clone();
    let app_config_for_spawn = app_config.clone();
    tokio::spawn(async move {
        while let Some(action) = action_rx.recv().await {
            let action_id = action.id();
            let config = app_config_for_spawn.clone();
            match &action.inner {
                ActionInner::Search(query) => {
                    tracing::info!(action_id, "Processing Search action");
                    let result_tx_clone = result_tx.clone();
                    let query = query.clone();
                    let config = config.clone();

                    tokio::spawn(async move {
                        let package_service = PackageService::new(config);
                        match package_service.search_all(&query).await {
                            Ok(results) => {
                                let _ = result_tx_clone.send(ActionResult::SearchResults(results));
                            }
                            Err(e) => {
                                tracing::error!(action_id, "Search failed: {}", e);
                                let _ = result_tx_clone
                                    .send(ActionResult::Error(format!("Search failed: {}", e)));
                            }
                        }
                    });
                }
                ActionInner::InitSudo(password) => {
                    tracing::info!(action_id, "Processing InitSudo action");
                    let result_tx_clone = result_tx.clone();
                    let password_str = password.expose_secret().to_string();

                    tokio::task::spawn_blocking(move || {
                        // Re-create SecretString for secure handling
                        let pwd = secrecy::SecretString::new(password_str);
                        let success = crate::utils::check_sudo_password(&pwd);
                        let _ = result_tx_clone.send(ActionResult::SudoResult(success));
                    });
                }
                ActionInner::RunCommands(commands) => {
                    tracing::info!(action_id, "Processing RunCommands action");
                    let result_tx_clone = result_tx.clone();
                    let active_pid_clone = active_pid_for_spawn.clone();
                    let cancel_requested_clone = cancel_requested_for_spawn.clone();
                    let commands = commands.clone();
                    let tm = transaction_manager_for_spawn.clone();

                    tokio::spawn(async move {
                        let res = tm
                            .run_safe_transaction("operation", Some(&commands.clone()), || async {
                                run_command_sequence(
                                    commands,
                                    result_tx_clone.clone(),
                                    active_pid_clone,
                                    cancel_requested_clone,
                                )
                                .await
                            })
                            .await;

                        match res {
                            Ok(CommandRunResult::Finished) => {
                                let _ = result_tx_clone.send(ActionResult::CommandFinished);
                            }
                            Ok(CommandRunResult::Cancelled) => {
                                let _ = result_tx_clone.send(ActionResult::CommandCancelled);
                            }
                            Err(e) => {
                                if let crate::errors::AppError::TransactionFailed(msg, Some(id)) = e
                                {
                                    let _ = result_tx_clone
                                        .send(ActionResult::TransactionFailedWithRollback(msg, id));
                                } else {
                                    let _ = result_tx_clone.send(ActionResult::Error(format!(
                                        "Transaction failed: {}",
                                        e
                                    )));
                                }
                            }
                        }
                    });
                }
                ActionInner::CheckUpdates => {
                    tracing::info!(action_id, "Processing CheckUpdates action");
                    let result_tx_clone = result_tx.clone();
                    let config = config.clone();

                    tokio::spawn(async move {
                        let package_service = PackageService::new(config);
                        match package_service.check_updates().await {
                            Ok(count) => {
                                let _ = result_tx_clone.send(ActionResult::UpdateCount(count));
                            }
                            Err(e) => {
                                tracing::warn!(action_id, "Failed to check updates: {}", e);
                                // Don't show error to user for background update check
                            }
                        }
                    });
                }
                ActionInner::SystemUpdate => {
                    tracing::info!(action_id, "Processing SystemUpdate action");
                    let result_tx_clone = result_tx.clone();
                    let action_tx_clone = action_tx.clone();
                    let aur_helper_value = aur_helper_for_spawn.clone();
                    let active_pid_clone = active_pid_for_spawn.clone();
                    let cancel_requested_clone = cancel_requested_for_spawn.clone();
                    let tm = transaction_manager_for_spawn.clone();

                    tokio::spawn(async move {
                        let helper_cmd = AurHelperCommand::new(&crate::config::AppConfig {
                            aur_helper: aur_helper_value,
                            ..Default::default()
                        });
                        let update_cmd = helper_cmd.update_command();
                        let update_cmds = vec![update_cmd.clone()];

                        let res = tm
                            .run_safe_transaction(
                                "system-update",
                                Some(&update_cmds.clone()),
                                || async {
                                    run_command_sequence(
                                        update_cmds,
                                        result_tx_clone.clone(),
                                        active_pid_clone,
                                        cancel_requested_clone,
                                    )
                                    .await
                                },
                            )
                            .await;

                        match res {
                            Ok(CommandRunResult::Finished) => {
                                let _ = result_tx_clone.send(ActionResult::CommandFinished);
                                let _ =
                                    action_tx_clone.send(Action::new(ActionInner::CheckUpdates));
                            }
                            Ok(CommandRunResult::Cancelled) => {
                                let _ = result_tx_clone.send(ActionResult::CommandCancelled);
                            }
                            Err(e) => {
                                if let crate::errors::AppError::TransactionFailed(msg, Some(id)) = e
                                {
                                    let _ = result_tx_clone
                                        .send(ActionResult::TransactionFailedWithRollback(msg, id));
                                } else {
                                    let _ = result_tx_clone.send(ActionResult::Error(format!(
                                        "Transaction failed: {}",
                                        e
                                    )));
                                }
                            }
                        }
                    });
                }
                ActionInner::Simulate(commands) => {
                    tracing::info!(action_id, "Processing Simulate action");
                    let result_tx_clone = result_tx.clone();
                    let simulator = simulator.clone();
                    let commands = commands.clone();

                    tokio::spawn(async move {
                        let pkg_names = if let Some(cmd) = commands.first() {
                            if cmd.args.len() > 1 {
                                vec![cmd.args[cmd.args.len() - 1].as_str()]
                            } else {
                                vec!["system"]
                            }
                        } else {
                            vec!["system"]
                        };

                        match simulator.simulate_install(&pkg_names).await {
                            Ok(res) => {
                                let _ = result_tx_clone.send(ActionResult::SimulationResult(res));
                            }
                            Err(e) => {
                                let _ = result_tx_clone
                                    .send(ActionResult::Error(format!("Simulation failed: {}", e)));
                            }
                        }
                    });
                }
                ActionInner::Rollback(id) => {
                    tracing::info!(action_id, "Processing Rollback action for id: {}", id);
                    let result_tx_clone = result_tx.clone();
                    let id = id.clone();
                    let tm = transaction_manager_for_spawn.clone();

                    tokio::spawn(async move {
                        if let Some(provider) = tm.get_snapshot_provider() {
                            match provider.rollback(&id).await {
                                Ok(_) => {
                                    let _ = result_tx_clone.send(ActionResult::CommandOutput(
                                        format!("Rollback to {} successful. Please reboot.", id),
                                    ));
                                    let _ =
                                        result_tx_clone.send(ActionResult::RollbackFinished(id));
                                }
                                Err(e) => {
                                    let _ = result_tx_clone.send(ActionResult::Error(format!(
                                        "Rollback failed: {}",
                                        e
                                    )));
                                }
                            }
                        } else {
                            let _ = result_tx_clone.send(ActionResult::Error(
                                "No snapshot provider available for rollback".to_string(),
                            ));
                        }
                    });
                }
                ActionInner::CancelOperation => {
                    tracing::info!(action_id, "Processing CancelOperation action");
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
                ActionInner::SwitchViewMode(mode) => {
                    tracing::info!(action_id, "Processing SwitchViewMode action: {:?}", mode);
                    let _ = result_tx.send(ActionResult::ViewModeSwitched(*mode));
                }
                ActionInner::SearchEcosystem { provider, query } => {
                    tracing::info!(
                        action_id,
                        "Processing SearchEcosystem action for {:?}: {}",
                        provider,
                        query
                    );
                    let result_tx_clone = result_tx.clone();
                    let query = query.clone();
                    let provider = *provider;
                    let config = config.clone();

                    tokio::spawn(async move {
                        let package_service = PackageService::new(config);
                        match package_service.search_ecosystem(provider, &query).await {
                            Ok(results) => {
                                let _ = result_tx_clone
                                    .send(ActionResult::EcosystemSearchResults(results));
                            }
                            Err(e) => {
                                tracing::error!(action_id, "Ecosystem search failed: {}", e);
                                let _ = result_tx_clone.send(ActionResult::Error(format!(
                                    "Ecosystem search failed: {}",
                                    e
                                )));
                            }
                        }
                    });
                }
            }
        }
    });

    // Main application loop
    let mut last_update = std::time::Instant::now();
    let mut last_update_check = std::time::Instant::now();

    loop {
        if app.should_quit {
            break;
        }

        // Handle debounced search (for live search as user types)
        if let Some(query) = app.should_execute_search() {
            app.clear_pending_search();
            app.is_loading = true;
            app.add_to_history(query.clone());

            if let Some(tx) = &app.action_tx {
                match app.view_mode {
                    ViewMode::System => {
                        app.results.clear();
                        let _ = tx.send(Action::new(ActionInner::Search(query)));
                    }
                    ViewMode::Ecosystem => {
                        app.ecosystem_results.clear();
                        let _ = tx.send(Action::new(ActionInner::SearchEcosystem {
                            provider: app.active_ecosystem,
                            query,
                        }));
                    }
                }
            }
        }

        // Handle immediate search (when user presses Enter)
        if let Some(query) = app.immediate_search.take() {
            app.is_loading = true;
            app.add_to_history(query.clone());

            if let Some(tx) = &app.action_tx {
                match app.view_mode {
                    ViewMode::System => {
                        app.results.clear();
                        let _ = tx.send(Action::new(ActionInner::Search(query)));
                    }
                    ViewMode::Ecosystem => {
                        app.ecosystem_results.clear();
                        let _ = tx.send(Action::new(ActionInner::SearchEcosystem {
                            provider: app.active_ecosystem,
                            query,
                        }));
                    }
                }
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

        terminal.draw(|f| ui::render(&mut app, f))?;

        app.tick(crate::constants::ui::TICK_INTERVAL_MS);

        // Handle Input with shorter timeout for responsiveness
        if event::poll(Duration::from_millis(INPUT_POLL_TIMEOUT_MS))? {
            let event = event::read()?;

            // Check for Ctrl+C (Interrupt) for graceful shutdown
            if let event::Event::Key(key) = &event {
                if key.code == event::KeyCode::Char('c')
                    && key.modifiers.contains(event::KeyModifiers::CONTROL)
                {
                    tracing::info!("Ctrl+C detected, initiating graceful shutdown");
                    shutdown_requested.store(true, Ordering::SeqCst);

                    // Signal the background task to stop
                    cancel_requested.store(true, Ordering::SeqCst);
                }
            }

            input::handle_event(&mut app, event);
        }

        // Check if shutdown was requested
        if shutdown_requested.load(Ordering::SeqCst) {
            tracing::info!("Shutdown requested, flushing telemetry...");

            // Flush telemetry before exit
            crate::telemetry::flush();

            // Give time for graceful shutdown
            break;
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
                    let count = app.results.len();
                    app.add_toast(
                        format!("Found {} packages", count),
                        crate::animations::ToastStyle::Info,
                    );
                }
                ActionResult::Error(msg) => {
                    tracing::error!("Error received: {}", msg);
                    let notifier = DesktopNotifier::new();
                    notifier.notify_error(&msg);
                    crate::telemetry::append_log_line(&format!("[error] {}", msg));
                    app.error_message = Some(msg.clone());
                    app.is_loading = false;
                    app.add_toast(msg, crate::animations::ToastStyle::Error);
                    if app.is_operation_running {
                        if let Some(mut tx) = app.current_transaction.take() {
                            tx.status = TransactionStatus::Failed;
                            tx.error = app.error_message.clone();
                            app.transaction_history.push_front(tx);
                            while app.transaction_history.len() > 100 {
                                app.transaction_history.pop_back();
                            }
                            let snapshot: Vec<_> =
                                app.transaction_history.iter().cloned().collect();
                            let _ = save_history(&snapshot);
                        }
                    }
                    app.is_operation_running = false;
                }
                ActionResult::TransactionFailedWithRollback(msg, snapshot_id) => {
                    tracing::error!(
                        "Transaction failed: {}. Snapshot {} is available.",
                        msg,
                        snapshot_id
                    );
                    app.error_message = Some(msg.clone());
                    app.is_loading = false;
                    app.is_operation_running = false;
                    app.show_rollback_confirm = true;
                    app.pending_rollback_id = Some(snapshot_id);
                    app.add_toast(
                        format!("Failed: {}", msg),
                        crate::animations::ToastStyle::Error,
                    );
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
                    crate::telemetry::append_log_line(&line);
                    app.add_console_output(line);
                }
                ActionResult::CommandStarted => {
                    app.is_operation_running = true;
                    if let Some(tx) = &app.current_transaction {
                        crate::telemetry::append_log_line(&format!("[tx-start] {}", tx.id));
                    }
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
                    app.add_toast(
                        "Operation completed successfully!".to_string(),
                        crate::animations::ToastStyle::Success,
                    );

                    let notifier = DesktopNotifier::new();
                    let _ = notifier.send("metapak", "Operation completed successfully!");

                    if let Some(mut tx) = app.current_transaction.take() {
                        tx.status = TransactionStatus::Success;
                        app.transaction_history.push_front(tx);
                        while app.transaction_history.len() > 100 {
                            app.transaction_history.pop_back();
                        }
                        let snapshot: Vec<_> = app.transaction_history.iter().cloned().collect();
                        let _ = save_history(&snapshot);
                    }
                    app.add_console_output(
                        "----- Process Finished (Press 'q' or 'Esc' to close) -----".to_string(),
                    );
                }
                ActionResult::CommandCancelled => {
                    app.is_operation_running = false;
                    app.command_stdin_tx = None;
                    app.console_input.clear();
                    if let Some(mut tx) = app.current_transaction.take() {
                        tx.status = TransactionStatus::Cancelled;
                        app.transaction_history.push_front(tx);
                        while app.transaction_history.len() > 100 {
                            app.transaction_history.pop_back();
                        }
                        let snapshot: Vec<_> = app.transaction_history.iter().cloned().collect();
                        let _ = save_history(&snapshot);
                    }
                    app.add_console_output("----- Operation Cancelled -----".to_string());
                }
                ActionResult::UpdateCount(count) => {
                    app.available_updates = Some(count);
                    if count > 0 {
                        app.add_toast(
                            format!("{} updates available!", count),
                            crate::animations::ToastStyle::Warning,
                        );
                    }
                }
                ActionResult::RollbackFinished(id) => {
                    app.is_operation_running = false;
                    app.add_toast(
                        format!("Rollback to {} successful! Please reboot.", id),
                        crate::animations::ToastStyle::Success,
                    );
                    app.add_console_output(
                        "----- Rollback Finished (Please reboot your system) -----".to_string(),
                    );
                }
                ActionResult::SimulationResult(res) => {
                    app.simulation_result = Some(res);
                    app.show_simulation = true;
                    app.is_loading = false;
                }
                ActionResult::Cancelled => {
                    app.is_operation_running = false;
                    app.is_loading = false;
                }
                ActionResult::ViewModeSwitched(mode) => {
                    app.view_mode = mode;
                    app.add_toast(
                        format!("Switched to {:?} mode", mode),
                        crate::animations::ToastStyle::Info,
                    );
                }
                ActionResult::EcosystemSearchResults(pkgs) => {
                    app.ecosystem_results = pkgs;
                    app.is_loading = false;
                    app.error_message = None;
                    app.current_page = 0;
                    app.selected_index = if app.get_paginated_results().is_empty() {
                        None
                    } else {
                        Some(0)
                    };
                    let count = app.ecosystem_results.len();
                    app.add_toast(
                        format!("Found {} packages in ecosystem", count),
                        crate::animations::ToastStyle::Info,
                    );
                }
            }
        }

        // Periodic cleanup
        if last_update.elapsed() > Duration::from_secs(CLEANUP_INTERVAL_SECS) {
            last_update = std::time::Instant::now();
        }

        // Periodic update checks
        if last_update_check.elapsed() > Duration::from_secs(UPDATE_CHECK_INTERVAL_SECS) {
            if let Some(tx) = &app.action_tx {
                let _ = tx.send(Action::new(ActionInner::CheckUpdates));
            }
            last_update_check = std::time::Instant::now();
        }
    }

    // Restore terminal
    tracing::info!("Shutting down metapak");
    disable_raw_mode().map_err(crate::errors::AppError::Io)?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen).map_err(crate::errors::AppError::Io)?;
    terminal
        .show_cursor()
        .map_err(crate::errors::AppError::Io)?;

    Ok(())
}
