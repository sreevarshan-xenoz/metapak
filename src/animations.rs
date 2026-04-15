//! Animation state management for spinners, border pulses, and toast notifications.
//!
//! This module provides frame-based animations that integrate into the main event loop.

use ratatui::style::{Color, Modifier, Style};
use std::time::{Duration, Instant};

use crate::theme::Theme;

// ---------------------------------------------------------------------------
// Spinner
// ---------------------------------------------------------------------------

const SPINNER_CHARS: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧"];
const SPINNER_FRAME_COUNT: u8 = SPINNER_CHARS.len() as u8; // 8
// At 30fps each frame advances every ~33ms; we advance one spinner frame every
// 3 frames so the full cycle takes ~800ms (comfortable for terminal rendering).
const MS_PER_SPINNER_FRAME: u64 = 100; // advance spinner every 100ms

// ---------------------------------------------------------------------------
// Border pulse
// ---------------------------------------------------------------------------

const BORDER_PULSE_PERIOD_MS: u64 = 2000; // full sine cycle = 2 seconds

// ---------------------------------------------------------------------------
// Toast constants
// ---------------------------------------------------------------------------

const DEFAULT_TOAST_DURATION: Duration = Duration::from_secs(3);
const MAX_VISIBLE_TOASTS: usize = 3;
const TOAST_MAX_WIDTH: usize = 60;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Visual style category for toast notifications.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToastStyle {
    Success,
    Error,
    Info,
    Warning,
}

/// A toast notification displayed at the bottom of the TUI.
#[derive(Debug, Clone)]
pub struct Toast {
    pub message: String,
    pub style: ToastStyle,
    pub created_at: Instant,
    pub duration: Duration,
}

/// Central animation state, updated once per frame from the event loop.
#[derive(Debug)]
pub struct AnimationState {
    spinner_frame: u8,
    border_phase: f32,
    #[allow(dead_code)]
    last_tick: Instant,
}

// ---------------------------------------------------------------------------
// AnimationState
// ---------------------------------------------------------------------------

impl AnimationState {
    /// Create a new `AnimationState` with all counters at zero.
    pub fn new() -> Self {
        Self {
            spinner_frame: 0,
            border_phase: 0.0,
            last_tick: Instant::now(),
        }
    }

    /// Advance animations by `delta_ms` milliseconds.
    ///
    /// Frame skipping: if `delta_ms` > 100 we skip intermediate frames and
    /// jump directly to the correct frame so the animation stays in sync
    /// with wall-clock time.
    pub fn tick(&mut self, delta_ms: u64) {
        // --- Spinner ---
        // Total spinner frames advanced = delta_ms / MS_PER_SPINNER_FRAME
        if delta_ms > 100 {
            // Frame-skipping path: compute target frame directly.
            let frames_advanced = (delta_ms / MS_PER_SPINNER_FRAME) as u8;
            self.spinner_frame = (self.spinner_frame + frames_advanced) % SPINNER_FRAME_COUNT;
        } else {
            // Normal path: advance frame by frame (single step at a time).
            let frames_advanced = (delta_ms / MS_PER_SPINNER_FRAME) as u8;
            if frames_advanced > 0 {
                self.spinner_frame = (self.spinner_frame + frames_advanced) % SPINNER_FRAME_COUNT;
            }
        }

        // --- Border pulse ---
        // Advance phase proportionally. One full cycle = BORDER_PULSE_PERIOD_MS.
        let phase_increment =
            (delta_ms as f32 / BORDER_PULSE_PERIOD_MS as f32) * 2.0 * std::f32::consts::PI;
        self.border_phase += phase_increment;
        // Keep phase bounded to avoid floating-point drift over long sessions.
        self.border_phase = self.border_phase % (2.0 * std::f32::consts::PI);
    }

    /// Return the current spinner character (braille dots, 8-frame cycle).
    pub fn spinner_char(&self) -> &'static str {
        SPINNER_CHARS[self.spinner_frame as usize]
    }

    /// Return the border-pulse brightness in the range [0.7, 1.0].
    ///
    /// Formula: normalize sin from [-1,1] to [0,1], then scale to [0.7, 1.0]:
    /// `brightness = 0.7 + 0.3 * ((sin(phase) + 1.0) / 2.0)`
    pub fn border_pulse_brightness(&self) -> f32 {
        let normalized = (self.border_phase.sin() + 1.0) / 2.0;
        0.7 + 0.3 * normalized
    }
}

impl Default for AnimationState {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Toast
// ---------------------------------------------------------------------------

impl Toast {
    /// Create a toast with the given message, style, and the default 3s duration.
    pub fn new(message: String, style: ToastStyle) -> Self {
        Self {
            message,
            style,
            created_at: Instant::now(),
            duration: DEFAULT_TOAST_DURATION,
        }
    }

