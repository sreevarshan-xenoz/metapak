use ratatui::{
    layout::{Constraint, Direction, Layout, Alignment, Flex},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph, Tabs, Widget},
    Frame,
};
use crate::app::App;
use crate::ui_utils::centered_rect;

pub fn render(app: &App, f: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search bar
            Constraint::Min(1),    // Results list
            Constraint::Length(3), // Footer / Status
        ])
        .split(f.size());

    // Search Bar
    let input_style = match app.input_mode {
        crate::app::InputMode::Normal => Style::default().fg(Color::Gray),
        crate::app::InputMode::Editing => Style::default().fg(Color::Cyan),
    };

    let border_style = match app.input_mode {
        crate::app::InputMode::Normal => Style::default().fg(Color::DarkGray),
        crate::app::InputMode::Editing => Style::default().fg(Color::Cyan),
    };

    let input = Paragraph::new(app.search_input.as_str())
        .style(input_style)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(format!("🔍 {}", app.localizer.t("search_placeholder"))));
    f.render_widget(input, chunks[0]);

    // Results List
    let items: Vec<ListItem> = app
        .results
        .iter()
        .map(|pkg| {
            let color = if pkg.is_installed {
                Color::Green
            } else {
                match pkg.source {
                    crate::models::PackageSource::Pacman => Color::Blue,
                    crate::models::PackageSource::Aur => Color::Rgb(255, 165, 0), // Orange for AUR
                }
            };

            let status_mark = if pkg.is_installed {
                format!("✓ [{}]", app.localizer.t("installed_label").chars().next().unwrap_or('I'))
            } else { "  ○".to_string() }; // Circle for not installed

            let source_indicator = match pkg.source {
                crate::models::PackageSource::Pacman => "📦",
                crate::models::PackageSource::Aur => " ↑", // Up arrow for AUR
            };

            let line = format!("{} {} {:<20} {}", status_mark, source_indicator, pkg.name, pkg.version);
            ListItem::new(line).style(Style::default().fg(color))
        })
        .collect();

    let title = if app.is_loading {
        format!("{} ({})", app.localizer.t("packages_label"), app.localizer.t("loading_label"))
    } else {
        app.localizer.t("packages_label")
    };
    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(title))
        .highlight_style(Style::default()
            .fg(Color::Black)
            .bg(Color::Rgb(100, 150, 255)) // Light blue highlight
            .add_modifier(ratatui::style::Modifier::BOLD));
    
    let mut state = ratatui::widgets::ListState::default();
    state.select(app.selected_index);
        
    f.render_stateful_widget(list, chunks[1], &mut state);

    // Package Details Overlay
    if app.show_package_details {
        if let Some(pkg) = app.get_selected_package() {
            let details_block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Thick)
                .title(format!("📦 Package Details: {} ({})", pkg.name, pkg.version))
                .style(Style::default().bg(Color::Rgb(30, 30, 40)));

            // Create a layout for the details
            let area = centered_rect(80, 80, f.size());
            f.render_widget(ratatui::widgets::Clear, area); // Clear background

            // Split the area into sections
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),  // Title
                    Constraint::Length(10), // Basic info
                    Constraint::Min(1),     // Description
                    Constraint::Length(12), // Dependencies
                    Constraint::Length(3),  // Footer
                ])
                .split(area);

            // Title
            let title = Paragraph::new(Line::from(vec![
                Span::styled("📦 Package: ", Style::default().fg(Color::Cyan).add_modifier(ratatui::style::Modifier::BOLD)),
                Span::styled(&pkg.name, Style::default().fg(Color::LightCyan)),
                Span::raw(" | "),
                Span::styled("Version: ", Style::default().fg(Color::Yellow).add_modifier(ratatui::style::Modifier::BOLD)),
                Span::styled(&pkg.version, Style::default().fg(Color::Yellow)),
            ])).alignment(Alignment::Center);
            f.render_widget(title, chunks[0]);

            // Basic info
            let mut info_spans = vec![
                Span::styled("📋 Description: ", Style::default().fg(Color::Rgb(173, 216, 230)).add_modifier(ratatui::style::Modifier::BOLD)),
                Span::raw(&pkg.description),
                Span::raw("\n\n"),
            ];

            if let Some(url) = &pkg.url {
                info_spans.push(Span::styled("🔗 URL: ", Style::default().fg(Color::Blue).add_modifier(ratatui::style::Modifier::BOLD)));
                info_spans.push(Span::styled(url, Style::default().fg(Color::Rgb(100, 150, 255))));
                info_spans.push(Span::raw("\n"));
            }

            if !pkg.licenses.is_empty() {
                info_spans.push(Span::styled("⚖️  License(s): ", Style::default().fg(Color::Rgb(144, 238, 144)).add_modifier(ratatui::style::Modifier::BOLD)));
                info_spans.push(Span::raw(pkg.licenses.join(", ")));
                info_spans.push(Span::raw("\n"));
            }

            if !pkg.groups.is_empty() {
                info_spans.push(Span::styled("📁 Groups: ", Style::default().fg(Color::Rgb(255, 182, 193)).add_modifier(ratatui::style::Modifier::BOLD)));
                info_spans.push(Span::raw(pkg.groups.join(", ")));
                info_spans.push(Span::raw("\n"));
            }

            let info_para = Paragraph::new(Line::from(info_spans))
                .wrap(ratatui::widgets::Wrap { trim: true });
            f.render_widget(info_para, chunks[1]);

            // Description
            let desc_para = Paragraph::new(pkg.description.clone())
                .wrap(ratatui::widgets::Wrap { trim: true })
                .style(Style::default().fg(Color::Rgb(200, 200, 200)));
            f.render_widget(desc_para, chunks[2]);

            // Dependencies
            let deps_title = Paragraph::new(Line::from(vec![
                Span::styled("📦 Dependencies: ", Style::default().fg(Color::Rgb(135, 206, 250)).add_modifier(ratatui::style::Modifier::BOLD)),
            ]));
            f.render_widget(deps_title, chunks[3]);

            let deps_list_items: Vec<ListItem> = pkg.depends_on
                .iter()
                .map(|dep| ListItem::new(format!("  • {}", dep)))
                .collect();

            let deps_list = List::new(deps_list_items)
                .block(Block::default().borders(Borders::NONE))
                .style(Style::default().fg(Color::Rgb(144, 238, 144)));

            f.render_widget(deps_list, chunks[3]);

            // Footer
            let footer = Paragraph::new("Press 'Esc' to return")
                .style(Style::default().fg(Color::Rgb(200, 200, 200)))
                .alignment(Alignment::Center);
            f.render_widget(footer, chunks[4]);

            f.render_widget(details_block, area);
            return; // Don't render other popups
        }
    }

    // Dependency Visualization Overlay
    if app.show_dependency_visualization {
        if let Some(pkg) = app.get_selected_package() {
            let viz_block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Thick)
                .title(format!("🌳 Dependency Tree: {}", pkg.name))
                .style(Style::default().bg(Color::Rgb(25, 25, 35)));

            // Create a layout for the visualization
            let area = centered_rect(85, 85, f.size());
            f.render_widget(ratatui::widgets::Clear, area); // Clear background

            // Split the area into sections
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),  // Title
                    Constraint::Min(1),     // Tree visualization
                    Constraint::Length(3),  // Footer
                ])
                .split(area);

            // Title
            let title = Paragraph::new(Line::from(vec![
                Span::styled("🌳 Package: ", Style::default().fg(Color::Cyan).add_modifier(ratatui::style::Modifier::BOLD)),
                Span::styled(&pkg.name, Style::default().fg(Color::LightCyan)),
            ])).alignment(Alignment::Center);
            f.render_widget(title, chunks[0]);

            // Generate and display dependency tree
            let tree = crate::dependency_visualization::DependencyVisualizationService::build_dependency_tree(pkg, 3);
            let tree_text = crate::dependency_visualization::DependencyVisualizationService::format_tree(&tree, 0);

            let tree_para = Paragraph::new(tree_text)
                .wrap(ratatui::widgets::Wrap { trim: false })
                .scroll((0, 0))
                .style(Style::default().fg(Color::Rgb(200, 200, 220)));
            f.render_widget(tree_para, chunks[1]);

            // Footer
            let footer = Paragraph::new("Press 'Esc' to return")
                .style(Style::default().fg(Color::Rgb(200, 200, 200)))
                .alignment(Alignment::Center);
            f.render_widget(footer, chunks[2]);

            f.render_widget(viz_block, area);
            return; // Don't render other popups
        }
    }

    // Console Overlay
    if app.show_console {
        let console_block = Block::default()
            .borders(Borders::ALL)
            .title(app.localizer.t("console_output_title"))
            .style(Style::default().bg(Color::Black));

        // We only show the last 20 lines to keep it simple, or full scroll
        let start_index = if app.console_buffer.len() > 20 {
            app.console_buffer.len() - 20
        } else {
            0
        };

        let logs: Vec<ListItem> = app.console_buffer[start_index..]
            .iter()
            .map(|l| ListItem::new(l.clone()))
            .collect();

        let list = List::new(logs).block(console_block);

        // Center the console
        let chunk = centered_rect(80, 80, f.size());
        f.render_widget(ratatui::widgets::Clear, chunk); // Clear background
        f.render_widget(list, chunk);
        return; // Don't render other popups
    }

    // Password Popup
    if app.show_password_prompt {
        let block = Block::default()
            .title(app.localizer.t("sudo_password_required"))
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Black));
        let area = centered_rect(60, 30, f.size());

        f.render_widget(ratatui::widgets::Clear, area);
        f.render_widget(block, area);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(2), Constraint::Length(3)].as_ref())
            .margin(2)
            .split(area);

        let msg = Paragraph::new(app.localizer.t("enter_sudo_password"))
            .style(Style::default().fg(Color::White));
        f.render_widget(msg, layout[0]);

        let width = app.password_input.len();
        let masked: String = format!("{}█", "*".repeat(width)); // Cursor indicator
        let input = Paragraph::new(masked)
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL).title(app.localizer.t("password_label")));
        f.render_widget(input, layout[1]);
        return;
    }
    
    // Confirmation Popup
    if app.show_confirm_prompt {
        if !app.packages_pending_confirmation.is_empty() {
            let pkg_count = app.packages_pending_confirmation.len();
            let is_multi = pkg_count > 1;

            // Check if we are removing or installing (assuming all in batch are same action for simplicity,
            // or we just say "Process X packages")
            // For now, let's just check the first one to determine "Install" vs "Remove" label,
            // or effectively "Apply changes"
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

            let block = Block::default().title(title).borders(Borders::ALL).style(Style::default().bg(Color::Blue));
            let area = centered_rect(50, 20, f.size());

            f.render_widget(ratatui::widgets::Clear, area); // Clear background
            f.render_widget(block, area);

            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(2), Constraint::Length(1)].as_ref())
                .margin(2)
                .split(area);

            let text = if is_multi {
                format!("{} {} {}?",
                    app.localizer.t("confirm_multiple"),
                    action_str.to_lowercase(),
                    pkg_count)
            } else {
                format!("{} {} {}?",
                    app.localizer.t("confirm_single"),
                    action_str.to_lowercase(),
                    first.name)
            };

            let p_text = Paragraph::new(text).style(Style::default().fg(Color::White).add_modifier(ratatui::style::Modifier::BOLD));
            f.render_widget(p_text, layout[0]);

            let p_actions = Paragraph::new(app.localizer.t("confirmation_instructions")).style(Style::default().fg(Color::Gray));
            f.render_widget(p_actions, layout[1]);
        }
        return;
    }

    // Footer
    if let Some(err) = &app.error_message {
        let footer = Paragraph::new(format!("{}: {}", app.localizer.t("error_occurred"), err))
            .style(Style::default().fg(Color::Red).bg(Color::Rgb(50, 0, 0)))
            .block(Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Thick)
                .title(app.localizer.t("error_occurred"))
                .title_alignment(Alignment::Center));
        f.render_widget(footer, chunks[2]);
    } else {
        let update_status = match app.available_updates {
            Some(0) => app.localizer.t("up_to_date"),
            Some(n) => format!("{}: {}", app.localizer.t("updates_available"), n),
            None => app.localizer.t("loading_label"),
        };

        let update_color = match app.available_updates {
            Some(n) if n > 0 => Color::Rgb(255, 165, 0), // Orange for updates available
            Some(0) => Color::Green,
            _ => Color::Gray,
        };

        let footer_line = Line::from(vec![
            Span::styled(" Esc ", Style::default().fg(Color::White).bg(Color::DarkGray)),
            Span::raw(format!(" {}", app.localizer.t("help_quit"))),
            Span::raw(" | "),
            Span::styled(" / ", Style::default().fg(Color::White).bg(Color::DarkGray)),
            Span::raw(format!(" {}", app.localizer.t("help_search"))),
            Span::raw(" | "),
            Span::styled(" Enter ", Style::default().fg(Color::White).bg(Color::DarkGray)),
            Span::raw(format!(" {}", app.localizer.t("help_install"))),
            Span::raw(" | "),
            Span::styled(" u ", Style::default().fg(Color::White).bg(Color::DarkGray)),
            Span::raw(format!(" {}", app.localizer.t("update_system_button"))),
            Span::raw(" ("),
            Span::styled(update_status, Style::default().fg(update_color)),
            Span::raw(")"),
        ]);

        let footer = Paragraph::new(footer_line)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(app.localizer.t("status"))
                .title_alignment(Alignment::Center));
        f.render_widget(footer, chunks[2]);
    }
}
