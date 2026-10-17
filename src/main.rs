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

const WIDTH: usize = 14;
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
    let mut board = [[false; WIDTH]; HEIGHT];
    let mut piece_cells: Vec<(usize, usize)> = Vec::new();
    let mut piece_width = 0;
    let mut piece_x = 0;
    let mut piece_y = 0;

    loop {
        terminal.draw(|frame| {
            let areas = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(32), Constraint::Length(18)])
                .split(frame.area());

            let mut board_text = String::new();
            for row in &board {
                board_text.push('│');
                for cell in row {
                    if *cell {
                        board_text.push_str("██");
                    } else {
                        board_text.push_str("  ");
                    }
                }
                board_text.push_str("│\n");
            }

            let board = Paragraph::new(board_text)
                .style(Style::default().fg(Color::Gray))
                .block(Block::default().title(" sand tetris ").borders(Borders::ALL));
            frame.render_widget(board, areas[0]);

            let controls = Paragraph::new(
                "← / →  move\n1  I piece\n2  O piece\n3  T piece\n4  S piece\n5  Z piece\n6  J piece\n7  L piece\nq  quit",
            )
                .block(Block::default().title(" controls ").borders(Borders::ALL));
            frame.render_widget(controls, areas[1]);
        })?;

        if let Event::Key(key) = event::read()? {
            if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                break;
            }

            let new_piece = match key.code {
                KeyCode::Char('1') => Some((vec![(0, 0), (1, 0), (2, 0), (3, 0)], 4, 1)),
                KeyCode::Char('2') => Some((vec![(0, 0), (1, 0), (0, 1), (1, 1)], 2, 2)),
                KeyCode::Char('3') => Some((vec![(0, 0), (1, 0), (2, 0), (1, 1)], 3, 2)),
                KeyCode::Char('4') => Some((vec![(1, 0), (2, 0), (0, 1), (1, 1)], 3, 2)),
                KeyCode::Char('5') => Some((vec![(0, 0), (1, 0), (1, 1), (2, 1)], 3, 2)),
                KeyCode::Char('6') => Some((vec![(0, 0), (0, 1), (1, 1), (2, 1)], 3, 2)),
                KeyCode::Char('7') => Some((vec![(2, 0), (0, 1), (1, 1), (2, 1)], 3, 2)),
                _ => None,
            };

            if let Some((cells, width, height)) = new_piece {
                for &(x, y) in &piece_cells {
                    board[piece_y + y][piece_x + x] = false;
                }

                piece_cells = cells;
                piece_width = width;
                piece_x = (WIDTH - piece_width) / 2;
                piece_y = HEIGHT - height;

                for &(x, y) in &piece_cells {
                    board[piece_y + y][piece_x + x] = true;
                }
            }

            if !piece_cells.is_empty() {
                let new_x = match key.code {
                    KeyCode::Left if piece_x > 0 => piece_x - 1,
                    KeyCode::Right if piece_x < WIDTH - piece_width => piece_x + 1,
                    _ => piece_x,
                };

                if new_x != piece_x {
                    for &(x, y) in &piece_cells {
                        board[piece_y + y][piece_x + x] = false;
                    }

                    piece_x = new_x;

                    for &(x, y) in &piece_cells {
                        board[piece_y + y][piece_x + x] = true;
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}








