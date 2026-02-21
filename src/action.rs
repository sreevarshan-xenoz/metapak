use crate::models::Package;
use crate::services::CommandSpec;
use secrecy::SecretString;

/// Actions that can be sent from the UI to background tasks
#[derive(Debug, Clone)]
pub enum Action {
    /// Search for packages
    Search(String),

    /// Initialize sudo with password
    InitSudo(SecretString),

    /// Check for available system updates
    CheckUpdates,

    /// Perform system update
    SystemUpdate,

    /// Run a shell command
    RunCommand { prog: String, args: Vec<String> },

    /// Run multiple commands in sequence
    RunCommands(Vec<CommandSpec>),

    /// Cancel current operation
    CancelOperation,
}

/// Results returned from background tasks to the UI
#[derive(Debug)]
pub enum ActionResult {
    /// Search completed successfully
    SearchResults(Vec<Package>),

    /// Sudo authentication result
    SudoResult(bool),

    /// Command output line
    CommandOutput(String),

    /// Command completed
    CommandFinished,

    /// Command was cancelled
    CommandCancelled,

    /// Update check completed
    UpdateCount(usize),

    /// Operation cancelled by user
    Cancelled,

    /// Error occurred
    Error(String),
}
