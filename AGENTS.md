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
- **Config**: `src/config.rs`, loads from `~/.config/arch-tui/config.toml`
- **Package service**: `src/services.rs` (unified search: pacman + AUR)

## Key Constraints

- Linux-only (requires pacman, AUR helper like `paru`/`yay` for full functionality)
- Requires sudo for package operations
- Tests require Linux environment

## Special Notes

- Debounced search: 300ms delay before executing (configurable)
- Async search uses tokio + reqwest for AUR RPC
- Command execution has automatic retry logic for db locks, dependency errors