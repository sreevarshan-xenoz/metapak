use crate::app::{App, FilterOption, InputMode};
use crate::ui_utils::centered_rect;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Gauge, List, ListItem, Paragraph},
    Frame,
};

pub fn render(app: &App, f: &mut Frame) {
    let theme = &app.theme;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search bar
            Constraint::Min(1),    // Results list
            Constraint::Length(4), // Status bar
        ])
        .split(f.size());

    // Search Bar
    render_search_bar(app, f, chunks[0], theme);

    // Results List
    render_results_list(app, f, chunks[1], theme);

    // Status Bar
    render_status_bar(app, f, chunks[2], theme);

    // Overlays (in order of priority)
    if app.show_help {
        render_help_overlay(f, f.size(), theme);
    } else if app.show_package_details {
        render_package_details(app, f, theme);
    } else if app.show_dependency_visualization {
        render_dependency_visualization(app, f, theme);
    } else if app.show_console {
        render_console(app, f, theme);
    } else if app.show_password_prompt {
        render_password_prompt(app, f, theme);
    } else if app.show_confirm_prompt {
        render_confirmation(app, f, theme);
    }
}

fn render_search_bar(app: &App, f: &mut Frame, area: Rect, theme: &crate::theme::Theme) {
    let input_style = match app.input_mode {
        InputMode::Normal => Style::default().fg(theme.foreground()),
        InputMode::Editing => Style::default().fg(theme.primary()),
    };

    let border_style = match app.input_mode {
        InputMode::Normal => Style::default().fg(theme.border()),
        InputMode::Editing => Style::default().fg(theme.primary()),
    };

    let title = if app.input_mode == InputMode::Editing && app.history_index.is_some() {
        format!(
            "🔍 {} (history {})",
            app.localizer.t("search_placeholder"),
            app.history_index.unwrap() + 1
        )
    } else {
        format!("🔍 {}", app.localizer.t("search_placeholder"))
    };

    let input = Paragraph::new(app.search_input.as_str())
        .style(input_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title(title),
        );
    f.render_widget(input, area);
}

fn render_results_list(app: &App, f: &mut Frame, area: Rect, theme: &crate::theme::Theme) {
    let page_items = app.get_paginated_results();

    let items: Vec<ListItem> = page_items
        .iter()
        .enumerate()
        .map(|(_idx, pkg)| {
            let color = if pkg.is_installed {
                theme.success()
            } else {
                match pkg.source {
                    crate::models::PackageSource::Pacman => theme.repo_color(),
                    crate::models::PackageSource::Aur => theme.aur_color(),
                }
            };

            let status_mark = if app.selected_packages.contains_key(&pkg.name) {
                "☑".to_string()
            } else if pkg.is_installed {
                format!(
                    "✓ [{}]",
                    app.localizer
                        .t("installed_label")
                        .chars()
                        .next()
                        .unwrap_or('I')
                )
            } else {
                "  ○".to_string()
            };

            let source_indicator = match pkg.source {
                crate::models::PackageSource::Pacman => "📦",
                crate::models::PackageSource::Aur => " ↑",
            };

            let line = format!(
                "{} {} {:<25} {}",
                status_mark, source_indicator, pkg.name, pkg.version
            );

            ListItem::new(line).style(Style::default().fg(color))
        })
        .collect();

    let title = if app.is_loading {
        format!(
            "{} ({})",
            app.localizer.t("packages_label"),
            app.localizer.t("loading_label")
        )
    } else {
        let filter_info = match app.current_filter {
            FilterOption::All => "".to_string(),
            FilterOption::Installed => " [Installed]".to_string(),
            FilterOption::NotInstalled => " [Not Installed]".to_string(),
            FilterOption::RepoOnly => " [Repo]".to_string(),
            FilterOption::AurOnly => " [AUR]".to_string(),
        };

        let page_info = if app.total_pages() > 1 {
            format!(" Page {}/{}", app.current_page + 1, app.total_pages())
        } else {
            "".to_string()
        };

        format!(
            "{}{}{}",
            app.localizer.t("packages_label"),
            filter_info,
            page_info
        )
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(title)
                .border_style(Style::default().fg(theme.border())),
        )
        .highlight_style(
            Style::default()
                .fg(theme.highlight_fg())
                .bg(theme.highlight_bg())
                .add_modifier(Modifier::BOLD),
        );

    let mut state = ratatui::widgets::ListState::default();
    state.select(app.selected_index);

    f.render_stateful_widget(list, area, &mut state);

    // Show filter/sort indicators
    if !app.is_loading && !page_items.is_empty() {
        let info_text = format!(
            "Filter: {:?} | Sort: {:?} | ? for help",
            app.current_filter, app.current_sort
        );
        let info = Paragraph::new(info_text)
            .style(
                Style::default()
                    .fg(theme.muted())
                    .add_modifier(Modifier::ITALIC),
            )
            .alignment(Alignment::Right);

        let info_area = Rect {
            x: area.x + area.width.saturating_sub(40),
            y: area.y + 1,
            width: 38,
            height: 1,
        };
        f.render_widget(info, info_area);
    }
}

