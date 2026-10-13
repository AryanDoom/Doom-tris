//error for the error handling and io for the input/output handling
//crossterm for terminal manipulation and ratatui for building the UI
use std::{error::Error, io};

use crossterm::{
    event::{self, Event, KeyCode}, //keycode is for keeb events 
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}, //all the funcs we need for now
};
use ratatui::{
    backend::CrosstermBackend, //bach-end (larping classical music)
    layout::{Constraint, Direction, Layout}, 
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

const WIDTH: usize = 10;
const HEIGHT: usize = 20;

fn main() -> Result<(), Box<dyn Error>> {
    match enable_raw_mode() { //launch the term (in RAW mode)
    Ok(()) => {}
    Err(error) => return Err(error.into()),
}
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|frame| {
            let areas = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(24), Constraint::Min(0)])
                .split(frame.area());

            let mut board_text = String::new();
            for _ in 0..HEIGHT {
                board_text.push_str(&format!("│{}│\n", "  ".repeat(WIDTH)));
            }

            let board = Paragraph::new(board_text)
                .style(Style::default().fg(Color::Gray))
                .block(Block::default().title(" sand tetris ").borders(Borders::ALL));
            frame.render_widget(board, areas[0]);

            let controls = Paragraph::new("q  quit")
                .block(Block::default().title(" controls ").borders(Borders::ALL));
            frame.render_widget(controls, areas[1]);
        })?;

        if let Event::Key(key) = event::read()? {
            if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                break;
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
