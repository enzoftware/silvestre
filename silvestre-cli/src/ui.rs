//! UI rendering with ratatui

use crate::app::{App, Screen};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, List, ListItem},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App) {
    match app.current_screen {
        Screen::Main => draw_main(f, app),
        Screen::FilterMenu => draw_filter_menu(f, app),
        Screen::ApplyFilter => draw_apply_filter(f, app),
        Screen::Info => draw_info(f, app),
        Screen::Help => draw_help(f, app),
        Screen::Processing => draw_processing(f, app),
    }
}

fn draw_main(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Length(15), Constraint::Min(3), Constraint::Length(2)].as_ref())
        .split(f.area());

    // Title and welcome
    let title = vec![
        Line::from(vec![
            Span::styled("╔════════════════════════════════════════╗", Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("║", Style::default().fg(Color::Cyan)),
            Span::styled(" 🐱 SILVESTRE - Image Processing CLI 🐱 ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled("║", Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("║", Style::default().fg(Color::Cyan)),
            Span::styled(" (Named after my magnificent cat!)         ", Style::default().fg(Color::Magenta).add_modifier(Modifier::ITALIC)),
            Span::styled("║", Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("╚════════════════════════════════════════╝", Style::default().fg(Color::Cyan)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Welcome, fellow feline admirer! 🐾", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        ]),
    ];

    let menu = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("Press "),
            Span::styled("f", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" to apply a "),
            Span::styled("filter", Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::raw("Press "),
            Span::styled("i", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" to inspect an "),
            Span::styled("image", Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::raw("Press "),
            Span::styled("h", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" for "),
            Span::styled("help", Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::raw("Press "),
            Span::styled("q", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" to "),
            Span::styled("quit", Style::default().fg(Color::Red)),
        ]),
    ];

    let welcome_block = Block::default()
        .title(" Silvestre's Main Menu ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    let welcome = Paragraph::new(title).block(welcome_block);
    f.render_widget(welcome, chunks[0]);

    let menu_block = Block::default()
        .title(" Options ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));
    let menu_widget = Paragraph::new(menu).block(menu_block);
    f.render_widget(menu_widget, chunks[1]);

    // Status bar
    let status = Paragraph::new(app.status_message.clone())
        .style(Style::default().fg(Color::White).bg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(status, chunks[2]);
}

fn draw_filter_menu(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Min(10), Constraint::Length(3)].as_ref())
        .split(f.area());

    // Filter list
    let filters: Vec<ListItem> = app
        .filters
        .iter()
        .enumerate()
        .map(|(idx, filter)| {
            let content = format!("  {} - {}", filter.name, filter.category);
            if idx == app.selected_filter {
                ListItem::new(content).style(
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                ListItem::new(content).style(Style::default().fg(Color::White))
            }
        })
        .collect();

    let filter_block = Block::default()
        .title(" 🐱 Available Filters (Press ↑↓ to navigate, Enter to select) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    let filter_list = List::new(filters).block(filter_block);
    f.render_widget(filter_list, chunks[0]);

    // Status bar
    let selected = &app.filters[app.selected_filter];
    let status = format!(
        "📝 {} - {} (Silvestre says: \"{}\")",
        selected.name, selected.category, selected.description
    );
    let status_widget = Paragraph::new(status)
        .style(Style::default().fg(Color::White).bg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(status_widget, chunks[1]);
}

fn draw_apply_filter(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(5),
                Constraint::Length(2),
            ]
            .as_ref(),
        )
        .split(f.area());

    let filter_name = &app.filters[app.selected_filter].name;

    // Input file
    let input_style = if app.selected_field == 0 {
        Style::default().bg(Color::Blue)
    } else {
        Style::default()
    };
    let input_block = Block::default()
        .title(" Input Image ")
        .borders(Borders::ALL)
        .style(input_style);
    let input_widget = Paragraph::new(app.input_file.clone()).block(input_block);
    f.render_widget(input_widget, chunks[0]);

    // Output file
    let output_style = if app.selected_field == 1 {
        Style::default().bg(Color::Blue)
    } else {
        Style::default()
    };
    let output_block = Block::default()
        .title(" Output Image ")
        .borders(Borders::ALL)
        .style(output_style);
    let output_widget = Paragraph::new(app.output_file.clone()).block(output_block);
    f.render_widget(output_widget, chunks[1]);

    // Filter parameters
    let params_style = if app.selected_field == 2 {
        Style::default().bg(Color::Blue)
    } else {
        Style::default()
    };
    let params_block = Block::default()
        .title(format!(" {} Parameters ", filter_name))
        .borders(Borders::ALL)
        .style(params_style);
    let params_widget = Paragraph::new(app.filter_params.clone()).block(params_block);
    f.render_widget(params_widget, chunks[2]);

    // Apply button
    let button_style = if app.selected_field == 3 {
        Style::default()
            .bg(Color::Green)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().bg(Color::DarkGray)
    };
    let button_block = Block::default()
        .title(" Apply Filter ")
        .borders(Borders::ALL)
        .style(button_style);
    let button_widget = Paragraph::new("Press Enter to apply 🐾")
        .block(button_block)
        .alignment(Alignment::Center);
    f.render_widget(button_widget, chunks[3]);

    // Info
    let info = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("Tab to switch fields • "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(" to cancel"),
        ]),
        Line::from(""),
        Line::from("Silvestre is watching your progress with great interest..."),
    ];
    let info_widget = Paragraph::new(info);
    f.render_widget(info_widget, chunks[4]);

    // Status bar
    let status = Paragraph::new(app.status_message.clone())
        .style(Style::default().fg(Color::White).bg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(status, chunks[5]);
}

fn draw_info(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Length(5), Constraint::Min(10), Constraint::Length(2)].as_ref())
        .split(f.area());

    // Input
    let input_block = Block::default()
        .title(" Image Path (🐱 Silvestre's curiosity) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    let input_widget = Paragraph::new(app.info_input.clone()).block(input_block);
    f.render_widget(input_widget, chunks[0]);

    // Info display
    let info_lines = vec![
        Line::from(""),
        Line::from("Waiting for image path..."),
        Line::from(""),
        Line::from("Once loaded, this will show:"),
        Line::from("  • Image dimensions"),
        Line::from("  • Color space (RGB, Grayscale, etc.)"),
        Line::from("  • Histogram statistics"),
        Line::from("  • File format"),
        Line::from(""),
        Line::from("Silvestre will inspect the image with her keen eyes 👀"),
    ];

    let info_block = Block::default()
        .title(" Image Information ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));
    let info_widget = Paragraph::new(info_lines).block(info_block);
    f.render_widget(info_widget, chunks[1]);

    // Status bar
    let status = Paragraph::new(app.status_message.clone())
        .style(Style::default().fg(Color::White).bg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(status, chunks[2]);
}

fn draw_help(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Min(1), Constraint::Length(2)].as_ref())
        .split(f.area());

    let help_text = vec![
        Line::from(vec![
            Span::styled("SILVESTRE - Image Processing CLI", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("FILTERS", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  Brightness   - Adjust image brightness (requires delta parameter)"),
        Line::from("  Contrast     - Adjust image contrast (requires factor parameter)"),
        Line::from("  Grayscale    - Convert image to grayscale"),
        Line::from("  Invert       - Invert all colors"),
        Line::from("  Crop         - Extract rectangular region (x, y, width, height)"),
        Line::from("  Mirror       - Flip image horizontally, vertically, or both"),
        Line::from("  Resize       - Change image dimensions (width, height)"),
        Line::from("  Rotate       - Rotate image by specified angle"),
        Line::from(""),
        Line::from(vec![
            Span::styled("TIPS FROM SILVESTRE 🐱", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  • Use Tab to navigate fields when applying filters"),
        Line::from("  • Press Esc to go back to the previous screen"),
        Line::from("  • The status bar shows helpful information"),
        Line::from("  • All operations are non-destructive on the original file"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press Esc to return to main menu", Style::default().fg(Color::Green)),
        ]),
    ];

    let help_block = Block::default()
        .title(" 🐾 Silvestre's Wisdom 🐾 ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));
    let help_widget = Paragraph::new(help_text).block(help_block);
    f.render_widget(help_widget, chunks[0]);

    // Status bar
    let status = Paragraph::new(app.status_message.clone())
        .style(Style::default().fg(Color::White).bg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(status, chunks[1]);
}

fn draw_processing(f: &mut Frame, _app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(10)
        .constraints([Constraint::Min(5)].as_ref())
        .split(f.area());

    let processing_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Processing Image...", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from("🐱 Silvestre is concentrating very hard... 😺"),
        Line::from(""),
        Line::from("███████████░░░░░░░░░░░░ 45%"),
        Line::from(""),
        Line::from("Please wait..."),
    ];

    let processing_block = Block::default()
        .title(" Processing ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    let processing_widget = Paragraph::new(processing_text)
        .block(processing_block)
        .alignment(Alignment::Center);
    f.render_widget(processing_widget, chunks[0]);
}
