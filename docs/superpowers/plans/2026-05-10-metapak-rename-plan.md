# metapak Project Rename Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rename the project from "Arch TUI" to "metapak" across the codebase, configuration, and system files, including a one-time configuration migration and backward-compatible snapshot detection.

**Architecture:** 
- Update `Cargo.toml` and internal name constants.
- Implement migration logic in `src/config.rs` to move `~/.config/arch-tui` to `~/.config/metapak`.
- Update snapshot providers to use `metapak-` prefix for new snapshots while still recognizing `arch-tui-`.
- Perform a comprehensive string replacement for "Arch TUI" and "arch-tui".

**Tech Stack:** Rust, Cargo, Shell Scripts

---

### Task 1: Project Metadata & Binary Rename

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/main.rs`

- [ ] **Step 1: Update Cargo.toml package name**
Change `name = "arch-tui"` to `name = "metapak"`.

- [ ] **Step 2: Update binary name in main.rs**
In `src/main.rs`, update the `clap` command name.
```rust
// src/main.rs
#[command(name = "metapak")]
```

- [ ] **Step 3: Update Cargo.lock**
Run: `cargo build`
Expected: `Cargo.lock` updated with the new package name.

- [ ] **Step 4: Commit**
```bash
git add Cargo.toml Cargo.lock src/main.rs
git commit -m "chore: rename package and binary to metapak"
```

---

### Task 2: Path Migration Logic

**Files:**
- Modify: `src/config.rs`

- [ ] **Step 1: Update config directory constant and add migration logic**
Modify `src/config.rs` to change the default directory and implement a migration function.

```rust
// src/config.rs - update find_config_file or similar path logic
pub fn get_config_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".config").join("metapak")
}

pub fn migrate_config() -> std::io::Result<()> {
    let old_dir = dirs::home_dir().map(|h| h.join(".config").join("arch-tui"));
    let new_dir = get_config_dir();

    if let Some(old) = old_dir {
        if old.exists() && !new_dir.exists() {
            tracing::info!("Migrating configuration from arch-tui to metapak");
            std::fs::create_dir_all(new_dir.parent().unwrap())?;
            // Using a simple rename or recursive copy
            std::fs::rename(&old, &new_dir)?;
        }
    }
    Ok(())
}
```
*Note: Ensure `migrate_config()` is called early in `main()`.*

- [ ] **Step 2: Verify migration logic**
Run: `cargo test` (ensure existing config tests pass or are updated).

- [ ] **Step 3: Commit**
```bash
git add src/config.rs
git commit -m "feat: implement config migration from arch-tui to metapak"
```

---

### Task 3: Updating Core State & Log Paths

**Files:**
- Modify: `src/state.rs`
- Modify: `src/telemetry.rs`
- Modify: `src/transaction_history.rs`
- Modify: `src/diagnostics.rs`

- [ ] **Step 1: Update session path in state.rs**
Replace `.join("arch-tui")` with `.join("metapak")`.

- [ ] **Step 2: Update log path in telemetry.rs**
Replace `.join("arch-tui")` with `.join("metapak")`.

- [ ] **Step 3: Update transactions path in transaction_history.rs**
Replace `.join("arch-tui")` with `.join("metapak")`.

- [ ] **Step 4: Update backup path in diagnostics.rs**
Replace `arch-tui/backups` with `metapak/backups`.

- [ ] **Step 5: Commit**
```bash
git add src/state.rs src/telemetry.rs src/transaction_history.rs src/diagnostics.rs
git commit -m "chore: update state, telemetry, and diagnostics paths to metapak"
```

---

### Task 4: Snapshot Prefix Update & Compatibility

**Files:**
- Modify: `src/backends/snapshots/btrfs.rs`
- Modify: `src/backends/snapshots/lvm.rs`
- Modify: `src/backends/snapshots/timeshift.rs`
- Modify: `src/backends/snapshots/zfs.rs`
- Modify: `tests/btrfs_snapshot_tests.rs`

- [ ] **Step 1: Update prefix in btrfs.rs**
Change `arch-tui-` to `metapak-` for new snapshots. Update filters to look for both.

- [ ] **Step 2: Update prefix in lvm.rs**
Change prefix to `metapak-snap`.

- [ ] **Step 3: Update prefix in timeshift.rs**
Update label comment to `metapak-`.

- [ ] **Step 4: Update prefix in zfs.rs**
Update snapshot name format to include `@metapak-`.

- [ ] **Step 5: Update tests**
Update `tests/btrfs_snapshot_tests.rs` to check for both prefixes.

- [ ] **Step 6: Commit**
```bash
git add src/backends/snapshots/ tests/btrfs_snapshot_tests.rs
git commit -m "feat: update snapshot prefixes to metapak- with backward compatibility"
```

---

### Task 5: Internationalization & Brand Strings

**Files:**
- Modify: `src/i18n.rs`
- Modify: `src/services.rs`
- Modify: `src/main.rs`
- Modify: `src/app.rs`
- Modify: `src/export.rs`
- Modify: `src/theme.rs`

- [ ] **Step 1: Update i18n.rs**
Global replace "Arch TUI" with "metapak" in all language dictionaries.

- [ ] **Step 2: Update User-Agent in services.rs**
Change `arch-tui/0.1.0` to `metapak/0.1.0`.

- [ ] **Step 3: Update notification titles in main.rs**
Change `notifier.send("Arch TUI", ...)` to `notifier.send("metapak", ...)`.

- [ ] **Step 4: Update doc comments in various files**
Update `//!` comments that mention "Arch TUI".

