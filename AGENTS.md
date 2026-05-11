# AGENTS.md

## Build Commands

```bash
cargo build          # Debug build
cargo build --release  # Release build (required for install)
cargo run            # Run in development
cargo test          # Run tests
cargo fmt          # Format code
cargo clippy        # Lint
```

## CI Pipeline

Runs on every push/PR: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`

## Architecture

- **Entry point**: `src/main.rs`
- **App state**: `src/app.rs`
- **UI rendering**: `src/ui.rs`
- **Config**: `src/config.rs`, loads from `~/.config/metapak/config.toml` (includes `[hooks]`, `[telemetry]`, etc.)
- **Package service**: `src/services.rs` (unified search across multiple backends)

## Key Constraints

- Cross-platform support: Windows (Scoop), macOS (Homebrew), Linux (Pacman, APT, DNF, APK, etc.)
- Requires sudo for system-level package operations, but works without sudo for ecosystem/user-level (Scoop, NPM, Cargo)
- Tests require relevant package managers to run comprehensively

## Special Notes

- Debounced search: 300ms delay before executing (configurable)
- Async search uses tokio + reqwest for AUR RPC
- Command execution has automatic retry logic for db locks, dependency errors