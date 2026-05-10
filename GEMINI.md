# metapak - Developer Context

metapak is a modern, terminal-based user interface for managing packages on Arch Linux (and other distributions). It provides a unified experience for searching, installing, and removing packages across multiple package managers (pacman, AUR helpers, apt, brew, etc.).

## 🚀 Project Overview

- **Main Technologies:** Rust, Ratatui (TUI rendering), Tokio (Async runtime), Crossterm (Terminal handling), Reqwest (HTTP client).
- **Core Purpose:** To provide a fast, safe, and visually appealing way to manage system packages directly from the terminal.
- **Key Features:** Unified search (Official Repos + AUR), batch operations, live search, system diagnostics, btrfs snapshots/rollbacks, and a robustness suite.

## 🛠️ Architecture

The project follows a modular architecture designed for extensibility and safety:

- **`src/main.rs`**: Entry point. Sets up the terminal, initializes the application state, spawns background tasks for async actions, and manages the main event loop.
- **`src/app.rs`**: Contains the `App` struct, which holds the entire application state (results, selection, loading state, history, etc.).
- **`src/backends/`**: Implements the `UniversalPackageManager` trait for different package managers (`pacman`, `apt`, `brew`, `winget`, etc.) and snapshot providers (`btrfs`).
- **`src/transaction_manager.rs`**: Orchestrates safe system modifications, handling pre-operation snapshots and potential rollbacks.
- **`src/ui.rs`**: Modularized UI components using Ratatui.
- **`src/action.rs` & `src/models.rs`**: Define the internal message passing (Actions) and data structures (Packages).

## 🏗️ Building and Running

### Prerequisites
- Rust toolchain (`cargo`, `rustc`)
- System dependencies: `pacman`, `inotify` (on Linux)

### Commands
- **Run:** `cargo run`
- **Build:** `cargo build --release`
- **Test:** `cargo test`
- **Lint:** `cargo clippy`
- **Format:** `cargo fmt`

## 📝 Development Conventions

- **Async First:** Most I/O and backend operations are asynchronous using `tokio`. Use `tokio::spawn` for non-blocking tasks.
- **Trait-Based Backends:** New package managers should implement the `UniversalPackageManager` trait in `src/backends/`.
- **Action-Driven State:** The TUI state is updated via `Action` objects sent through an async channel. This keeps UI rendering decoupled from backend logic.
- **Robustness Suite:**
    - **Simulation:** Use `SimulationEngine` in `src/simulation.rs` to dry-run operations.
    - **Watchdog:** `HealthWatchdog` in `src/watchdog.rs` monitors application health.
    - **Snapshots:** Use `SnapshotProvider` in `src/backends/snapshots/` for filesystem safety.
- **Error Handling:** Use `thiserror` for library-level errors and `anyhow` for application-level context. Custom error types are in `src/errors.rs`.
- **Theming:** Centralized theme configuration in `src/theme.rs`.

## ⚙️ Configuration

- **Default Config:** `config/default.toml`
- **User Config:** `~/.config/metapak/config.toml`
- Configuration is loaded using the `config` crate and managed in `src/config.rs`.

## 🧪 Testing

- **Unit Tests:** Located within source files (e.g., `src/config.rs`).
- **Integration Tests:** Located in `tests/`, covering snapshots, robustness, and simulation.
- **Simulation:** A dedicated simulation engine allows testing UI flows without modifying the actual system.
