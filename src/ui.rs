use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
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
        crate::app::InputMode::Normal => Style::default(),
        crate::app::InputMode::Editing => Style::default().fg(Color::Yellow),
    };

    let input = Paragraph::new(app.search_input.as_str())
        .style(input_style)
        .block(Block::default().borders(Borders::ALL).title("Search (Press /)"));
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
                    crate::models::PackageSource::Aur => Color::Yellow,
                }
            };
            
            let status_mark = if pkg.is_installed { "[I]" } else { "   " };
            let line = format!("{} {:<20} {}", status_mark, pkg.name, pkg.version);
            ListItem::new(line).style(Style::default().fg(color))
        })
        .collect();

    let title = if app.is_loading { "Packages (Loading...)" } else { "Packages" };
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(Style::default().add_modifier(ratatui::style::Modifier::REVERSED));
    
    let mut state = ratatui::widgets::ListState::default();
    state.select(app.selected_index);
        
    f.render_stateful_widget(list, chunks[1], &mut state);

    // Console Overlay
    if app.show_console {
        let console_block = Block::default()
            .borders(Borders::ALL)
            .title("Command Output (Type 'y'/'n' to interact, 'Esc' to close)")
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
        let block = Block::default().title("Sudo Password Required").borders(Borders::ALL);
        let area = centered_rect(60, 20, f.size());
        
        f.render_widget(ratatui::widgets::Clear, area);
        f.render_widget(block, area);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Length(3)].as_ref())
            .margin(2)
            .split(area);
            
        let msg = Paragraph::new("Please enter your sudo password:");
        f.render_widget(msg, layout[0]);
        
        let width = app.password_input.len();
        let masked: String = "*".repeat(width);
        let input = Paragraph::new(masked)
            .block(Block::default().borders(Borders::ALL).style(Style::default().fg(Color::Yellow)));
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
            let action_str = if first.is_installed { "REMOVE" } else { "INSTALL" };
            
            let title = format!("Confirm {} Packages", if first.is_installed { "Removal" } else { "Installation" });
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
                format!("Are you sure you want to {} {} packages?", action_str, pkg_count)
            } else {
                format!("Are you sure you want to {} {}?", action_str, first.name)
            };
            
            let p_text = Paragraph::new(text).style(Style::default().fg(Color::White).add_modifier(ratatui::style::Modifier::BOLD));
            f.render_widget(p_text, layout[0]);
            
            let p_actions = Paragraph::new("Press 'y' to proceed, 'n' to cancel").style(Style::default().fg(Color::Gray));
            f.render_widget(p_actions, layout[1]);
        }
        return;
    }

    // Footer
    if let Some(err) = &app.error_message {
        let footer = Paragraph::new(format!("Error: {}", err))
            .style(Style::default().fg(Color::Red))
            .block(Block::default().borders(Borders::ALL).title("Error"));
        f.render_widget(footer, chunks[2]);
    } else {
        let footer = Paragraph::new("Esc: Quit | /: Search | Enter: Install")
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::ALL).title("Status"));
        f.render_widget(footer, chunks[2]);
    }
}
