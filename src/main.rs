//error for the error handling and io for the input/output handling
//crossterm for terminal manipulation and ratatui for building the UI
use std::{error::Error, io, time::{Duration, Instant}};

use crossterm::{
    event::{self, Event, KeyCode}, //keycode is for keeb events 
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}, //all the funcs we need for now
};
use ratatui::{
    backend::CrosstermBackend, //bach-end (larping classical music)
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Terminal,
};

const WIDTH: usize = 14;
const HEIGHT: usize = 20;
const LOST_TEXT: &str = "█   █  ████  █   █\n █ █  █    █ █   █\n  █   █    █ █   █\n  █   █    █ █   █\n  █    ████   ███\n\n█       ███    ███   █████\n█      █   █  █        █  \n█      █   █   ███     █  \n█      █   █      █    █  \n█████   ███   ████     █  ";

#[derive(Clone, Copy, PartialEq)] //i did not know this 
enum PieceColor { //there are 4 colours in the sandd tris thing 
    Yellow,
    Red,
    Green,
    Blue,
}
//all the pieces in the game and thier shapes
fn piece_shape(number: usize) -> (Vec<(usize, usize)>, usize, usize, PieceColor) {
    match number {
        1 => (vec![(0, 0), (1, 0), (2, 0), (3, 0)], 4, 1, PieceColor::Blue),
        2 => (vec![(0, 0), (1, 0), (0, 1), (1, 1)], 2, 2, PieceColor::Yellow),
        3 => (vec![(0, 0), (1, 0), (2, 0), (1, 1)], 3, 2, PieceColor::Green),
        4 => (vec![(1, 0), (2, 0), (0, 1), (1, 1)], 3, 2, PieceColor::Red),
        5 => (vec![(0, 0), (1, 0), (1, 1), (2, 1)], 3, 2, PieceColor::Green),
        6 => (vec![(0, 0), (0, 1), (1, 1), (2, 1)], 3, 2, PieceColor::Blue),
        _ => (vec![(2, 0), (0, 1), (1, 1), (2, 1)], 3, 2, PieceColor::Yellow),
    }
}

