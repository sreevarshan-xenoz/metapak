# Design Doc: Documentation Improvements

**Topic:** Expanding and restructuring metapak's documentation suite

## 1. Overview

The metapak project has grown significantly — cross-platform support, multiple backends, many features — but the documentation hasn't kept pace. The README is the primary (and nearly only) user-facing doc, and many source modules lack even a basic doc comment. This spec covers a comprehensive documentation overhaul.

## 2. Goals

- Expand README.md into a comprehensive landing page covering all user-facing aspects
- Create CHANGELOG.md from git history for release tracking
- Create CONTRIBUTING.md for developer onboarding
- Add module-level doc comments to all undocumented source modules
- Ensure all docs are accurate, consistent, and well-organized

## 3. Files to Create/Modify

### 3.1 README.md (modify — comprehensive expansion)

Current sections to keep/polish:
- Title + badges
- Project overview description
- Key Features (condense — deduplicate between Features and System Views lists)

New sections to add:
- **Quick Start / Installation** — keep existing but improve with OS-specific tabs/badges
- **Usage** — expand controls table, add CLI flags reference, add filter/sort reference
- **Configuration Reference** — full explanation of every `config.toml` section (theme, aur, keyboard, ui, search, robustness, hooks, i18n, notifications, telemetry) with examples
- **View Reference** — detailed walkthrough of each view: Updates, Diagnostics, System Info, Orphans, Package Sizes, Cache, Foreign Packages, Package Groups, Dependency Tree
- **Hooks & Automation** — how pre/post hooks work, example configurations
- **i18n** — supported languages, auto-detection, how to configure
- **Backup & Recovery** — already exists briefly, expand with examples
- **Troubleshooting FAQ** — common issues: search not working, sudo password, AUR helper detection, empty results, permission errors, cross-platform notes
- **Architecture** — already exists, enhance with module directory tree and data flow description

### 3.2 CHANGELOG.md (create new)

Generated from `git log --oneline`, organized chronologically. Simple format:
```
# Changelog

## [0.1.0] - 2026-05-28

### Added
- Cross-platform support (Windows/Scoop, macOS/Brew, Linux)
- Unified search across system repos and language ecosystems
- ...
```

### 3.3 CONTRIBUTING.md (create new)

Sections:
- **Development Setup** — clone, build, required tooling
- **Project Structure** — directory tree overview
- **Code Conventions** — async-first, trait-based backends, action-driven state, error handling patterns
- **Making Changes** — branch naming, commit style, PR workflow
- **Testing** — running tests, writing tests, simulation engine
- **CI Pipeline** — what runs on push/PR
- **Adding a Package Manager Backend** — step-by-step guide for implementing `PackageProvider`

### 3.4 Source module doc comments (modify 21 files)

Add `//!` module-level doc comments (2-6 lines each) to these files:
- `src/main.rs` — entry point, CLI parsing, event loop
- `src/action.rs` — internal action/message types for state updates
- `src/config.rs` — configuration loading from TOML files
- `src/constants.rs` — application-wide constants
- `src/errors.rs` — custom error types using thiserror
- `src/models.rs` — core data types (Package, PackageSource, etc.)
- `src/diagnostics.rs` — system diagnostics collection
- `src/hooks.rs` — pre/post command hooks execution
- `src/input.rs` — keyboard input handling and mapping
- `src/notifications.rs` — desktop notifications
- `src/simulation.rs` — dry-run simulation engine
- `src/telemetry.rs` — operation logging with rotation
- `src/transaction_history.rs` — session history tracking
- `src/transaction_manager.rs` — safe orchestration of system modifications
- `src/ui.rs` — main TUI rendering
- `src/ui_utils.rs` — shared UI rendering helpers
- `src/watchdog.rs` — health monitoring circuit breaker
- `src/app_test.rs` — test utilities (not a public module)
- `src/backends/snapshots/mod.rs` — snapshot provider subsystem
- `src/backends/snapshots/btrfs.rs` — btrfs snapshot provider
- `src/backends/snapshots/timeshift.rs` — Timeshift snapshot provider

## 4. Approach

- All doc files to use clear, concise English with consistent formatting
- README.md to serve as entry point; CHANGELOG.md and CONTRIBUTING.md as sibling companion docs
- Module doc comments to follow the existing pattern: 2-6 lines describing purpose, key types, and typical usage
- No code changes — documentation only
- Run `cargo doc --no-deps` after adding module docs to verify no warnings

## 5. Success Criteria

- `cargo doc --no-deps` completes without warnings
- Every `.rs` file in `src/` starts with a meaningful `//!` doc comment
- README.md covers: install, usage, config reference, view reference, troubleshooting, CLI flags
- CHANGELOG.md contains accurate project history from git
- CONTRIBUTING.md enables a new developer to set up and contribute in under 15 minutes