    /// Create a toast with a custom duration.
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Check whether this toast has expired (elapsed >= duration).
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() >= self.duration
    }

    /// Compute the rendered `(Color, Style)` based on toast age and theme.
    ///
    /// Fade effect timeline (for default 3s duration):
    /// - 0.0 – 0.5s : full brightness (bold)
    /// - 0.5 – 2.5s : normal
    /// - 2.5 – 3.0s : dimmed / muted
    pub fn get_render_style(&self, theme: &Theme) -> (Color, Style) {
        let elapsed = self.created_at.elapsed();
        let total = self.duration.as_secs_f32();
        let ratio = if total > 0.0 {
            elapsed.as_secs_f32() / total
        } else {
            1.0
        };

        let base_color = match self.style {
            ToastStyle::Success => theme.success(),
            ToastStyle::Error => theme.error(),
            ToastStyle::Info => theme.info(),
            ToastStyle::Warning => theme.warning(),
        };

        if ratio <= 0.0 / 3.0 {
            // unreachable guard (0s) — full brightness
            (base_color, Style::default().add_modifier(Modifier::BOLD))
        } else if elapsed.as_secs_f32() <= 0.5 {
            // First 0.5s — full brightness, bold
            (base_color, Style::default().add_modifier(Modifier::BOLD))
        } else if elapsed.as_secs_f32() <= 2.5 {
            // 0.5s – 2.5s — normal
            (base_color, Style::default())
        } else {
            // 2.5s – 3.0s — dimmed / muted
            (theme.muted(), Style::default())
        }
    }

    /// Return the maximum number of toasts that should be visible at once.
    pub const fn max_visible() -> usize {
        MAX_VISIBLE_TOASTS
    }

    /// Return the maximum display width for toast messages.
    pub const fn max_width() -> usize {
        TOAST_MAX_WIDTH
    }
}

// ---------------------------------------------------------------------------
// Toast queue helper
// ---------------------------------------------------------------------------

/// Enqueue a toast, enforcing the max-visible limit (oldest dismissed).
pub fn enqueue_toast(toasts: &mut Vec<Toast>, toast: Toast) {
    // Expired toasts are useless; skip them.
    if toast.is_expired() {
        return;
    }
    toasts.push(toast);
    while toasts.len() > MAX_VISIBLE_TOASTS {
        toasts.remove(0); // remove oldest
    }
}

/// Remove any expired toasts from the queue.
pub fn prune_expired_toasts(toasts: &mut Vec<Toast>) {
    toasts.retain(|t| !t.is_expired());
}

