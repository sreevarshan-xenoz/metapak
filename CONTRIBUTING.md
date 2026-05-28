# Contributing to metapak

## Development Setup

### Prerequisites
- Rust toolchain (`rustup`, `cargo`, `rustc`)
- Git

### Getting Started

```bash
git clone https://github.com/sreevarshan-xenoz/metapak.git
cd metapak
cargo build
cargo test
```

## Project Structure

```
src/
├── main.rs              # Entry point, CLI (clap), event loop
├── lib.rs               # Public module declarations
├── app.rs               # App state, modes, action handling
├── ui.rs                # Ratatui rendering (largest module)
├── services.rs          # PackageService — search, install, remove orchestration
├── backends/            # Package provider implementations
├── config.rs            # TOML config loading and merging
├── traits.rs            # PackageProvider, UpdateProvider, PackageSimulator traits
├── search.rs            # EnhancedSearch — fuzzy, query syntax, filters
├── transaction_manager.rs # Safe system modification orchestration
├── hooks.rs             # Pre/post command hooks
├── i18n.rs              # Internationalization
├── theme.rs             # Dynamic theming
├── simulation.rs        # Dry-run simulation
├── watchdog.rs          # Health monitoring
└── ...
```

## Code Conventions

- **Async First**: Use `tokio::spawn` for non-blocking I/O. Backend operations should be async.
- **Trait-Based Backends**: Implement `PackageProvider` from `traits.rs` to add a new package manager.
- **Action-Driven State**: UI state updates via `Action` enums sent through async channels. Never mutate UI state directly from background tasks.
- **Error Handling**: Use `thiserror` for library-level errors, `anyhow` for application context. Custom errors in `errors.rs`.
- **Config**: Defaults in `config/default.toml`, user overrides in `~/.config/metapak/config.toml`.

## Testing

```bash
cargo test              # Run all tests
cargo test -- --nocapture # See test output
cargo clippy            # Lint checks
cargo fmt --check       # Formatting check
```

- Unit tests live inside source files (e.g., `src/config.rs` has inline tests)
- Integration tests are in `tests/`
- Use the `SimulationEngine` for testing UI flows without modifying the system
- Tests may require specific package managers to be installed for full coverage

## Pull Request Process

1. Fork the repository and create a feature branch (`git checkout -b feature/amazing-feature`)
2. Make your changes following the conventions above
3. Run `cargo test && cargo clippy && cargo fmt --check` to verify
4. Commit with descriptive messages (`git commit -m "feat: add amazing feature"`)
5. Push to your fork and open a Pull Request
6. Ensure the CI pipeline passes (fmt, clippy, test on Linux/macOS/Windows)

## CI Pipeline

On every push and pull request:
1. `cargo fmt --check` — formatting
2. `cargo clippy --all-targets -- -D warnings` — lint (warnings are errors)
3. `cargo test` — all tests

## Adding a Package Manager Backend

1. Create a new file or add to `src/backends/mod.rs`
2. Implement `PackageProvider` (and optionally `UpdateProvider`) from `src/traits.rs`
3. Register the backend in `src/services.rs` for auto-detection
4. Add tests for the new backend
5. Run `cargo test && cargo clippy` to verify

Then commit it:
```bash
git add CONTRIBUTING.md
git commit -m "docs: add CONTRIBUTING.md for developer onboarding"
```
