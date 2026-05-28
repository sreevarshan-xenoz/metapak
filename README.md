# metapak (Unified Package Manager UI)

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/built_with-Rust-orange.svg)
![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20Windows-blue.svg)

**metapak** is a modern, terminal-based user interface for managing packages. It creates a unified experience for searching, installing, and removing packages across system package managers (pacman, apt, dnf, zypper, brew, scoop, apk) and language ecosystems (npm, pip, cargo), all without ever leaving the TUI.

---

## 🚀 Key Features

### Package Management
*   **Universal Search**: Query both **System Repositories** (pacman, apt, brew, scoop, apk) and **Language Ecosystems** (npm, pip, cargo) simultaneously.
*   **Batch Operations**: Select multiple packages using `Tab` and install/remove them in a single transaction.
*   **Live Search**: Search as you type with debounced queries.
*   **Fuzzy Matching**: Find packages even with partial or typo-prone queries.
*   **Pre/Post Hooks**: Execute custom shell commands before or after package operations.

### System Views
*   **Updates View** (`U`): View and manage available system updates.
*   **System Info** (`I`): CPU, RAM, uptime, OS details, desktop environment.
*   **Diagnostics** (`h`): Check pacman status, AUR helper, disk space.
*   **Foreign Packages** (`F`): View AUR/explicitly installed packages.
*   **Package Groups** (`G`): Browse and manage package groups.
*   **Package Sizes** (`P`): See top 30 largest installed packages.
*   **Orphan Packages** (`O`): Find unrequired packages.
*   **Cache Info** (`C`): View pacman and AUR cache sizes.

### Native TUI Experience
*   **Runtime Theme Switching**: Cycle between beautiful dark and light themes (Mocha/Latte) instantly using `T` or `Shift+T`.
*   **i18n Support**: Auto-detected or configurable language localization.
*   **Startup Sudo Cache**: Enter your password once at launch; no interruptions during installation.
*   **Embedded Console**: Watch build logs and installation output directly inside the UI.
*   **Safe Conflict Resolution**: Automatic safety checks and conflict parsing before executing any system changes.
*   **Dependency Visualization** (`v`): View package dependency trees.
*   **Toasts & Animations**: Smooth UI feedback.

### Backup & Recovery
*   **System Backup**: Export explicit packages to `~/.config/metapak/backups/` for disaster recovery.
*   **Restore**: Use `pacman -S --needed < backup.txt` to restore.

### Robustness
*   **Circuit Breaker**: Automatic protection against external service outages.
*   **Graceful Shutdown**: Ctrl+C safely closes the application.
*   **Telemetry**: Built-in structured operations logging with automatic log rotation.
*   **Input Validation**: Sanitized package names and paths.
*   **Search Limits**: Configurable result limits to prevent memory exhaustion.

## 📦 Installation

You can install metapak using the included automated script.

### Prerequisites
*   **OS**: Linux (any distro), macOS, or Windows
*   **Rust**: `rustup` — the installer scripts will attempt to install it if missing
*   **Package Manager**: At least one of the supported backends:
    *   **Linux**: `pacman`, `apt`, `dnf`, `zypper`, `apk`
    *   **macOS**: `brew`
    *   **Windows**: `scoop`

### Automated Install (One-Liner)

The universal installer will automatically download the source, install Rust if missing, build the release binary, and configure your PATH.

#### Linux / macOS
Open your terminal and run:
```bash
curl -sSL https://raw.githubusercontent.com/sreevarshan-xenoz/metapak/main/install.sh | bash
```

#### Windows (PowerShell)
Open PowerShell and run:
```powershell
irm https://raw.githubusercontent.com/sreevarshan-xenoz/metapak/main/install.ps1 | iex
```

### Manual Installation
If you prefer to install manually:
1.  Clone the repository:
    ```bash
    git clone https://github.com/sreevarshan-xenoz/metapak.git
    cd metapak
    ```
2.  Build and install:
    ```bash
    cargo build --release
    # Copy target/release/metapak to your PATH
    ```

## CLI Usage

```bash
metapak [OPTIONS] [COMMAND]
```

### Flags

| Flag | Description |
|------|-------------|
| `-s`, `--search <QUERY>` | Open the TUI with a search query pre-filled |
| `-h`, `--help` | Print help information |
| `-V`, `--version` | Print version information |

### Commands

| Command | Description |
|---------|-------------|
| `search <query>` | Search for packages (headless, output to stdout) |
| `check` | Check for available updates |
| `install <packages...>` | Install one or more packages |
| `remove <packages...>` | Remove one or more packages |

---

## 🎮 Usage

Launch the application from your terminal or application launcher:

```bash
metapak
```

### Keyboard Controls

