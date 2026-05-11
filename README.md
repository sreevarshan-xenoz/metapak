# metapak (Unified Package Manager UI)

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/built_with-Rust-orange.svg)
![Platform](https://img.shields.io/badge/platform-Arch_Linux-1793d1.svg)

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
*   A supported OS (Linux, macOS, Windows)
*   `rustup` (Rust toolchain) - *Script will attempt to install if missing*
*   One or more supported package managers (pacman, apt, brew, scoop, dnf, etc.)

### Automated Install
1.  Clone the repository:
    ```bash
    git clone https://github.com/sreevarshan-xenoz/metapak.git
    cd metapak
    ```

2.  Run the installer:
    ```bash
    chmod +x install.sh
    ./install.sh
    ```

This will:
*   Build the release binary.
*   Install it to `~/.local/bin/metapak`.
*   Create a Desktop Entry so it appears in your Application Menu.

## 🎮 Usage

Launch the application from your terminal or application launcher:

```bash
metapak
```

### Controls

| Key | Action |
| --- | --- |
| **Input** | |
| `/` or `i` | Enter **Search Mode** (type generic query) |
| `Esc` | Exit Search Mode / Cancel Popup / Quit |
| **Navigation** | |
| `Up` / `k` | Move selection up |
| `Down` / `j` | Move selection down |
| `n` / `p` | Next / Previous page |
| **Actions** | |
| `Tab` | **Toggle Selection** (for batch operations) |
| `Enter` | **Install/Remove** selected package(s) |
| **Views** | |
| `h` | System Diagnostics |
| `I` | System Information |
| `O` | Orphan Packages |
| `P` | Package Sizes (top 30) |
| `C` | Cache Information |
| `F` | Foreign Packages (AUR) |
| `G` | Package Groups |
| `U` | Updates View |
| `d` | Package Details |
| `v` | Dependency Tree |
| `?` | Help |
| `q` | Quit application |

### Workflow Example
1.  **Launch**: Enter your `sudo` password when prompted (cached for the session).
2.  **Search**: Type `firefox` and press `Enter`.
3.  **Batch Select**: Scroll to `firefox` press `Tab`. Scroll to `vlc` press `Tab`.
4.  **Confirm**: Press `Enter`. A popup confirms "Install 2 items?".
5.  **Watch**: The console overlay appears, showing the installation progress.
6.  **Done**: Press `Esc` to return to the search list.

### Filter & Sort
*   `f` - Cycle filters (All → Installed → Not Installed → Repo → AUR)
*   `s` - Cycle sort (Name ↑ → Name ↓ → Source → Size)

### Keyboard Modifiers
*   `Shift+U` - System update
*   `T` or `Shift+T` - Cycle UI Theme
*   `Ctrl+C` - Graceful shutdown

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