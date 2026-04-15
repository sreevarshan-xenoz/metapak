# Arch TUI Future Enhancements Plan

> **For agentic workers:** Implementation plan for extending arch-tui with UX, performance, and integration features.

**Goal:** Expand arch-tui with fuzzy search, better keybindings, caching, parallel operations, external integrations.

---

## UX Enhancements

### 1. Fuzzy Search Implementation
- Add fuzzy matching algorithm (use `fuzzy-matcher` crate)
- Highlight matched characters in results
- Score-based ranking

### 2. Custom Keybindings
- Allow user-configurable keyboard shortcuts in `config.toml`
- Add keybindings for all actions

### 3. Search Filters as Tabs
- Tab-based filter UI (All | Installed | Not Installed | Repo | AUR)
- Quick filter switching with number keys 1-5

### 4. Package Preview Panel
- Quick preview on hover/selection without full details view
- Show size, dependencies count, maintainer

### 5. Package Groups Support
- Display package groups
- Filter by group
- Group-based operations

---

## Performance & Data

### 6. Search Result Caching
- Cache pacman/AUR search results
- Configurable cache TTL
- Manual cache clear

### 7. Parallel Package Operations
- Parallel downloads for multi-package installs
- Progress tracking per package

### 8. AUR Vote/Clone Count Display
- Show AUR votes and popularity
- Show last updated date

### 9. Package Size Information
- Display download size
- Installed size
- Size in results list

### 10. Out-of-date Package Indicator
- Mark outdated AUR packages
- Show update available badge

---

## Integration

### 11. External Package Viewer
- Open package page in browser (aur.archlinux.org, pkg.archlinux.org)
- Open PKGBUILD in editor
- Copy AUR clone command

### 12. System Notifications
- Desktop notifications for completed operations
- Update available notifications (when terminal not focused)

### 13. Export/Import Package List
- Export installed packages to file
- Import package list for batch install

### 14. Custom Pacman/AUR Commands
- Define custom pre/post install hooks
- Customaur helper options

---

## File Structure (Changes)

| File | Action | Responsibility |
|---|---|---|
| `src/search.rs` | **Create** | Fuzzy matching, scoring |
| `src/config.rs` | **Extend** | Keybinding config |
| `src/ui.rs` | **Extend** | Tabs, preview panel |
| `src/services.rs` | **Extend** | Caching, parallel ops |
| `src/models.rs` | **Extend** | AUR fields (votes, outdated) |
| `src/notifications.rs` | **Create** | Desktop notifications |
| `src/export.rs` | **Create** | Export/import functionality |
| `config.example.toml` | **Update** | New config options |

---

## Implementation Priority

1. Fuzzy Search (highest impact)
2. Keybinding Config (usability)
3. Search Caching (performance)
4. Package Size Info (useful)
5. AUR Metadata Display (popularity, votes)
6. External Browser Open (convenience)
7. Desktop Notifications (integration)
8. Filter Tabs (UI polish)
9. Export/Import (backup/restore)
10. Package Groups (organization)