//the function name is clear dumahh
fn clear_spanning_groups(board: &mut [[Option<PieceColor>; WIDTH]; HEIGHT]) { 
    let mut checked = [[false; WIDTH]; HEIGHT];
    // go throguh each cell in the thing to check if the cell is filled or nah
    for row in 0..HEIGHT {
        for col in 0..WIDTH {
            let Some(color) = board[row][col] else {
                continue;
            };
            if checked[row][col] {
                continue;
            }

            let mut group = Vec::new();
            let mut pending = vec![(row, col)];
            let mut touches_left = false;
            let mut touches_right = false;
            checked[row][col] = true;
            // this is a flood fill algorithm to find all connected filled cells (this is all ais idea cuz we will have sand particles so i wasnt smart enough to know what to do)
            while let Some((current_row, current_col)) = pending.pop() {
                group.push((current_row, current_col));
                touches_left |= current_col == 0;
                touches_right |= current_col == WIDTH - 1;
                // check all 8 neighboring cells (including diagonals)
                for next_row in current_row.saturating_sub(1)..=(current_row + 1).min(HEIGHT - 1) {
                    for next_col in current_col.saturating_sub(1)..=(current_col + 1).min(WIDTH - 1) {
                        if board[next_row][next_col] == Some(color) && !checked[next_row][next_col] {
                            checked[next_row][next_col] = true;
                            pending.push((next_row, next_col));
                        }
                    }
                }
            }
            // if the group touches both the left and right edges, clear it
            if touches_left && touches_right {
                for (row, col) in group {
                    board[row][col] = None;
                }
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    match enable_raw_mode() { //launch the term (in RAW mode)
    Ok(()) => {}
    Err(error) => return Err(error.into()),
}
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?; // so that i can see if sum fked while running not in the og shell ()
    let backend = CrosstermBackend::new(stdout); // call backed
    let mut terminal = Terminal::new(backend)?; //call terminal
    let mut board = [[None; WIDTH]; HEIGHT];
    let mut piece_cells: Vec<(usize, usize)> = Vec::new();
    let mut piece_color = PieceColor::Blue;
    let mut piece_width = 0;
    let mut piece_height = 0;
    let mut piece_x = 0;
    let mut piece_y = 0;
    let mut piece_landed = false;
    let mut last_fall = Instant::now();
    //rn let dev mode be the defualt 
    let mut dev_mode = true; 
    let mut next_piece = None;
    let mut game_over = false;

    loop {
        // clear the falling piece when switching, but leave landed pieces there
        if let Some((cells, width, height, color)) = next_piece.take() {
            if !piece_landed {
                for &(x, y) in &piece_cells {
                    board[piece_y + y][piece_x + x] = None;
                }
            }

            // update the piece properties and place the new piece on the board
            piece_cells = cells;
            piece_width = width;
            piece_height = height;
            piece_color = color;
            piece_x = (WIDTH - piece_width) / 2;
            piece_y = 0;
            piece_landed = false;
            last_fall = Instant::now();
            for &(x, y) in &piece_cells {
                board[piece_y + y][piece_x + x] = Some(piece_color);
            }
        }

        let mut just_landed = false;

        terminal.draw(|frame| {
            let areas = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(32), Constraint::Length(18)])
                .split(frame.area());
            // render the game board
            let mut board_lines = Vec::new();
            for row in &board {
                let mut spans = vec![Span::styled("│", Style::default().fg(Color::Gray))];
                for cell in row {
                    if let Some(piece_color) = *cell { //this is to check if the chell is filled or nah
                        let color = match piece_color { //matching the piece colour to the term colour
                            PieceColor::Yellow => Color::Yellow,
                            PieceColor::Red => Color::Red,
                            PieceColor::Green => Color::Green,
                            PieceColor::Blue => Color::Blue,
                        };
                        spans.push(Span::styled("██", Style::default().fg(color)));
                    } else {
                        spans.push(Span::raw("  "));
                    }
                }
                spans.push(Span::styled("│", Style::default().fg(Color::Gray)));
                board_lines.push(Line::from(spans));
            }
            // add the bottom border of the board
            let board = Paragraph::new(board_lines)
                .block(Block::default().title(" sand tetris ").borders(Borders::ALL));
            frame.render_widget(board, areas[0]);
            // text on the right side hmmm,,
            let mode = if dev_mode { "dev" } else { "random" };
            let piece_controls = if dev_mode {
                "1  I piece\n2  O piece\n3  T piece\n4  S piece\n5  Z piece\n6  J piece\n7  L piece"
            } else {
                "next: random"
            };
            let controls = Paragraph::new(format!(
                "g  random\nd  dev\nmode: {mode}\n← / →  move\n↓  drop faster\nfalls every 0.5s\n{piece_controls}\nq  quit"
            ))
                .block(Block::default().title(" controls ").borders(Borders::ALL));
            frame.render_widget(controls, areas[1]);
            //if u lost lmaoooooo(this is for the text)
            if game_over {
                frame.render_widget(Clear, frame.area());
                let rows = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Min(0), Constraint::Length(11), Constraint::Min(0)])
                    .split(frame.area());
                let banner = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Min(0), Constraint::Length(30), Constraint::Min(0)])
                    .split(rows[1]);
                let lost = Paragraph::new(LOST_TEXT)
                    .alignment(Alignment::Left)
                    .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));
                frame.render_widget(lost, banner[1]);
            }
        })?;
        // handle user input and shi
        if event::poll(Duration::from_millis(50))? { //basically how fast the game will be (50 seemed good enuf)
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                    break;
                }
                if game_over {
                    continue;
                }
                if key.code == KeyCode::Char('g') || key.code == KeyCode::Char('G') {
                    dev_mode = false;
                    if !dev_mode && (piece_cells.is_empty() || piece_landed) {
                        next_piece = Some(piece_shape(rand::random_range(1..=7)));
                    }
                }
                if key.code == KeyCode::Char('d') || key.code == KeyCode::Char('D') {
                    dev_mode = true;
                }

                //all the pieces and their shapes and sizes (if there is that why the option is used)
                let new_piece = if dev_mode {
                    match key.code {
                        KeyCode::Char('1') => Some(piece_shape(1)),
                        KeyCode::Char('2') => Some(piece_shape(2)),
                        KeyCode::Char('3') => Some(piece_shape(3)),
                        KeyCode::Char('4') => Some(piece_shape(4)),
                        KeyCode::Char('5') => Some(piece_shape(5)),
                        KeyCode::Char('6') => Some(piece_shape(6)),
                        KeyCode::Char('7') => Some(piece_shape(7)),
                        _ => None,
                    }
                } else {
                    None
                };
                if let Some(shape) = new_piece {
                    next_piece = Some(shape);
                }
                // handle piece movement based on key input
                if next_piece.is_none() && !piece_cells.is_empty() && !piece_landed {
                    let new_x = match key.code {
                        KeyCode::Left if piece_x > 0 => piece_x - 1,
                        KeyCode::Right if piece_x < WIDTH - piece_width => piece_x + 1,
                        _ => piece_x,
                    };
                    // if the new x position is different from the current position, update the board and piece position
                    if new_x != piece_x {
                        for &(x, y) in &piece_cells {
                            board[piece_y + y][piece_x + x] = None;
                        }
                        //checking if another thing is here or not
                        if piece_cells
                            .iter()
                            .all(|&(x, y)| board[piece_y + y][new_x + x].is_none())
                        {
                            piece_x = new_x;
                        }
                        // update the with the new thing
                        for &(x, y) in &piece_cells {
                            board[piece_y + y][piece_x + x] = Some(piece_color);
                        }
                    }
                    // handle piece falling when the down key is pressed
                    if key.code == KeyCode::Down {
                        for &(x, y) in &piece_cells {
                            board[piece_y + y][piece_x + x] = None;
                        }
                        //check if falling is possible or nah
                        let can_fall = piece_y < HEIGHT - piece_height
                            && piece_cells
                                .iter()
                                .all(|&(x, y)| board[piece_y + y + 1][piece_x + x].is_none());
                        //if it can fall, then fall, if not, then land (lowk decartes type statement )
                        if can_fall {
                            piece_y += 1;
                        } else {
                            piece_landed = true;
                            just_landed = true;
                        }
                        // update the area with the new piece position
                        for &(x, y) in &piece_cells {
                            board[piece_y + y][piece_x + x] = Some(piece_color);
                        }
                        //tihs is to reset the last fall time so that the piece dosent fall too fast when the down key is held down
                        last_fall = Instant::now();
                    }
                }
            }
        }
        // handle automatic piece falling every 0.5 seconds
        if !game_over
            && !piece_cells.is_empty()
            && !piece_landed
            && last_fall.elapsed() >= Duration::from_millis(500)
        {
            for &(x, y) in &piece_cells {
                board[piece_y + y][piece_x + x] = None;
            }
            // check if the piece can fall down one row
            let can_fall = piece_y < HEIGHT - piece_height
                && piece_cells
                    .iter()
                    .all(|&(x, y)| board[piece_y + y + 1][piece_x + x].is_none());
            //same thing bro 
            if can_fall {
                piece_y += 1;
            } else {
                piece_landed = true;
                just_landed = true;
            }
            for &(x, y) in &piece_cells {
                board[piece_y + y][piece_x + x] = Some(piece_color);
            }

            last_fall = Instant::now();
        }
        //this is the main loss condition check, basically if the top row if filled then u lst
        if just_landed {
            clear_spanning_groups(&mut board);
            game_over = board[0].iter().any(|cell| cell.is_some());
            if !dev_mode && !game_over {
                next_piece = Some(piece_shape(rand::random_range(1..=7)));
            }
        }
    }   
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}