| Key | Action |
|-----|--------|
| **Search & Navigation** |
| `/` or `i` | Enter search mode |
| `Esc` | Exit search mode / cancel popup / go back |
| `Up` / `k` | Move selection up |
| `Down` / `j` | Move selection down |
| `n` / `p` | Next / previous page |
| **Selection & Actions** |
| `Tab` | Toggle package selection (for batch operations) |
| `Enter` | Install / remove selected package(s) |
| `u` | Undo last selection |
| `r` | Refresh current view |
| **Views** |
| `h` | System diagnostics |
| `I` | System information (CPU, RAM, OS) |
| `O` | Orphan packages (unrequired) |
| `P` | Top 30 largest installed packages |
| `C` | Cache information |
| `F` | Foreign/AUR packages |
| `G` | Package groups |
| `U` | Available updates |
| `d` | Package details |
| `v` | Dependency tree visualization |
| `?` | Help screen |
| **Filters & Sort** |
| `f` | Cycle filters: All → Installed → Not Installed → Repo → AUR |
| `s` | Cycle sort order: Name ↑ → Name ↓ → Source → Size |
| **Modifiers** |
| `Shift+U` | System update (upgrade all) |
| `T` / `Shift+T` | Cycle UI theme (Mocha, Latte, Dark, Light) |
| `Ctrl+C` | Graceful shutdown |
| `q` | Quit application |

### Workflow Example
1.  **Launch**: Enter your `sudo` password when prompted (cached for the session).
2.  **Search**: Type `firefox` and press `Enter`.
3.  **Batch Select**: Scroll to `firefox` press `Tab`. Scroll to `vlc` press `Tab`.
4.  **Confirm**: Press `Enter`. A popup confirms "Install 2 items?".
5.  **Watch**: The console overlay appears, showing the installation progress.
6.  **Done**: Press `Esc` to return to the search list.

## Configuration

metapak reads configuration from `~/.config/metapak/config.toml`. A full example with defaults is in `config.example.toml` at the project root.

### Theme

```toml
[theme]
preset = "mocha"   # "mocha" (dark), "latte" (light), "dark", "light", or "custom"
# Color overrides (optional):
# primary_color = "#89b4fa"
# secondary_color = "#fab387"
# accent_color = "#a6e3a1"
```

Colors can be hex strings, named colors (`blue`, `red`, `green`, etc.), or RGB objects `{ r = 137, g = 180, b = 250 }`.

### AUR Helper

```toml
aur_helper = "auto"   # "auto", "yay", "paru", or "pacman"
```

### Keyboard Bindings

All keys are fully customizable under `[keyboard]`. Default bindings:

| Key | Default |
|-----|---------|
| `quit` | `"q"` |
| `search` | `"/"` |
| `install` | `"enter"` |
| `toggle_selection` | `"tab"` |
| `next_page` | `"n"` |
| `prev_page` | `"p"` |
| `next` | `"j"` |
| `prev` | `"k"` |
| `help` | `"?"` |
| `history` | `"t"` |
| `diagnostics` | `"h"` |
| `filter` | `"f"` |
| `sort` | `"s"` |
| `undo` | `"u"` |
| `details` | `"d"` |
| `dependencies` | `"v"` |
| `sidebar` | `"\\"` |
| `refresh` | `"r"` |
| `update` | `"U"` |
| `rollback` | `"R"` |

### UI

```toml
[ui]
items_per_page = 20
search_debounce_ms = 300
max_search_history = 50
max_undo_history = 20
auto_check_updates = false
update_check_interval_minutes = 60
auto_update_on_startup = false
```

### Search

```toml
[search]
cache_ttl_seconds = 300   # How long to cache search results
```

### Robustness

```toml
[robustness]
snapshot_keep_count = 5    # Number of btrfs snapshots to keep
simulation_backend = "auto" # "auto" or "mock"
```

### Hooks

Execute custom shell commands before or after package operations:

```toml
[hooks]
pre_install = ["echo 'Starting installation...'"]
post_install = ["echo 'Installation complete!'"]
pre_remove = []
post_remove = []
pre_update = []
post_update = []
```

### Internationalization

```toml
[i18n]
language = "auto"   # "auto", "en", "es", "fr", "de", "zh", "ja"
```

When set to `"auto"`, metapak detects the system locale automatically.

### Notifications

```toml
[notifications]
enabled = true
on_install = true
on_update = true
on_error = true
```

### Telemetry

```toml
[telemetry]
enabled = true          # Operation logging
max_log_size_mb = 5     # Max log file size before rotation
max_log_files = 5       # Number of rotated log files to keep
```

---

## 🛠️ Architecture

Built with the **Rust** ecosystem for speed and safety:

*   **Ratatui**: Robust TUI rendering engine.
*   **Tokio**: Async runtime for non-blocking I/O and background search tasks.
*   **Crossterm**: Cross-platform terminal handling.
*   **Reqwest**: Async HTTP client for AUR RPC v5 queries.
*   **Dashmap**: Concurrent cache for search results.
*   **Thiserror**: Error handling with custom error types.

## 🤝 Contributing

Contributions are welcome!
1.  Fork the repository.
2.  Create a feature branch (`git checkout -b feature/amazing-feature`).
3.  Commit changes (`git commit -m 'Add amazing feature'`).
4.  Push to branch (`git push origin feature/amazing-feature`).
5.  Open a Pull Request.

## 📄 License

Distributed under the MIT License. See `LICENSE` for more information.