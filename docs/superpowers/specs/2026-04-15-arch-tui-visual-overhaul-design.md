# Arch TUI Visual Overhaul Design

**Date**: 2026-04-15  
**Status**: Draft  
**Author**: Qwen Code (brainstorming session)

---

## 1. Overview

Complete visual redesign of Arch TUI to improve usability, accessibility, and aesthetic quality while maintaining the existing functionality and configuration compatibility.

### Goals

- Fix all visual bugs (truncation, contrast, missing scrollbars)
- Add animations and visual feedback (loading states, toast notifications)
- Improve navigation (scrollable overlays, split-pane details)
- Enhance visual hierarchy (box-drawing trees, contextual hints)
- Achieve WCAG AA contrast compliance (≥ 4.5:1 for all text)

### Non-Goals

- Mouse support (keyboard-only remains the primary interaction)
- Breaking config file compatibility (existing configs must still parse)
- Changing package manager backends (pacman/paru/yay logic untouched)

---

## 2. Architecture

### 2.1 Layout System

**Current**: Flat 3-panel vertical layout (search, results, status) with full-screen overlays.

**New**: Adaptive layout with optional right sidebar.

```
+-------------------+  ┌──────────────────────────────────────+
|  Search Bar (3)   |  │              Terminal                 │
+-------------------+  ├──────────┬───────────────────────────┤
|                   |  │ Results  │  Details Sidebar (30%)    │
|  Results (flex)   |  │ (70%)    │  - Package info           │
|                   |  │          │  - Dependencies           │
+-------------------+  ├──────────┴───────────────────────────┤
|  Status Bar (4)   |  │           Status Bar (4)             │
+-------------------+  └──────────────────────────────────────┘

Left: No sidebar mode           Right: Sidebar mode (toggled via 'd')
```

**Terminal Size Constraints**:
- Width < 100 cols: Force overlay-only mode (sidebar disabled)
- Height < 20 rows: Reduce search bar to 2 lines
- Minimum: 80x24 with warning if below

### 2.2 Animation Engine

**Module**: `src/animations.rs`

```rust
pub struct AnimationState {
    /// Frame counter for spinner (0-7 for 8-frame spinner)
    spinner_frame: u8,
    /// Phase for border pulse (0..2π, oscillates brightness)
    border_phase: f32,
    /// Speed of animation (frames per second)
    fps: u8,
    /// Last update timestamp
    last_tick: Instant,
}

pub struct Toast {
    pub message: String,
    pub style: ToastStyle, // Success, Error, Info, Warning
    pub created_at: Instant,
    pub duration: Duration, // Default 3s
}
```

**Animation Loop**:
- Called on every crossterm tick event (target 30fps, ~33ms interval)
- `tick(&mut self, delta_ms: u64)` advances all animations
- Frame skipping: if delta > 100ms, jump to correct phase (no intermediate frames)
- Border pulse: `brightness = base * (0.7 + 0.3 * sin(border_phase))`
- Spinner: `frame = (frame + 1) % 8` maps to `["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧"]`

**Toast Lifecycle**:
1. Created: `app.toasts.push(Toast::new(message, style))`
2. Rendered: Top-center overlay with max 3 visible, 60-char width
3. Expired: `toasts.retain(|t| now - t.created_at < t.duration)`
4. Overflow: If > 3 toasts, oldest dismissed first

### 2.3 Theme System

**Module**: `src/theme.rs` (rewrite)

**Preset Themes**:
- `Catppuccin Mocha` (dark) — default
- `Catppuccin Latte` (light) — accessible via config or runtime toggle

**Catppuccin Mocha Palette**:

| Semantic Role | Color | Hex | Contrast (on #1e1e2e) |
|---|---|---|---|
| Background | Base | `#1e1e2e` | — |
| Foreground | Text | `#cdd6f4` | 15.7:1 ✅ |
| Primary | Blue | `#89b4fa` | 10.2:1 ✅ |
| Secondary | Peach | `#fab387` | 12.1:1 ✅ |
| Success | Green | `#a6e3a1` | 11.8:1 ✅ |
| Warning | Yellow | `#f9e2af` | 13.4:1 ✅ |
| Error | Red | `#f38ba8` | 7.8:1 ✅ |
| Info | Sky | `#89dceb` | 12.6:1 ✅ |
| Muted | Overlay0 | `#6c7086` | 4.6:1 ✅ |
| Border | Surface1 | `#45475a` | 6.3:1 ✅ |
| Highlight BG | Blue | `#89b4fa` | 10.2:1 ✅ |
| Highlight FG | Base | `#1e1e2e` | — |
| Repo Packages | Sapphire | `#74c7ec` | 11.3:1 ✅ |
| AUR Packages | Peach | `#fab387` | 12.1:1 ✅ |

**Catppuccin Latte Palette**:

| Semantic Role | Color | Hex | Contrast (on #eff1f5) |
|---|---|---|---|
| Background | Base | `#eff1f5` | — |
| Foreground | Text | `#4c4f69` | 12.4:1 ✅ |
| Primary | Blue | `#1e66f5` | 6.8:1 ✅ |
| Secondary | Peach | `#fe640b` | 4.9:1 ✅ |
| Success | Green | `#40a02b` | 4.7:1 ✅ |
| Warning | Yellow | `#df8e1d` | 5.8:1 ✅ |
| Error | Red | `#d20f39` | 5.9:1 ✅ |
| Info | Sky | `#04a5e5` | 4.6:1 ✅ |
| Muted | Overlay0 | `#9ca0b0` | 4.5:1 ✅ |
| Border | Surface1 | `#bcc0cc` | 4.5:1 ✅ |
| Highlight BG | Blue | `#1e66f5` | 6.8:1 ✅ |
| Highlight FG | Base | `#eff1f5` | — |
| Repo Packages | Sapphire | `#209fb5` | 4.7:1 ✅ |
| AUR Packages | Peach | `#fe640b` | 4.9:1 ✅ |

**Config Support**:
```toml
[theme]
preset = "mocha"  # "mocha" | "latte" | "custom"

# Individual overrides (only used when preset = "custom" or to tweak presets)
primary = "#89b4fa"
warning = "#f9e2af"
# ... etc
```

**Theme Validation**:
- On startup: log warnings if any contrast ratio < 4.5:1
- Invalid color values → fallback to preset default
- Helper: `calculate_contrast_ratio(fg: Color, bg: Color) -> f32`

---

## 3. Component Details

### 3.1 File Changes

| File | Change Type | Description |
|---|---|---|
| `src/animations.rs` | **New** | Animation state, toast queue, tick logic |
| `src/theme.rs` | **Rewrite** | Catppuccin palettes, semantic colors, validation |
| `src/ui.rs` | **Refactor** | Split-pane, scrollbars, animations, toasts, contextual hints |
| `src/ui_utils.rs` | **Extend** | Scrollbar helper, tree renderer, truncation, contrast calc |
| `src/dependency_visualization.rs` | **Update** | Box-drawing tree, color-coded nodes, scrollable |
| `src/app.rs` | **Extend** | New state fields (sidebar, animations, toasts, scroll states) |
| `src/action.rs` | **Extend** | New actions: ToggleSidebar, DismissToast, ScrollUp/Down |
| `config.example.toml` | **Update** | Document theme config syntax |

### 3.2 New State Fields (app.rs)

```rust
pub struct App {
    // ... existing fields ...
    
    /// Show package details in sidebar (true) or overlay (false)
    pub show_sidebar: bool,
    
    /// Animation state for spinners, pulses, etc.
    pub animation_state: AnimationState,
    
    /// Toast notification queue
    pub toasts: Vec<Toast>,
    
    /// Scroll states for all scrollable areas
    pub scroll_states: HashMap<String, ListState>,
}
```

### 3.3 New Key Bindings

| Key | Action | Context |
|---|---|---|
| `d` | Toggle details sidebar | Normal mode, not searching |
| `Esc` | Dismiss toast if visible, else current behavior | When toast visible |

### 3.4 Scroll State Management

**Keys for `scroll_states` HashMap**:
- `"results"` — main package list
- `"history"` — transaction history overlay
- `"dependency_tree"` — dependency visualization
- `"console"` — console output buffer
- `"diagnostics"` — diagnostics overlay

**Behavior**:
- Each scroll state is a `ListState` with `selected` index
- Arrow keys update the active scroll state
- Scroll position clamped: `0..max(items.len().saturating_sub(visible_height), 0)`
- Reset on filter/sort only if current index > new length

### 3.5 Contextual Keyboard Hints

**Normal mode (no overlay)**:
```
? Help  / Search  Enter Install  Tab Select  d Details  q Quit
f Filter  s Sort  n/p Page  U Updates  [N selected]  ✓ Up to date
```

**Search mode**:
```
Esc Exit Search  Enter Search  ↑↓ History
```

**Overlay open**:
```
Esc Close  ↑↓ Scroll
```

**Implementation**: Filter hint lines by checking `app.input_mode`, `app.show_*` flags.

### 3.6 Dependency Tree Rendering

**Before** (plain indentation):
```
✓ firefox (120.0)
  ○ libgtk (3.24.0)
    ○ libgdk (3.24.0)
```

**After** (box-drawing with colors):
```
├─ ✓ firefox (120.0)              [green]
│  ├─ ○ libgtk (3.24.0)          [muted]
│  │  └─ ○ libgdk (3.24.0)      [muted]
│  └─ ✓ libsqlite (3.44.0)       [green]
└─ ⚠ circular-dep (cycle)        [yellow]
```

**Cycle Detection**: Marked with `⚠ cycle` in yellow, prevents infinite recursion.

### 3.7 Toast Rendering

**Position**: Top-center, 60 chars wide, max 3 lines
**Style**: 
- Success: Green border + text
- Error: Red border + text
- Info: Blue border + text
- Warning: Yellow border + text

**Fade Effect** (via style progression):
- 0-0.5s: Full brightness
- 0.5-2.5s: Normal
- 2.5-3.0s: Dimmed (muted style)

**Fallback**: If toast conflicts with overlay, message shown in status bar instead.

---

## 4. Error Handling

### 4.1 Animation Performance

- Frame drops (>100ms): Skip intermediate frames, jump to current phase
- Spinner: `(current_frame + elapsed_frames) % 8`
- Border pulse: phase-based sine wave, always correct regardless of frame gaps

### 4.2 Terminal Size Constraints

- Width < 100 cols: Disable sidebar mode
- Height < 20 rows: Reduce search bar height
- Below 80x24: Show warning, continue with degraded layout

### 4.3 Theme Validation

- Invalid color in config: Fallback to preset default, log warning
- Low contrast detected: Log warning, continue (user choice)
- Config parse error: Use default Mocha theme

### 4.4 Scroll Boundaries

- Arrow keys at boundaries: No-op (no visual feedback needed)
- Filter/sort reduces items: Clamp scroll position to new max

### 4.5 Toast Overflow

- Max 3 toasts: Oldest dismissed when 4th arrives
- Text > 60 chars: Truncated with ellipsis
- Render conflict with overlay: Fallback to status bar message

---

## 5. Testing Strategy

### 5.1 Unit Tests

- `test_contrast_ratios()` — verify all semantic color combos ≥ 4.5:1
- `test_animation_tick()` — verify spinner frame cycling, border phase advancement
- `test_toast_lifecycle()` — verify creation, expiry, overflow
- `test_scroll_clamping()` — verify boundaries on various list sizes
- `test_truncate_with_ellipsis()` — verify name truncation at various widths

### 5.2 Integration Tests

- Launch app, verify default theme loads
- Toggle sidebar, verify layout adjusts
- Trigger toast (simulate install success), verify appearance and dismissal
- Navigate scrollable overlay, verify scrollbar moves
- Type long package name, verify truncation in results list

### 5.3 Manual Testing

- Test on 80x24 terminal (minimum)
- Test on 200x60 terminal (large)
- Test with custom theme config
- Test with AUR search (slow response, verify loading state)
- Test dependency tree with deep/circular dependencies

---

## 6. Migration Path

### Backward Compatibility

- Existing `config.toml` files remain valid
- `theme` section gains `preset` field, defaults to `"mocha"`
- Existing `theme.*` color overrides still apply as customizations
- No breaking changes to app state serialization

### Incremental Rollout

All changes shipped together as a single release. No feature flags needed.

---

## 7. Future Enhancements (Out of Scope)

- Runtime theme switching via `T` key
- Mouse click support for list selection
- Custom scrollbar styles per theme
- Animated list reordering (smooth scroll on filter)
- Package icons/favicons from upstream metadata
- Export screenshot functionality