// ---------------------------------------------------------------------------
// Tests — TDD (tests written first, then implementation)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // Helper: create an `AnimationState` with a known `last_tick` by manually
    // constructing it (bypassing `Instant::now()`).
    fn make_state(spinner_frame: u8, border_phase: f32) -> AnimationState {
        AnimationState {
            spinner_frame,
            border_phase,
            last_tick: Instant::now(),
        }
    }

    // ---- Test 1: spinner cycles through all 8 frames ----
    #[test]
    fn test_spinner_cycles_through_all_frames() {
        let mut state = make_state(0, 0.0);
        // Verify initial state is frame 0
        assert_eq!(state.spinner_char(), SPINNER_CHARS[0]);
        // Each 100ms delta advances one spinner frame.
        for expected in 1..=SPINNER_FRAME_COUNT {
            state.tick(MS_PER_SPINNER_FRAME);
            let idx = (expected % SPINNER_FRAME_COUNT) as usize;
            assert_eq!(
                state.spinner_char(),
                SPINNER_CHARS[idx],
                "Expected spinner frame {} to show '{}'",
                idx,
                SPINNER_CHARS[idx]
            );
        }
    }

    // ---- Test 2: border pulse stays in valid range [0.7, 1.0] ----
    #[test]
    fn test_border_pulse_stays_in_valid_range() {
        let mut state = make_state(0, 0.0);
        // Tick 100 times at 30fps cadence (~33ms each).
        for _ in 0..100 {
            state.tick(33);
            let brightness = state.border_pulse_brightness();
            assert!(
                brightness >= 0.7 - 1e-5 && brightness <= 1.0 + 1e-5,
                "Brightness out of range: {} (phase={})",
                brightness,
                state.border_phase
            );
        }
    }

    // ---- Test 3: frame skipping on large deltas ----
    #[test]
    fn test_frame_skipping_on_large_deltas() {
        let mut state = make_state(0, 0.0);
        // A 200ms delta should advance spinner by 200/100 = 2 frames.
        state.tick(200);
        assert_eq!(state.spinner_frame, 2, "200ms delta should advance 2 frames");

        // A 500ms delta should advance by 5 frames → from 2 to 7.
        state.tick(500);
        assert_eq!(state.spinner_frame, 7, "500ms delta should advance 5 frames");

        // A 1000ms delta advances 10 frames → wraps: (7+10)%8 = 1.
        state.tick(1000);
        assert_eq!(state.spinner_frame, 1, "1000ms delta should wrap to frame 1");
    }

    // ---- Test 4: toast expiry after 3 seconds ----
    #[test]
    fn test_toast_expiry() {
        let toast = Toast::new("hello".to_string(), ToastStyle::Info);
        // Fresh toast should not be expired.
        assert!(!toast.is_expired());

        // Create a toast that will definitely be expired by fudging created_at.
        let expired = Toast {
            message: "old".to_string(),
            style: ToastStyle::Warning,
            created_at: Instant::now() - Duration::from_secs(4),
            duration: DEFAULT_TOAST_DURATION,
        };
        assert!(expired.is_expired());
    }

    // ---- Test 5: toast max queue enforced ----
    #[test]
    fn test_toast_max_queue() {
        let mut queue: Vec<Toast> = Vec::new();

        enqueue_toast(&mut queue, Toast::new("1".into(), ToastStyle::Info));
        assert_eq!(queue.len(), 1);

        enqueue_toast(&mut queue, Toast::new("2".into(), ToastStyle::Info));
        assert_eq!(queue.len(), 2);

        enqueue_toast(&mut queue, Toast::new("3".into(), ToastStyle::Info));
        assert_eq!(queue.len(), 3);

        enqueue_toast(&mut queue, Toast::new("4".into(), ToastStyle::Info));
        assert_eq!(
            queue.len(),
            MAX_VISIBLE_TOASTS,
            "Queue should not exceed {} toasts",
            MAX_VISIBLE_TOASTS
        );
        // Oldest should have been removed → messages should be "2","3","4".
        assert_eq!(queue[0].message, "2");
        assert_eq!(queue[2].message, "4");
    }

    // ---- Additional useful tests ----

    #[test]
    fn test_toast_render_style_phases() {
        let theme = Theme::default();

        // Fresh toast (< 0.5s) → bold
        let toast = Toast {
            message: "test".into(),
            style: ToastStyle::Success,
            created_at: Instant::now() - Duration::from_millis(200),
            duration: DEFAULT_TOAST_DURATION,
        };
        let (color, style) = toast.get_render_style(&theme);
        assert_eq!(color, theme.success());
        assert!(style.add_modifier.contains(Modifier::BOLD));

        // Mid-life toast (1.5s) → normal
        let toast = Toast {
            message: "test".into(),
            style: ToastStyle::Info,
            created_at: Instant::now() - Duration::from_millis(1500),
            duration: DEFAULT_TOAST_DURATION,
        };
        let (color, style) = toast.get_render_style(&theme);
        assert_eq!(color, theme.info());
        assert!(!style.add_modifier.contains(Modifier::BOLD));

        // Dying toast (2.8s) → muted
        let toast = Toast {
            message: "test".into(),
            style: ToastStyle::Error,
            created_at: Instant::now() - Duration::from_millis(2800),
            duration: DEFAULT_TOAST_DURATION,
        };
        let (color, _style) = toast.get_render_style(&theme);
        assert_eq!(color, theme.muted());
    }

    #[test]
    fn test_toast_constants() {
        assert_eq!(Toast::max_visible(), 3);
        assert_eq!(Toast::max_width(), 60);
    }

    #[test]
    fn test_prune_expired_toasts() {
        let mut queue: Vec<Toast> = Vec::new();
        queue.push(Toast::new("alive".into(), ToastStyle::Info));
        queue.push(Toast {
            message: "dead".into(),
            style: ToastStyle::Warning,
            created_at: Instant::now() - Duration::from_secs(5),
            duration: DEFAULT_TOAST_DURATION,
        });
        queue.push(Toast::new("alive2".into(), ToastStyle::Success));

        prune_expired_toasts(&mut queue);
        assert_eq!(queue.len(), 2);
        assert_eq!(queue[0].message, "alive");
        assert_eq!(queue[1].message, "alive2");
    }

    #[test]
    fn test_enqueue_expired_toast_ignored() {
        let mut queue: Vec<Toast> = Vec::new();
        let expired = Toast {
            message: "too late".into(),
            style: ToastStyle::Error,
            created_at: Instant::now() - Duration::from_secs(10),
            duration: DEFAULT_TOAST_DURATION,
        };
        enqueue_toast(&mut queue, expired);
        assert_eq!(queue.len(), 0, "Expired toast should not be enqueued");
    }

    #[test]
    fn test_default_animation_state() {
        let state = AnimationState::default();
        assert_eq!(state.spinner_frame, 0);
        assert_eq!(state.border_phase, 0.0);
    }

    #[test]
    fn test_toast_with_custom_duration() {
        let toast = Toast::new("hi".into(), ToastStyle::Info)
            .with_duration(Duration::from_secs(10));
        assert_eq!(toast.duration, Duration::from_secs(10));
    }
}
