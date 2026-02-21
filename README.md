# Arch TUI (Arch Linux Package Manager UI)

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/built_with-Rust-orange.svg)
![Platform](https://img.shields.io/badge/platform-Arch_Linux-1793d1.svg)

**Arch TUI** is a modern, terminal-based user interface for managing packages on Arch Linux. It creates a unified experience for searching, installing, and removing packages from both the official repositories (`pacman`) and the AUR (via `paru` or `yay`), all without ever leaving the TUI.

---

## 🚀 Key Features

*   **Unified Search**: Query both **Official Repositories** and the **AUR** simultaneously.
*   **Batch Operations**: Select multiple packages using `Tab` and install/remove them in a single transaction.
*   **Native TUI Experience**:
    *   **Startup Sudo Cache**: Enter your password once at launch; no interruptions during installation.
    *   **Embedded Console**: Watch build logs and installation output directly inside the UI.
    *   **Confirmation Popups**: Safety checks before executing any system changes.
*   **Performance**: Asynchronous searching ensures the UI never freezes, even when querying slow AUR endpoints.
*   **Visual Clarity**: Color-coded results (Blue for Repo, Yellow for AUR, Green for Installed) with clear status indicators.

## 📦 Installation

You can install Arch TUI using the included automated script.

### Prerequisites
*   Arch Linux (or derivative)
*   `rustup` (Rust toolchain) - *Script will attempt to install if missing*
*   `pacman`
*   An AUR helper (`paru` or `yay`) is recommended for full functionality.

### Automated Install
1.  Clone the repository:
    ```bash
    git clone https://github.com/sreevarshan-xenoz/arch-tui.git
    cd arch-tui
    ```

2.  Run the installer:
    ```bash
    chmod +x install.sh
    ./install.sh
    ```

This will:
*   Build the release binary.
*   Install it to `~/.local/bin/arch-tui`.
*   Create a Desktop Entry so it appears in your Application Menu.

## 🎮 Usage

Launch the application from your terminal or application launcher:

```bash
arch-tui
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
| **Actions** | |
| `Tab` | **Toggle Selection** (for batch operations) |
| `Enter` | **Install/Remove** selected package(s) |
| `q` | Quit application |

### Workflow Example
1.  **Launch**: Enter your `sudo` password when prompted (cached for the session).
2.  **Search**: Type `firefox` and press `Enter`.
3.  **Batch Select**: Scroll to `firefox` press `Tab`. Scroll to `vlc` press `Tab`.
4.  **Confirm**: Press `Enter`. A popup confirms "Install 2 items?".
5.  **Watch**: The console overlay appears, showing the installation progress.
6.  **Done**: Press `Esc` to return to the search list.

## 🛠️ Architecture

Built with the **Rust** ecosystem for speed and safety:

*   **Ratatui**: Robust TUI rendering engine.
*   **Tokio**: Async runtime for non-blocking I/O and background search tasks.
*   **Crossterm**: Cross-platform terminal handling.
*   **Reqwest**: Async HTTP client for AUR RPC v5 queries.

## 🤝 Contributing

Contributions are welcome!
1.  Fork the repository.
2.  Create a feature branch (`git checkout -b feature/amazing-feature`).
3.  Commit changes (`git commit -m 'Add amazing feature'`).
4.  Push to branch (`git push origin feature/amazing-feature`).
5.  Open a Pull Request.

## 📄 License

Distributed under the MIT License. See `LICENSE` for more information.
