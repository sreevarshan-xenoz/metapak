# Design Doc: Project Rename to metapak

**Topic:** renaming the Arch TUI project to "metapak" across all codebase identifiers, files, and system paths.

## 1. Overview
The project is being rebranded from "Arch TUI" to "metapak". This involves a comprehensive rename of the binary, configuration directories, snapshot prefixes, and all internal strings.

## 2. Goals
- Rename the package and binary to `metapak`.
- Update all human-readable references to `metapak`.
- Migrate configuration directories from `arch-tui` to `metapak`.
- Update snapshot prefixes while maintaining backward compatibility for existing snapshots.
- Rename related system files (e.g., `.desktop` files).

## 3. Identifiers Mapping
- **Crate Name:** `arch-tui` -> `metapak`
- **Binary Name:** `arch-tui` -> `metapak`
- **Human Brand:** `Arch TUI` -> `metapak`
- **Configuration Directory:** `~/.config/arch-tui` -> `~/.config/metapak`
- **Snapshot Prefix:** `arch-tui-` -> `metapak-`
- **Desktop Entry:** `arch-tui.desktop` -> `metapak.desktop`
- **AUR URL:** `https://github.com/sreevarshan-xenoz/arch-tui` -> `https://github.com/sreevarshan-xenoz/metapak` (Update where applicable)

## 4. Architecture & Components

### 4.1 Build System (`Cargo.toml`, `Cargo.lock`)
- Update `[package] name` to `metapak`.
- Run `cargo build` to update `Cargo.lock`.

### 4.2 Path Resolution (`src/config.rs`, `src/state.rs`, etc.)
- Update constants or helper functions that determine configuration and data paths.
- **Migration Logic:**
  - On startup, check if `~/.config/metapak` exists.
  - If it does NOT exist and `~/.config/arch-tui` DOES exist, perform a recursive copy/rename of the directory.
  - Ensure all related paths (logs, sessions, transactions) use the new base directory.

### 4.3 Snapshot Providers (`src/backends/snapshots/`)
- Update `Btrfs`, `Zfs`, `Lvm`, and `Timeshift` providers to use `metapak-` as the default prefix for new snapshots.
- **Backward Compatibility:** Update the `list` logic to filter for both `metapak-` and `arch-tui-` prefixes so existing snapshots remain visible and manageable.

### 4.3 UI & Internationalization (`src/i18n.rs`, `src/ui.rs`)
- Replace "Arch TUI" with "metapak" in all localized strings.
- Update the app title and success notifications.

### 4.4 System Integration
- Rename `arch-tui.desktop` to `metapak.desktop`.
- Update `PKGBUILD` and `install.sh` to reflect new binary and file names.

## 5. Implementation Strategy
1. **Research Phase:** Final verification of all string occurrences.
2. **Surgical Renames:** Update `Cargo.toml` and internal strings.
3. **File System Changes:** Rename the `.desktop` file and update script references.
4. **Migration Implementation:** Add the config migration logic to the startup sequence.
5. **Validation:** Run tests and verify path resolution.

## 6. Testing
- Verify that the binary compiles and runs as `metapak`.
- Verify that configuration is migrated correctly from `~/.config/arch-tui`.
- Verify that existing `arch-tui-` snapshots are still listed in the UI.
- Verify that new snapshots are created with the `metapak-` prefix.
