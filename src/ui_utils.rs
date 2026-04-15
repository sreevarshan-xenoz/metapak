use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState},
};

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

pub fn truncate_with_ellipsis(s: &str, max_width: usize) -> String {
    if s.chars().count() <= max_width {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_width - 1).collect();
        format!("{}…", truncated)
    }
}

pub fn render_scrollbar(
    frame: &mut ratatui::Frame,
    area: Rect,
    state: &mut ScrollbarState,
    color: Color,
) {
    let scrollbar = Scrollbar::default()
        .orientation(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"))
        .track_symbol(Some("│"))
        .thumb_symbol("█")
        .style(Style::default().fg(color).add_modifier(Modifier::BOLD));

    frame.render_stateful_widget(scrollbar, area, state);
}

pub fn visible_height(area: Rect, has_borders: bool, title_lines: usize) -> usize {
    let mut height = area.height as usize;

    if has_borders {
        height = height.saturating_sub(2);
    }

    if title_lines > 0 {
        height = height.saturating_sub(title_lines);
    }

    height
}