fn render_status_bar(app: &App, f: &mut Frame, area: Rect, theme: &crate::theme::Theme) {
    if let Some(err) = &app.error_message {
        let footer = Paragraph::new(format!("{}: {}", app.localizer.t("error_occurred"), err))
            .style(Style::default().fg(theme.error()).bg(Color::Rgb(50, 0, 0)))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Thick)
                    .title(app.localizer.t("error_occurred"))
                    .title_alignment(Alignment::Center)
                    .border_style(Style::default().fg(theme.error())),
            );
        f.render_widget(footer, area);
    } else {
        let update_status = match app.available_updates {
            Some(0) => app.localizer.t("up_to_date"),
            Some(n) => format!("{}: {}", app.localizer.t("updates_available"), n),
            None => app.localizer.t("loading_label"),
        };

        let update_color = match app.available_updates {
            Some(n) if n > 0 => theme.warning(),
            Some(0) => theme.success(),
            _ => theme.muted(),
        };

        let selection_info = if !app.selected_packages.is_empty() {
            format!(" [{} selected]", app.selected_packages.len())
        } else {
            "".to_string()
        };

        let footer_lines = vec![
            Line::from(vec![
                Span::styled(
                    " ? ",
                    Style::default().fg(theme.foreground()).bg(theme.muted()),
                ),
                Span::raw(" Help "),
                Span::styled(
                    " / ",
                    Style::default().fg(theme.foreground()).bg(theme.muted()),
                ),
                Span::raw(" Search "),
                Span::styled(
                    " ↵ ",
                    Style::default().fg(theme.foreground()).bg(theme.muted()),
                ),
                Span::raw(" Install/Remove "),
                Span::styled(
                    " Tab ",
                    Style::default().fg(theme.foreground()).bg(theme.muted()),
                ),
                Span::raw(" Select "),
            ]),
            Line::from(vec![
                Span::styled(
                    " q ",
                    Style::default().fg(theme.foreground()).bg(theme.muted()),
                ),
                Span::raw(" Quit "),
                Span::styled(
                    " f ",
                    Style::default().fg(theme.foreground()).bg(theme.muted()),
                ),
                Span::raw(" Filter "),
                Span::styled(
                    " s ",
                    Style::default().fg(theme.foreground()).bg(theme.muted()),
                ),
                Span::raw(" Sort "),
                Span::styled(
                    " n/p ",
                    Style::default().fg(theme.foreground()).bg(theme.muted()),
                ),
                Span::raw(" Page "),
                Span::styled(
                    " U ",
                    Style::default().fg(theme.foreground()).bg(theme.muted()),
                ),
                Span::raw(" Update "),
                Span::styled(update_status, Style::default().fg(update_color)),
                Span::styled(
                    selection_info,
                    Style::default()
                        .fg(theme.primary())
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
        ];

        let footer = Paragraph::new(footer_lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(app.localizer.t("status"))
                .title_alignment(Alignment::Center)
                .border_style(Style::default().fg(theme.border())),
        );
        f.render_widget(footer, area);
    }
}

fn render_password_prompt(app: &App, f: &mut Frame, theme: &crate::theme::Theme) {
    let block = Block::default()
        .title(app.localizer.t("sudo_password_required"))
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .style(
            Style::default()
                .bg(theme.background())
                .fg(theme.foreground()),
        )
        .border_style(Style::default().fg(theme.warning()));

    let area = centered_rect(60, 30, f.size());

    f.render_widget(Clear, area);
    f.render_widget(block.clone(), area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Length(3)].as_ref())
        .margin(2)
        .split(area);

    let msg = Paragraph::new(app.localizer.t("enter_sudo_password"))
        .style(Style::default().fg(theme.foreground()));
    f.render_widget(msg, layout[0]);

    let masked = app.password_input.masked();
    let input = Paragraph::new(masked)
        .style(Style::default().fg(theme.warning()))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(app.localizer.t("password_label")),
        );
    f.render_widget(input, layout[1]);
}

