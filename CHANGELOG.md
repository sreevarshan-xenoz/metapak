# Changelog

## [0.1.0] - 2026-05-28

### Added
- Cross-platform support for Linux (pacman, apt, dnf, zypper, apk), macOS (brew), Windows (scoop)
- Unified search across system repositories and language ecosystems (npm, pip, cargo)
- Batch package operations (select multiple packages with Tab, install/remove in one transaction)
- Live debounced search with fuzzy matching and query syntax (AND, OR, NOT, filters)
- Multiple TUI views: Updates, Diagnostics, System Info, Orphans, Package Sizes, Cache, Foreign Packages, Package Groups, Dependency Tree
- Runtime theme switching (Catppuccin Mocha/Latte, Dark, Light)
- Internationalization with auto-detection (en, es, fr, de, zh, ja)
- Pre/post operation hooks (custom shell commands before/after install, remove, update)
- Desktop notifications for install, update, and error events
- Operation telemetry with log rotation
- btrfs snapshot provider with rollback support
- Timeshift snapshot provider
- Simulation engine for dry-run testing
- Circuit breaker for AUR API reliability
- Configurable keyboard bindings
- System backup and restore (export/import explicit package list)
- Startup sudo password caching
- Embedded console for viewing build/install logs
- Toast notifications and animations
- Secure password handling (secrecy crate)
- Search history and undo support
- npm, pip, and cargo ecosystem package backends
- Scoop backend for Windows
- CLI interface with subcommands (search, check, install, remove)
- Search query pre-fill via `--search` flag
- Universal installer scripts (install.sh, install.ps1)

### Changed
- Project renamed from "arch-tui" to "metapak"
- Configuration directory migrated to ~/.config/metapak/
- Snapshot prefix updated from `arch-tui-` to `metapak-` with backward compatibility
- Desktop entry renamed to metapak.desktop

### Fixed
- Configuration path fallback for existing arch-tui directories
- Dead keybindings module removed
- Stub operation_queue module removed
- CLI --search flag properly wired to pre-fill search input
- Orphaned app_test.rs integrated as declared test module
