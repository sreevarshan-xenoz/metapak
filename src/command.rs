use std::collections::VecDeque;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::sync::Mutex;

use crate::constants::ui::CAPTURED_OUTPUT_MAX_LINES;
use crate::constants::retry::{
    LOCK_RETRY_DELAY_SECS, MAX_ATTEMPTS, NETWORK_RETRY_DELAY_SECS, GENERAL_RETRY_DELAY_SECS,
};
use crate::services::CommandSpec;
use crate::action::ActionResult;

pub enum CommandRunResult {
    Finished,
    Cancelled,
}

pub struct CommandExecutor {
    active_pid: Arc<Mutex<Option<u32>>>,
    cancel_requested: Arc<AtomicBool>,
}

impl CommandExecutor {
    pub fn new() -> Self {
        Self {
            active_pid: Arc::new(Mutex::new(None)),
            cancel_requested: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn get_active_pid(&self) -> Arc<Mutex<Option<u32>>> {
        self.active_pid.clone()
    }

    pub fn get_cancel_flag(&self) -> Arc<AtomicBool> {
        self.cancel_requested.clone()
    }

    pub async fn run_sequence(
        &self,
        commands: Vec<CommandSpec>,
        tx: tokio::sync::mpsc::UnboundedSender<ActionResult>,
    ) -> CommandRunResult {
        let _ = tx.send(ActionResult::CommandStarted);

        for command in &commands {
            let mut attempts = 0usize;

            loop {
                attempts += 1;
                match self
                    .run_single(command, tx.clone())
                    .await
                {
                    Ok(CommandRunResult::Finished) => break,
                    Ok(CommandRunResult::Cancelled) => return CommandRunResult::Cancelled,
                    Err(err) => {
                        let error_lower = err.to_lowercase();
                        let is_dependency_error = error_lower.contains("dependency")
                            || error_lower.contains("[dependency-error]");
                        let is_lock_error = error_lower.contains("lock")
                            || error_lower.contains("[db-lock-error]");
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

                        if is_lock_error && attempts < MAX_ATTEMPTS {
                            let _ = tx.send(ActionResult::CommandOutput(format!(
                                "Detected pacman DB lock. Attempt {}/3. Waiting {}s...",
                                attempts, LOCK_RETRY_DELAY_SECS
                            )));
                            tokio::time::sleep(Duration::from_secs(LOCK_RETRY_DELAY_SECS)).await;
                            continue;
                        }

                        if is_dependency_error {
                            let _ = tx.send(ActionResult::CommandOutput(
                                "Dependency issue detected. Running: sudo pacman -Syu...".to_string(),
                            ));
                            let fix = CommandSpec {
                                prog: "sudo".to_string(),
                                args: vec![
                                    "pacman".to_string(),
                                    "-Syu".to_string(),
                                    "--noconfirm".to_string(),
                                ],
                            };
                            match self.run_single(&fix, tx.clone()).await {
                                Ok(CommandRunResult::Finished) => {
                                    let _ = tx.send(ActionResult::CommandOutput(
                                        "System upgrade complete. Retrying...".to_string(),
                                    ));
                                    if attempts < MAX_ATTEMPTS {
                                        continue;
                                    }
                                }
                                Ok(CommandRunResult::Cancelled) => {
                                    return CommandRunResult::Cancelled
                                }
                                Err(fix_err) => {
                                    let _ = tx.send(ActionResult::Error(format!(
                                        "System upgrade failed: {}",
                                        fix_err
                                    )));
                                    return CommandRunResult::Finished;
                                }
                            }
                        }

                        if is_conflict_error {
                            let _ = tx.send(ActionResult::CommandOutput(
                                "Package conflict detected. Trying to resolve...".to_string(),
                            ));
                            let fix = CommandSpec {
                                prog: "sudo".to_string(),
                                args: vec![
                                    "pacman".to_string(),
                                    "-Syu".to_string(),
                                    "--overwrite".to_string(),
                                    "*".to_string(),
                                    "--noconfirm".to_string(),
                                ],
                            };
                            match self.run_single(&fix, tx.clone()).await {
                                Ok(CommandRunResult::Finished) => {
                                    let _ = tx.send(ActionResult::CommandOutput(
                                        "Conflicts resolved.".to_string(),
                                    ));
                                    if attempts < MAX_ATTEMPTS {
                                        continue;
                                    }
                                }
                                Ok(CommandRunResult::Cancelled) => {
                                    return CommandRunResult::Cancelled
                                }
                                Err(fix_err) => {
                                    let _ = tx.send(ActionResult::Error(format!(
                                        "Conflict resolution failed: {}",
                                        fix_err
                                    )));
                                    return CommandRunResult::Finished;
                                }
                            }
                        }

                        if is_signature_error {
                            let _ = tx.send(ActionResult::CommandOutput(
                                "PGP signature issue detected. Attempting to refresh keys..."
                                    .to_string(),
                            ));
                            let fix = CommandSpec {
                                prog: "sudo".to_string(),
                                args: vec!["pacman-key".to_string(), "--init".to_string()],
                            };
                            let _ = self.run_single(&fix, tx.clone()).await;

                            let fix2 = CommandSpec {
                                prog: "sudo".to_string(),
                                args: vec![
                                    "pacman".to_string(),
                                    "-Sy".to_string(),
                                    "--noconfirm".to_string(),
                                ],
                            };
                            if let Ok(CommandRunResult::Finished) =
                                self.run_single(&fix2, tx.clone()).await
                            {
                                let _ = tx.send(ActionResult::CommandOutput(
                                    "Keys refreshed. Retrying...".to_string(),
                                ));
                                if attempts < MAX_ATTEMPTS {
                                    continue;
                                }
                            }
                        }

                        if is_disk_space_error {
                            let _ = tx.send(ActionResult::CommandOutput(
                                "Disk space issue. Try: sudo pacman -Scc to clean cache"
                                    .to_string(),
                            ));
                        }

                        if is_network_error && attempts < 2 {
                            let _ = tx.send(ActionResult::CommandOutput(format!(
                                "Network error. Retrying in {}s...",
                                NETWORK_RETRY_DELAY_SECS
                            )));
                            tokio::time::sleep(Duration::from_secs(NETWORK_RETRY_DELAY_SECS)).await;
                            continue;
                        }

                        if is_cache_error {
                            let _ = tx.send(ActionResult::CommandOutput(
                                "Cache issue. Cleaning local cache...".to_string(),
                            ));
                            let fix = CommandSpec {
                                prog: "rm".to_string(),
                                args: vec![
                                    "-rf".to_string(),
                                    "/var/cache/pacman/pkg/*".to_string(),
                                ],
                            };
                            let _ = self.run_single(&fix, tx.clone()).await;
                        }

                        if attempts >= MAX_ATTEMPTS {
                            let _ = tx.send(ActionResult::Error(format!(
                                "Failed after {} attempts: {}",
                                attempts, err
                            )));
                            return CommandRunResult::Finished;
                        }

                        let _ = tx.send(ActionResult::CommandOutput(format!(
                            "Error (attempt {}/{}): {}. Retrying...",
                            attempts, MAX_ATTEMPTS, err
                        )));
                        tokio::time::sleep(Duration::from_secs(GENERAL_RETRY_DELAY_SECS)).await;
                    }
                }
            }
        }

        CommandRunResult::Finished
    }

    pub async fn run_single(
        &self,
        command: &CommandSpec,
        tx: tokio::sync::mpsc::UnboundedSender<ActionResult>,
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
            let mut pid_guard = self.active_pid.lock().await;
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
            let mut pid_guard = self.active_pid.lock().await;
            *pid_guard = None;
        }

        if self.cancel_requested.swap(false, Ordering::SeqCst) {
            return Ok(CommandRunResult::Cancelled);
        }

        if status.success() {
            Ok(CommandRunResult::Finished)
        } else {
            let output: Vec<String> = captured_output.lock().await.iter().cloned().collect();
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

    pub fn request_cancel(&self) {
        self.cancel_requested.store(true, Ordering::SeqCst);
    }

    pub async fn kill_active_process(&self) {
        if let Some(pid) = *self.active_pid.lock().await {
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
    }
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