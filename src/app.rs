use crate::models::Package;
use crate::action::Action;
use tokio::sync::mpsc::UnboundedSender;
use std::collections::HashMap;

pub enum InputMode {
    Normal,
    Editing,
}

pub struct App {
    pub search_input: String,
    pub input_mode: InputMode,
    pub results: Vec<Package>,
    pub should_quit: bool,
    pub is_loading: bool,
    pub action_tx: Option<UnboundedSender<Action>>,
    pub selected_index: Option<usize>,
    pub pending_command: Option<(String, Vec<String>)>,
    pub error_message: Option<String>,
    pub available_updates: Option<usize>,
    // Password Prompt
    pub show_password_prompt: bool,
    pub password_input: String,
    
    // Batch Selection
    pub selected_packages: HashMap<String, Package>,

    // Confirmation
    pub show_confirm_prompt: bool,
    pub packages_pending_confirmation: Vec<Package>,
    
    // Console / Execution
    pub show_console: bool,
    pub console_buffer: Vec<String>,
    pub command_stdin_tx: Option<UnboundedSender<String>>,
}

impl App {
    pub fn new() -> Self {
        Self {
            search_input: String::new(),
            input_mode: InputMode::Normal,
            results: Vec::new(),
            should_quit: false,
            is_loading: false,
            action_tx: None,
            selected_index: None,
            pending_command: None,
            error_message: None,
            available_updates: None,
            
            // Start with password prompt
            show_password_prompt: true, 
            password_input: String::new(),
            
            // Batch Selection
            selected_packages: HashMap::new(),

            // Confirmation
            show_confirm_prompt: false,
            packages_pending_confirmation: Vec::new(),

            show_console: false,
            console_buffer: Vec::new(),
            command_stdin_tx: None,
        }
    }
    
    pub fn set_sender(&mut self, tx: UnboundedSender<Action>) {
        self.action_tx = Some(tx);
    }
    
    pub fn toggle_selection(&mut self) {
        if let Some(idx) = self.selected_index {
            if let Some(pkg) = self.results.get(idx) {
                if self.selected_packages.contains_key(&pkg.name) {
                    self.selected_packages.remove(&pkg.name);
                } else {
                    self.selected_packages.insert(pkg.name.clone(), pkg.clone());
                }
            }
        }
    }

    pub fn next(&mut self) {
        if self.results.is_empty() {
            return;
        }
        let i = match self.selected_index {
            Some(i) => {
                if i >= self.results.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.selected_index = Some(i);
    }

    pub fn previous(&mut self) {
        if self.results.is_empty() {
            return;
        }
        let i = match self.selected_index {
            Some(i) => {
                if i == 0 {
                    self.results.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.selected_index = Some(i);
    }
}
