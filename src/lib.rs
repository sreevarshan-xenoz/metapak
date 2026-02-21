//! # Arch TUI
//!
//! A terminal user interface for managing packages on Arch Linux.
//! This crate provides a unified interface for searching, installing,
//! and removing packages from both official repositories (pacman) and AUR.
//!
//! ## Features
//!
//! - Unified search across pacman repositories and AUR with caching
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
pub mod app;
pub mod config;
pub mod dependency_visualization;
pub mod errors;
pub mod i18n;
pub mod input;
pub mod models;
pub mod services;
pub mod theme;
pub mod traits;
pub mod ui;
pub mod ui_utils;
pub mod utils;
