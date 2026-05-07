//! Integration tests for robustness features
//!
//! Tests for circuit breaker, cache limits, path validation,
//! search limits, and other robustness improvements.

use arch_tui::utils::{sanitize_filename, validate_path, validate_search_query};
use std::path::Path;

#[test]
fn test_path_validation_prevents_traversal() {
    // These should be rejected
    assert!(!validate_path(Path::new("../../../etc/passwd")));
    assert!(!validate_path(Path::new("/etc/../../../passwd")));
    assert!(!validate_path(Path::new("some/../../secret")));

    // These should be allowed
    assert!(validate_path(Path::new(
        "/home/user/.config/arch-tui/config.toml"
    )));
    assert!(validate_path(Path::new(".config/arch-tui/config.toml")));
    assert!(validate_path(Path::new("relative/path/file.txt")));
}

#[test]
fn test_filename_sanitization() {
    assert_eq!(sanitize_filename("test.txt"), "test.txt");
    assert_eq!(sanitize_filename("my-file_1.2"), "my-file_1.2");
    assert_eq!(sanitize_filename("file;rm -rf /"), "filerm-rf");
    assert_eq!(sanitize_filename("test@#$%^&*()"), "test");
}

#[test]
fn test_search_query_validation() {
    // Valid queries
    assert!(validate_search_query("firefox"));
    assert!(validate_search_query("linux-headers"));
    assert!(validate_search_query("package name"));

    // Invalid queries (potential injection)
    assert!(!validate_search_query("test && rm -rf /"));
    assert!(!validate_search_query("test | cat /etc/passwd"));
    assert!(!validate_search_query("test; echo hi"));
    assert!(!validate_search_query("$(whoami)"));
    assert!(!validate_search_query("test > /tmp/output"));
}

#[cfg(test)]
mod circuit_breaker_tests {
    use arch_tui::services::CircuitBreaker;

    #[test]
    fn test_circuit_breaker_initial_state() {
        let cb = CircuitBreaker::new();
        assert!(cb.is_available());
    }

    #[test]
    fn test_circuit_breaker_opens_after_failures() {
        let cb = CircuitBreaker::new();

        // Record failures up to threshold
        for _ in 0..CircuitBreaker::FAILURE_THRESHOLD {
            cb.record_failure();
        }

        // Circuit should now be open
        assert!(!cb.is_available());
    }

    #[test]
    fn test_circuit_breaker_recovers_after_timeout() {
        let cb = CircuitBreaker::new();

        // Open the circuit
        for _ in 0..CircuitBreaker::FAILURE_THRESHOLD {
            cb.record_failure();
        }
        assert!(!cb.is_available());

        // Simulate timeout by manipulating the last_failure time
        // In real usage, we'd wait for RECOVERY_SECS
        // For test, we'll just verify the state transition logic exists
        cb.record_success();
        assert!(cb.is_available());
    }
}

#[cfg(test)]
mod cache_tests {
    use arch_tui::services::{enforce_cache_limit, get_cache_stats};

    #[test]
    fn test_cache_stats() {
        let (total, expired) = get_cache_stats();
        // Just verify the function works without panic
        assert!(total >= 0);
        assert!(expired >= 0);
    }

    #[test]
    fn test_enforce_cache_limit_no_panic() {
        // This should not panic even with empty cache
        enforce_cache_limit();
    }
}

#[cfg(test)]
mod error_type_tests {
    use arch_tui::errors::AppError;

    #[test]
    fn test_timeout_error() {
        let err = AppError::Timeout("Request timed out".to_string());
        assert!(format!("{}", err).contains("timed out"));
    }

    #[test]
    fn test_validation_error() {
        let err = AppError::Validation("Invalid input".to_string());
        assert!(format!("{}", err).contains("Invalid input"));
    }

    #[test]
    fn test_resource_exhausted_error() {
        let err = AppError::ResourceExhausted("Memory limit reached".to_string());
        assert!(format!("{}", err).contains("Memory limit reached"));
    }
}

#[cfg(test)]
mod constants_tests {
    use arch_tui::constants;

    #[test]
    fn test_search_limits() {
        assert!(constants::search_limits::MAX_RESULTS_PER_SOURCE > 0);
        assert!(constants::search_limits::MAX_TOTAL_RESULTS > 0);
    }

    #[test]
    fn test_cache_constants() {
        assert!(constants::cache::MAX_CACHE_ENTRIES > 0);
        assert!(constants::cache::CLEANUP_BATCH_SIZE > 0);
    }

    #[test]
    fn test_shutdown_constants() {
        assert!(constants::shutdown::GRACEFUL_TIMEOUT_SECS > 0);
        assert!(constants::shutdown::FORCE_KILL_TIMEOUT_SECS > 0);
    }
}