- [ ] **Step 5: Commit**
```bash
git add src/i18n.rs src/services.rs src/main.rs src/app.rs src/export.rs src/theme.rs
git commit -m "chore: update brand strings and i18n to metapak"
```

---

### Task 6: System Files & Scripts

**Files:**
- Rename: `arch-tui.desktop` -> `metapak.desktop`
- Modify: `metapak.desktop` (contents)
- Modify: `PKGBUILD`
- Modify: `install.sh`

- [ ] **Step 1: Rename and update desktop file**
```bash
mv arch-tui.desktop metapak.desktop
```
Update `Name=metapak` and `Exec=metapak` in `metapak.desktop`.

- [ ] **Step 2: Update PKGBUILD**
Update `pkgname`, `url`, and install paths.

- [ ] **Step 3: Update install.sh**
Update all directory references and binary names.

- [ ] **Step 4: Commit**
```bash
git add metapak.desktop PKGBUILD install.sh
git rm arch-tui.desktop
git commit -m "chore: rename desktop file and update build/install scripts"
```

---

### Task 7: Documentation & Examples

**Files:**
- Modify: `README.md`
- Modify: `GEMINI.md`
- Modify: `AGENTS.md`
- Modify: `config.example.toml`

- [ ] **Step 1: Update README.md**
Comprehensive replace of "Arch TUI" and "arch-tui". Update URLs and install instructions.

- [ ] **Step 2: Update GEMINI.md & AGENTS.md**
Update developer context and file maps.

- [ ] **Step 3: Update config.example.toml**
Update header comments.

- [ ] **Step 4: Commit**
```bash
git add README.md GEMINI.md AGENTS.md config.example.toml
git commit -m "docs: update documentation to reflect metapak rebranding"
```

---

### Task 8: Verification & Cleanup

- [ ] **Step 1: Final grep search**
Run: `grep -r "arch-tui" .` and `grep -r "Arch TUI" .`
Verify remaining occurrences (e.g., in `Cargo.lock` dependencies or changelogs, if any, are acceptable).

- [ ] **Step 2: Run all tests**
Run: `cargo test`

- [ ] **Step 3: Build release binary**
Run: `cargo build --release`
Verify binary name is `target/release/metapak`.

- [ ] **Step 4: Commit final fixes**
```bash
git commit -m "chore: final cleanup and verification for metapak rename"
```
