use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
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
    if max_width == 0 {
        return String::new();
    }

    let char_count = s.chars().count();
    if char_count <= max_width {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_width.saturating_sub(1)).collect();
        format!("{}…", truncated)
    }
}

pub fn create_highlighted_line(
    text: &str,
    indices: &[usize],
    base_style: Style,
    highlight_style: Style,
) -> Line<'static> {
    if indices.is_empty() {
        return Line::from(Span::styled(text.to_string(), base_style));
    }

    let mut spans = Vec::new();
    let char_indices: Vec<usize> = text.char_indices().map(|(b, _)| b).collect();
    let total_chars = char_indices.len();

    if total_chars == 0 {
        return Line::from(Vec::new());
    }

    let mut start_char_idx = 0;
    let mut is_highlighted = indices.contains(&0);

    for i in 1..total_chars {
        let h = indices.contains(&i);
        if h != is_highlighted {
            let start_byte = char_indices[start_char_idx];
            let end_byte = char_indices[i];
            spans.push(Span::styled(
                text[start_byte..end_byte].to_string(),
                if is_highlighted {
                    highlight_style
                } else {
                    base_style
                },
            ));
            start_char_idx = i;
            is_highlighted = h;
        }
    }

    let start_byte = char_indices[start_char_idx];
    spans.push(Span::styled(
        text[start_byte..].to_string(),
        if is_highlighted {
            highlight_style
        } else {
            base_style
        },
    ));

    Line::from(spans)
}

pub fn render_scrollbar(frame: &mut Frame, area: Rect, state: &mut ScrollbarState, color: Color) {
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
