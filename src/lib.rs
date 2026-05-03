//! # Universal TUI
//!
//! A terminal user interface for managing packages across different
//! operating systems and package managers.
//!
//! ## Features
//!
//! - Multi-platform support (Linux, macOS, Windows)
//! - Multiple package manager backends (pacman, apt, dnf, zypper, brew, winget, chocolatey)
//! - Auto-detection of available package managers
//! - Unified search and package operations
//! - Batch operations for installing/removing multiple packages
//! - Interactive TUI with keyboard shortcuts
//! - Search debouncing and history
//! - Pagination for large result sets
//! - Filter and sort functionality
//! - Comprehensive theming system
//! - Secure password handling
//! - Undo functionality for selections
//! - Progress indicators
//! - Internationalization support
//! - Dependency visualization

pub mod action;
pub mod animations;
pub mod app;
pub mod backends;
pub mod command;
pub mod config;
pub mod constants;
pub mod dependency_visualization;
pub mod diagnostics;
pub mod errors;
pub mod export;
pub mod hooks;
pub mod i18n;
pub mod input;
pub mod keybindings;
pub mod models;
pub mod notifications;
pub mod operation_queue;
pub mod parallel;
pub mod platform;
pub mod search;
pub mod security;
pub mod services;
pub mod simulation;
pub mod telemetry;
pub mod theme;
pub mod traits;
pub mod transaction_history;
pub mod state;
pub mod ui;
pub mod ui_utils;
pub mod utils;
