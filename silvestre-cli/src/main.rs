mod app;
mod ui;
mod handlers;

use app::App;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;

    // Run app
    let app = App::new();
    let res = run_app(terminal, app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        std::io::stdout(),
        LeaveAlternateScreen
    )?;

    if let Err(err) = res {
        println!("Error: {}", err);
    }

    Ok(())
}

fn run_app<B: Backend>(
    mut terminal: Terminal<B>,
    mut app: App,
) -> Result<(), Box<dyn Error>> {
    loop {
        terminal.draw(|f| ui::draw(f, &app))?;

        if crossterm::event::poll(std::time::Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                match app.current_screen {
                    app::Screen::Main => {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                            KeyCode::Char('f') => app.go_to_filter_menu(),
                            KeyCode::Char('i') => app.go_to_info(),
                            KeyCode::Char('h') => app.go_to_help(),
                            _ => {}
                        }
                    }
                    app::Screen::FilterMenu => {
                        match key.code {
                            KeyCode::Esc => app.go_to_main(),
                            KeyCode::Up => app.select_previous_filter(),
                            KeyCode::Down => app.select_next_filter(),
                            KeyCode::Enter => app.go_to_apply_filter(),
                            _ => {}
                        }
                    }
                    app::Screen::ApplyFilter => {
                        match key.code {
                            KeyCode::Esc => app.go_to_main(),
                            KeyCode::Tab => app.next_field(),
                            KeyCode::BackTab => app.prev_field(),
                            KeyCode::Char(c) => app.input_char(c),
                            KeyCode::Backspace => app.input_backspace(),
                            KeyCode::Enter => {
                                if app.is_apply_button_focused() {
                                    app.apply_filter_action();
                                }
                            }
                            _ => {}
                        }
                    }
                    app::Screen::Info => {
                        match key.code {
                            KeyCode::Esc => app.go_to_main(),
                            KeyCode::Char(c) => app.info_input_char(c),
                            KeyCode::Backspace => app.info_input_backspace(),
                            KeyCode::Enter => app.load_image_info(),
                            _ => {}
                        }
                    }
                    app::Screen::Help => {
                        match key.code {
                            KeyCode::Esc => app.go_to_main(),
                            _ => {}
                        }
                    }
                    app::Screen::Processing => {
                        // Processing screen doesn't handle input
                    }
                }
            }
        }

        // Check if processing is done
        if app.is_processing_done() {
            app.go_to_main();
        }
    }
}
