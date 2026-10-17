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
    layout::{Constraint, Direction, Layout}, 
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

const WIDTH: usize = 14;
const HEIGHT: usize = 20;
//the function name is clear dumahh
fn clear_spanning_groups(board: &mut [[bool; WIDTH]; HEIGHT]) { 
    let mut checked = [[false; WIDTH]; HEIGHT];
    // go throguh each cell in the thing to check if the cell is filled or nah
    for row in 0..HEIGHT {
        for col in 0..WIDTH {
            if !board[row][col] || checked[row][col] {
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
                        if board[next_row][next_col] && !checked[next_row][next_col] {
                            checked[next_row][next_col] = true;
                            pending.push((next_row, next_col));
                        }
                    }
                }
            }
            // if the group touches both the left and right edges, clear it
            if touches_left && touches_right {
                for (row, col) in group {
                    board[row][col] = false;
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
    let mut board = [[false; WIDTH]; HEIGHT];
    let mut piece_cells: Vec<(usize, usize)> = Vec::new();
    let mut piece_width = 0;
    let mut piece_height = 0;
    let mut piece_x = 0;
    let mut piece_y = 0;
    let mut piece_landed = false;
    let mut last_fall = Instant::now();

    loop {
        let mut just_landed = false;

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
            // add the bottom border of the board
            let board = Paragraph::new(board_text)
                .style(Style::default().fg(Color::Gray))
                .block(Block::default().title(" sand tetris ").borders(Borders::ALL));
            frame.render_widget(board, areas[0]);
            // text on the right side hmmm,,
            let controls = Paragraph::new(
                "← / →  move\n↓  drop faster\nfalls every 0.5s\n1  I piece\n2  O piece\n3  T piece\n4  S piece\n5  Z piece\n6  J piece\n7  L piece\nq  quit",
            )
                .block(Block::default().title(" controls ").borders(Borders::ALL));
            frame.render_widget(controls, areas[1]);
        })?;
        // handle user input and shi
        if event::poll(Duration::from_millis(50))? { //basically how fast the game will be (50 seemed good enuf)
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                    break;
                }
                //all the pieces and their shapes and sizes (if there is that why the option is used)
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
                // clear the falling piece when switching, but leave landed pieces there
                if let Some((cells, width, height)) = new_piece {
                    if !piece_landed {
                        for &(x, y) in &piece_cells {
                            board[piece_y + y][piece_x + x] = false;
                        }
                    }
                    // update the piece properties and place the new piece on the board
                    piece_cells = cells;
                    piece_width = width;
                    piece_height = height;
                    piece_x = (WIDTH - piece_width) / 2;
                    piece_y = 0;
                    piece_landed = false;
                    last_fall = Instant::now();

                    for &(x, y) in &piece_cells {
                        board[piece_y + y][piece_x + x] = true;
                    }
                }
                // handle piece movement based on key input
                if !piece_cells.is_empty() && !piece_landed {
                    let new_x = match key.code {
                        KeyCode::Left if piece_x > 0 => piece_x - 1,
                        KeyCode::Right if piece_x < WIDTH - piece_width => piece_x + 1,
                        _ => piece_x,
                    };
                    // if the new x position is different from the current position, update the board and piece position
                    if new_x != piece_x {
                        for &(x, y) in &piece_cells {
                            board[piece_y + y][piece_x + x] = false;
                        }
                        //checking if another thing is here or not
                        if piece_cells
                            .iter()
                            .all(|&(x, y)| !board[piece_y + y][new_x + x])
                        {
                            piece_x = new_x;
                        }
                        // update the with the new thing
                        for &(x, y) in &piece_cells {
                            board[piece_y + y][piece_x + x] = true;
                        }
                    }
                    // handle piece falling when the down key is pressed
                    if key.code == KeyCode::Down {
                        for &(x, y) in &piece_cells {
                            board[piece_y + y][piece_x + x] = false;
                        }
                        //check if falling is possible or nah
                        let can_fall = piece_y < HEIGHT - piece_height
                            && piece_cells
                                .iter()
                                .all(|&(x, y)| !board[piece_y + y + 1][piece_x + x]);
                        //if it can fall, then fall, if not, then land (lowk decartes type statement )
                        if can_fall {
                            piece_y += 1;
                        } else {
                            piece_landed = true;
                            just_landed = true;
                        }
                        // update the area with the new piece position
                        for &(x, y) in &piece_cells {
                            board[piece_y + y][piece_x + x] = true;
                        }
                        //tihs is to reset the last fall time so that the piece dosent fall too fast when the down key is held down
                        last_fall = Instant::now();
                    }
                }
            }
        }
        // handle automatic piece falling every 0.5 seconds
        if !piece_cells.is_empty()
            && !piece_landed
            && last_fall.elapsed() >= Duration::from_millis(500)
        {
            for &(x, y) in &piece_cells {
                board[piece_y + y][piece_x + x] = false;
            }
            // check if the piece can fall down one row
            let can_fall = piece_y < HEIGHT - piece_height
                && piece_cells
                    .iter()
                    .all(|&(x, y)| !board[piece_y + y + 1][piece_x + x]);
            //same thing bro 
            if can_fall {
                piece_y += 1;
            } else {
                piece_landed = true;
                just_landed = true;
            }
            for &(x, y) in &piece_cells {
                board[piece_y + y][piece_x + x] = true;
            }

            last_fall = Instant::now();
        }

        if just_landed {
            clear_spanning_groups(&mut board);
        }
    }   
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}