fn render_confirmation(app: &App, f: &mut Frame, theme: &crate::theme::Theme) {
    if app.packages_pending_confirmation.is_empty() {
        return;
    }

    let pkg_count = app.packages_pending_confirmation.len();
    let is_multi = pkg_count > 1;

    let first = &app.packages_pending_confirmation[0];
    let action_str = if first.is_installed {
        app.localizer.t("remove_button")
    } else {
        app.localizer.t("install_button")
    };

    let title = if first.is_installed {
        app.localizer.t("confirm_remove")
    } else {
        app.localizer.t("confirm_install")
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .style(
            Style::default()
                .bg(theme.background())
                .fg(theme.foreground()),
        )
        .border_style(Style::default().fg(theme.primary()));

    let area = centered_rect(50, 25, f.size());

    f.render_widget(Clear, area);
    f.render_widget(block.clone(), area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length(2),
                Constraint::Length(1),
            ]
            .as_ref(),
        )
        .margin(2)
        .split(area);

    let text = if is_multi {
        format!(
            "{} {} {}?",
            app.localizer.t("confirm_multiple"),
            action_str.to_lowercase(),
            pkg_count
        )
    } else {
        format!(
            "{} {} {}?",
            app.localizer.t("confirm_single"),
            action_str.to_lowercase(),
            first.name
        )
    };

    let p_text = Paragraph::new(text)
        .style(
            Style::default()
                .fg(theme.foreground())
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .wrap(ratatui::widgets::Wrap { trim: true });
    f.render_widget(p_text, layout[0]);

    // Show packages being processed
    if is_multi {
        let pkg_list = app
            .packages_pending_confirmation
            .iter()
            .map(|p| p.name.clone())
            .collect::<Vec<_>>()
            .join(", ");
        let list_text = Paragraph::new(pkg_list)
            .style(Style::default().fg(theme.muted()))
            .alignment(Alignment::Center)
            .wrap(ratatui::widgets::Wrap { trim: true });
        f.render_widget(list_text, layout[1]);
    }

    let p_actions = Paragraph::new(app.localizer.t("confirmation_instructions"))
        .style(Style::default().fg(theme.muted()))
        .alignment(Alignment::Center);
    f.render_widget(p_actions, layout[2]);
}

fn render_console(app: &App, f: &mut Frame, theme: &crate::theme::Theme) {
    let console_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .title("Console Output")
        .style(Style::default().bg(Color::Black).fg(theme.foreground()))
        .border_style(Style::default().fg(theme.info()));

    // Show last 25 lines with progress bar if available
    let start_index = app.console_buffer.len().saturating_sub(25);
    let logs: Vec<ListItem> = app.console_buffer[start_index..]
        .iter()
        .map(|l| {
            let style = if l.contains("[stderr]") {
                Style::default().fg(theme.error())
            } else if l.contains("error") || l.contains("Error") || l.contains("failed") {
                Style::default().fg(theme.error())
            } else if l.contains("warning") || l.contains("Warning") {
                Style::default().fg(theme.warning())
            } else if l.contains("success") || l.contains("completed") || l.contains("Finished") {
                Style::default().fg(theme.success())
            } else {
                Style::default().fg(theme.foreground())
            };
            ListItem::new(l.clone()).style(style)
        })
        .collect();

    let list = List::new(logs).block(console_block);

    let chunk = centered_rect(85, 85, f.size());
    f.render_widget(Clear, chunk);
    f.render_widget(list, chunk);

    // Render progress bar if available
    if let Some(progress) = &app.command_progress {
        let progress_area = Rect {
            x: chunk.x + 2,
            y: chunk.y + chunk.height - 3,
            width: chunk.width - 4,
            height: 1,
        };

        let ratio = progress.current as f64 / progress.total as f64;
        let gauge = Gauge::default()
            .ratio(ratio.min(1.0))
            .label(format!(
                "{} {}/{}",
                progress.current_package, progress.current, progress.total
            ))
            .style(Style::default().fg(theme.primary()).bg(theme.border()));

        f.render_widget(gauge, progress_area);
    }
}

fn render_package_details(app: &App, f: &mut Frame, theme: &crate::theme::Theme) {
    if let Some(pkg) = app.get_selected_package() {
        let details_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .title(format!(
                "📦 Package Details: {} ({})",
                pkg.name, pkg.version
            ))
            .style(
                Style::default()
                    .bg(theme.background())
                    .fg(theme.foreground()),
            )
            .border_style(Style::default().fg(theme.info()));

        let area = centered_rect(80, 80, f.size());
        f.render_widget(Clear, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(10),
                Constraint::Min(1),
                Constraint::Length(12),
                Constraint::Length(3),
            ])
            .margin(2)
            .split(area);

        // Title
        let title = Paragraph::new(Line::from(vec![
            Span::styled(
                "📦 Package: ",
                Style::default()
                    .fg(theme.info())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(&pkg.name, Style::default().fg(theme.primary())),
            Span::raw(" | "),
            Span::styled(
                "Version: ",
                Style::default()
                    .fg(theme.warning())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(&pkg.version, Style::default().fg(theme.warning())),
        ]))
        .alignment(Alignment::Center);
        f.render_widget(title, chunks[0]);

        // Basic info
        let mut info_spans = vec![
            Span::styled(
                "📋 Description: ",
                Style::default()
                    .fg(theme.info())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(&pkg.description),
            Span::raw("\n\n"),
        ];

        if let Some(url) = &pkg.url {
            info_spans.push(Span::styled(
                "🔗 URL: ",
                Style::default()
                    .fg(theme.info())
                    .add_modifier(Modifier::BOLD),
            ));
            info_spans.push(Span::styled(url, Style::default().fg(theme.primary())));
            info_spans.push(Span::raw("\n"));
        }

        if !pkg.licenses.is_empty() {
            info_spans.push(Span::styled(
                "⚖️  License(s): ",
                Style::default()
                    .fg(theme.success())
                    .add_modifier(Modifier::BOLD),
            ));
            info_spans.push(Span::raw(pkg.licenses.join(", ")));
            info_spans.push(Span::raw("\n"));
        }

        if !pkg.groups.is_empty() {
            info_spans.push(Span::styled(
                "📁 Groups: ",
                Style::default()
                    .fg(theme.secondary())
                    .add_modifier(Modifier::BOLD),
            ));
            info_spans.push(Span::raw(pkg.groups.join(", ")));
            info_spans.push(Span::raw("\n"));
        }

        let info_para =
            Paragraph::new(Line::from(info_spans)).wrap(ratatui::widgets::Wrap { trim: true });
        f.render_widget(info_para, chunks[1]);

        // Description
        let desc_para = Paragraph::new(pkg.description.clone())
            .wrap(ratatui::widgets::Wrap { trim: true })
            .style(Style::default().fg(theme.foreground()));
        f.render_widget(desc_para, chunks[2]);

        // Dependencies
        let deps_title = Paragraph::new(Line::from(vec![Span::styled(
            "📦 Dependencies: ",
            Style::default()
                .fg(theme.info())
                .add_modifier(Modifier::BOLD),
        )]));
        f.render_widget(deps_title, chunks[3]);

        let deps_list_items: Vec<ListItem> = pkg
            .depends_on
            .iter()
            .map(|dep| ListItem::new(format!("  • {}", dep)))
            .collect();

        let deps_list = List::new(deps_list_items)
            .block(Block::default().borders(Borders::NONE))
            .style(Style::default().fg(theme.success()));

        f.render_widget(deps_list, chunks[3]);

        // Footer
        let footer = Paragraph::new("Press 'Esc' to return | 'v' for dependencies")
            .style(Style::default().fg(theme.muted()))
            .alignment(Alignment::Center);
        f.render_widget(footer, chunks[4]);

        f.render_widget(details_block, area);
    }
}

fn render_dependency_visualization(app: &App, f: &mut Frame, theme: &crate::theme::Theme) {
    if let Some(pkg) = app.get_selected_package() {
        let viz_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .title(format!("🌳 Dependency Tree: {}", pkg.name))
            .style(
                Style::default()
                    .bg(theme.background())
                    .fg(theme.foreground()),
            )
            .border_style(Style::default().fg(theme.secondary()));

        let area = centered_rect(85, 85, f.size());
        f.render_widget(Clear, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(1),
                Constraint::Length(3),
            ])
            .margin(2)
            .split(area);

        // Title
        let title = Paragraph::new(Line::from(vec![
            Span::styled(
                "🌳 Package: ",
                Style::default()
                    .fg(theme.info())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(&pkg.name, Style::default().fg(theme.primary())),
        ]))
        .alignment(Alignment::Center);
        f.render_widget(title, chunks[0]);

        // Generate and display dependency tree
        let tree =
            crate::dependency_visualization::DependencyVisualizationService::build_dependency_tree(
                pkg, 3,
            );
        let tree_text =
            crate::dependency_visualization::DependencyVisualizationService::format_tree(&tree, 0);

        let tree_para = Paragraph::new(tree_text)
            .wrap(ratatui::widgets::Wrap { trim: false })
            .scroll((0, 0))
            .style(Style::default().fg(theme.foreground()));
        f.render_widget(tree_para, chunks[1]);

        // Footer
        let footer = Paragraph::new("Press 'Esc' to return")
            .style(Style::default().fg(theme.muted()))
            .alignment(Alignment::Center);
        f.render_widget(footer, chunks[2]);

        f.render_widget(viz_block, area);
    }
}

fn render_help_overlay(f: &mut Frame, area: Rect, theme: &crate::theme::Theme) {
    let help_text = vec![
        Line::from(vec![Span::styled(
            "Keyboard Shortcuts",
            Style::default()
                .fg(theme.primary())
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::UNDERLINED),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Navigation",
            Style::default()
                .fg(theme.info())
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  ↑/k       Move up"),
        Line::from("  ↓/j       Move down"),
        Line::from("  n         Next page"),
        Line::from("  p         Previous page"),
        Line::from("  Home      First page"),
        Line::from("  End       Last page"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Search & Filter",
            Style::default()
                .fg(theme.info())
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  / or i    Enter search mode"),
        Line::from("  ↑/↓       Navigate search history (in search mode)"),
        Line::from("  f         Cycle filter (All → Installed → Not Installed → Repo → AUR)"),
        Line::from("  s         Cycle sort (Name ↑ → Name ↓ → Source)"),
        Line::from("  r         Clear results and selection"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Actions",
            Style::default()
                .fg(theme.info())
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  Tab       Toggle package selection"),
        Line::from("  u         Undo last selection"),
        Line::from("  Enter     Install/Remove selected packages"),
        Line::from("  y/n       Confirm/Cancel operation"),
        Line::from("  U         Update system"),
        Line::from("  d         Show package details"),
        Line::from("  v         Show dependency tree"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "General",
            Style::default()
                .fg(theme.info())
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  ?         Toggle this help"),
        Line::from("  q         Quit application"),
        Line::from("  Esc       Cancel/Go back"),
    ];

    let help_para = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Thick)
                .title("Help")
                .border_style(Style::default().fg(theme.primary())),
        )
        .style(Style::default().fg(theme.foreground()));

    let help_area = centered_rect(70, 80, area);
    f.render_widget(Clear, help_area);
    f.render_widget(help_para, help_area);
}
