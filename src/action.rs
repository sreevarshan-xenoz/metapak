use crate::models::Package;
use crate::services::CommandSpec;
use secrecy::SecretString;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::sync::mpsc::UnboundedSender;

/// Actions that can be sent from the UI to background tasks
#[derive(Debug, Clone)]
pub struct Action {
    /// Unique request ID for tracing
    pub id: u32,
    /// The actual action
    pub inner: ActionInner,
}

/// Inner action types
#[derive(Debug, Clone)]
pub enum ActionInner {
    /// Search for packages
    Search(String),

    /// Initialize sudo with password
    InitSudo(SecretString),

    /// Check for available system updates
    CheckUpdates,

    /// Perform system update
    SystemUpdate,

    /// Rollback to a specific snapshot ID
    Rollback(String),

    /// Simulate a command sequence
    Simulate(Vec<CommandSpec>),

    /// Run multiple commands in sequence
    RunCommands(Vec<CommandSpec>),

    /// Cancel current operation
    CancelOperation,
}

static ACTION_ID_COUNTER: AtomicU32 = AtomicU32::new(0);

impl Action {
    /// Create a new Action with a unique ID
    pub fn new(inner: ActionInner) -> Self {
        let id = ACTION_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        Self { id, inner }
    }

    /// Get the request ID
    pub fn id(&self) -> u32 {
        self.id
    }
}

/// For backward compatibility and easier pattern matching
impl std::ops::Deref for Action {
    type Target = ActionInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
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

    /// Command execution has started
    CommandStarted,

    /// Command input channel is ready
    CommandInputReady(UnboundedSender<String>),

    /// Command input channel is closed
    CommandInputClosed,

    /// Command completed
    CommandFinished,

    /// Command was cancelled
    CommandCancelled,

    /// Update check completed
    UpdateCount(usize),

    /// Simulation result
    SimulationResult(crate::traits::SimulationResult),

    /// Operation cancelled by user
    Cancelled,

    /// Error occurred
    Error(String),
}
