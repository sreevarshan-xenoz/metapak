use crate::models::Package;

#[derive(Debug, Clone)]
pub enum Action {
    Search(String),
    InitSudo(String), 
    CheckUpdates,
    SystemUpdate,
    RunCommand { prog: String, args: Vec<String> },
    CommandInput(String),
}

#[derive(Debug)]
pub enum ActionResult {
    SearchResults(Vec<Package>),
    SudoResult(bool),
    CommandOutput(String),
    CommandFinished,
    UpdateCount(usize),
    Error(String),
}
