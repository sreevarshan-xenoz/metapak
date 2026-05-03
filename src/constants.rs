pub mod search {
    pub const DEBOUNCE_MS_DEFAULT: u64 = 300;
    pub const DEBOUNCE_MS_MIN: u64 = 50;
    pub const HISTORY_MAX_SIZE_DEFAULT: usize = 50;
    pub const UNDO_HISTORY_MAX_DEFAULT: usize = 20;
    pub const CACHE_TTL_SECONDS: u64 = 300;
}

pub mod ui {
    pub const ITEMS_PER_PAGE_DEFAULT: usize = 20;
    pub const CONSOLE_BUFFER_MAX_LINES: usize = 1000;
    pub const CAPTURED_OUTPUT_MAX_LINES: usize = 200;
    pub const MAX_TOASTS: usize = 3;
    pub const TOAST_MESSAGE_MAX_CHARS: usize = 60;
    pub const TICK_INTERVAL_MS: u64 = 33;
    pub const INPUT_POLL_TIMEOUT_MS: u64 = 50;
    pub const CLEANUP_INTERVAL_SECS: u64 = 30;
    pub const UPDATE_CHECK_INTERVAL_SECS: u64 = 900;
    pub const MIN_SIDEBAR_WIDTH: u16 = 100;
    pub const MIN_TERMINAL_HEIGHT: u16 = 20;
}

pub mod retry {
    pub const MAX_ATTEMPTS: usize = 3;
    pub const LOCK_RETRY_DELAY_SECS: u64 = 3;
    pub const NETWORK_RETRY_DELAY_SECS: u64 = 5;
    pub const GENERAL_RETRY_DELAY_SECS: u64 = 2;
    pub const AUR_RETRY_COUNT: usize = 3;
    pub const AUR_RETRY_BASE_DELAY_MS: u64 = 250;
}

pub mod network {
    pub const AUR_REQUEST_TIMEOUT_SECS: u64 = 8;
    pub const AUR_CONNECT_TIMEOUT_SECS: u64 = 4;
}

pub mod transaction {
    pub const MAX_HISTORY_SIZE: usize = 100;
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_search_constants() {
        assert!(search::DEBOUNCE_MS_DEFAULT >= search::DEBOUNCE_MS_MIN);
        assert!(search::HISTORY_MAX_SIZE_DEFAULT > 0);
        assert!(search::UNDO_HISTORY_MAX_DEFAULT > 0);
    }

    #[test]
    fn test_ui_constants() {
        assert!(ui::ITEMS_PER_PAGE_DEFAULT > 0);
        assert!(ui::CONSOLE_BUFFER_MAX_LINES > 0);
        assert!(ui::CAPTURED_OUTPUT_MAX_LINES > 0);
        assert!(ui::TICK_INTERVAL_MS > 0);
        assert!(ui::INPUT_POLL_TIMEOUT_MS > 0);
    }

    #[test]
    fn test_retry_constants() {
        assert!(retry::MAX_ATTEMPTS > 0);
        assert!(retry::LOCK_RETRY_DELAY_SECS > 0);
        assert!(retry::NETWORK_RETRY_DELAY_SECS > 0);
        assert!(retry::GENERAL_RETRY_DELAY_SECS > 0);
    }

    #[test]
    fn test_network_constants() {
        assert!(network::AUR_REQUEST_TIMEOUT_SECS > 0);
        assert!(network::AUR_CONNECT_TIMEOUT_SECS > 0);
    }
}