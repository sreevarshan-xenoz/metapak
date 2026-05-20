#![cfg_attr(not(test), allow(dead_code))]

pub mod search {
    pub const DEBOUNCE_MS_DEFAULT: u64 = 300;
    pub const HISTORY_MAX_SIZE_DEFAULT: usize = 50;
    pub const UNDO_HISTORY_MAX_DEFAULT: usize = 20;
    
}

pub mod ui {
    pub const ITEMS_PER_PAGE_DEFAULT: usize = 20;
    pub const CONSOLE_BUFFER_MAX_LINES: usize = 1000;
    pub const CAPTURED_OUTPUT_MAX_LINES: usize = 200;
    pub const TICK_INTERVAL_MS: u64 = 16;
    pub const INPUT_POLL_TIMEOUT_MS: u64 = 16;
    pub const CLEANUP_INTERVAL_SECS: u64 = 30;
    pub const UPDATE_CHECK_INTERVAL_SECS: u64 = 900;
}

pub mod retry {
    pub const MAX_ATTEMPTS: usize = 3;
    pub const LOCK_RETRY_DELAY_SECS: u64 = 3;
    pub const NETWORK_RETRY_DELAY_SECS: u64 = 5;
    pub const GENERAL_RETRY_DELAY_SECS: u64 = 2;
}

pub mod network {
    pub const AUR_REQUEST_TIMEOUT_SECS: u64 = 8;
    pub const AUR_CONNECT_TIMEOUT_SECS: u64 = 4;
    pub const HTTP_MAX_CONNECTIONS: u32 = 10;
    pub const HTTP_IDLE_TIMEOUT_SECS: u64 = 30;
}

pub mod search_limits {
    pub const MAX_TOTAL_RESULTS: usize = 1000;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_constants() {
        let _ = search::DEBOUNCE_MS_DEFAULT;
        let _ = search::HISTORY_MAX_SIZE_DEFAULT;
        let _ = search::UNDO_HISTORY_MAX_DEFAULT;
    }

    #[test]
    fn test_ui_constants() {
        let _ = ui::ITEMS_PER_PAGE_DEFAULT;
        let _ = ui::CONSOLE_BUFFER_MAX_LINES;
        let _ = ui::CAPTURED_OUTPUT_MAX_LINES;
        let _ = ui::TICK_INTERVAL_MS;
        let _ = ui::INPUT_POLL_TIMEOUT_MS;
    }

    #[test]
    fn test_retry_constants() {
        let _ = retry::MAX_ATTEMPTS;
        let _ = retry::LOCK_RETRY_DELAY_SECS;
        let _ = retry::NETWORK_RETRY_DELAY_SECS;
        let _ = retry::GENERAL_RETRY_DELAY_SECS;
    }

    #[test]
    fn test_network_constants() {
        let _ = network::AUR_REQUEST_TIMEOUT_SECS;
        let _ = network::AUR_CONNECT_TIMEOUT_SECS;
    }
}
