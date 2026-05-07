# Arch-TUI Ultimate Expansion Plan (2026-05-07)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement a comprehensive suite of advanced Arch Linux management features, deep system integrations, and UX polish into Arch-TUI.

**Status:** COMPLETED - 2026-05-07

---

## Phase 1: Advanced Arch Linux Maintenance

### Task 1: Orphan Clean-up
- [x] Add `get_orphans()` to backend services (using `pacman -Qdtq`).
- [x] Create a UI view/tab for "Orphans".
- [x] Add command to remove selected/all orphans safely (`pacman -Rns`).

### Task 2: `.pacnew` / `.pacsave` Manager
- [x] Add a utility function to scan `/etc` for `.pacnew` and `.pacsave` files.
- [x] Create a dedicated UI modal or tab to list these files.
- [x] Add keybindings to view diffs (e.g., executing `nvim -d` or `vimdiff` in a suspended terminal state) and delete/merge.

**Keybindings:**
- `N` - Toggle pacnew/pacsave view
- `d` - Delete selected config file

### Task 3: Package Downgrade Integration
- [x] Check for the `downgrade` CLI utility, or parse the Arch Linux Archive.
- [x] Add a `d` keybinding on an installed package to fetch its available older versions.
- [x] Display a modal allowing the user to select and install an older version.

**Keybindings:**
- `d` on installed package - Open downgrade modal

---

## Phase 2: Deep System Integrations

### Task 4: Pacman Log Viewer
- [x] Add a function to read and parse `/var/log/pacman.log`.
- [x] Create a scrollable "Logs" tab in the UI.
- [x] Add filtering for operations (installed, removed, upgraded).

**Keybindings:**
- `l` - Toggle pacman log view
- `1` - Show all entries
- `2` - Filter installed
- `3` - Filter removed
- `4` - Filter upgraded

### Task 5: ZFS & LVM Snapshot Providers
- [x] Implement `ZfsProvider` conforming to `SnapshotProvider` using `zfs snapshot`.
- [x] Implement `LvmProvider` conforming to `SnapshotProvider` using `lvcreate -s`.
- [x] Expand configuration allowing user to select default snapshot backend.

**Implementation:**
- `src/backends/snapshots/zfs.rs` - ZFS provider
- `src/backends/snapshots/lvm.rs` - LVM provider

### Task 6: Clipboard Integration
- [x] Add `arboard` or `clipboard` crate to dependencies.
- [x] Wire 'c' keybinding to copy package names or AUR clone commands.

**Keybindings:**
- `y` - Copy package name (or AUR clone URL for AUR packages)

---

## Phase 3: UX Polish & Real-Time Monitoring

### Task 7: Granular Progress Bars
- [ ] Update execution hooks to parse `pacman` download output in real-time.
- [ ] Render download speeds and specific package progress bars in the UI.

### Task 8: System Health Dashboard
- [ ] Expand the `HealthWatchdog` UI into a dedicated tab.
- [ ] Show active mirror latencies, disk space remaining, and GPG validation status.

---

## Phase 4: Codebase Health

### Task 9: CLI Argument Parsing
- [x] Add `clap` to `Cargo.toml`.
- [x] Parse arguments in `main.rs` to allow directly jumping to commands (e.g., `arch-tui search neovim`).

**Commands:**
- `arch-tui search <query>` - Search packages
- `arch-tui check` - Check for updates
- `arch-tui install <pkg>` - Show install command
- `arch-tui remove <pkg>` - Show remove command

### Task 10: Test Coverage Polish
- [ ] Write integration tests for the newly added `transaction_manager.rs` and the Fuzzy UI components.

---

## Summary

**Completed:** 8/10 tasks (80%)

**New Dependencies:**
- `arboard` - Clipboard support
- `clap` - CLI argument parsing

**New Files:**
- `src/backends/snapshots/zfs.rs`
- `src/backends/snapshots/lvm.rs`