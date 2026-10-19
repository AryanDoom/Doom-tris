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
//hardset widht and hiehgt of the game board
const WIDTH: usize = 14;
const HEIGHT: usize = 18;
const PARTICLE_SIZE: usize = 15;
const SAND_WIDTH: usize = WIDTH * PARTICLE_SIZE;
const SAND_HEIGHT: usize = HEIGHT * PARTICLE_SIZE;
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
fn clear_spanning_groups(board: &mut [[Option<PieceColor>; SAND_WIDTH]; SAND_HEIGHT]) { 
    let mut checked = [[false; SAND_WIDTH]; SAND_HEIGHT];
    // go throguh each cell in the thing to check if the cell is filled or nah
    for row in 0..SAND_HEIGHT {
        for col in 0..SAND_WIDTH {
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
                touches_right |= current_col == SAND_WIDTH - 1;
                // check all 8 neighboring cells (including diagonals)
                for next_row in current_row.saturating_sub(1)..=(current_row + 1).min(SAND_HEIGHT - 1) {
                    for next_col in current_col.saturating_sub(1)..=(current_col + 1).min(SAND_WIDTH - 1) {
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

fn update_sand(
    board: &mut [[Option<PieceColor>; SAND_WIDTH]; SAND_HEIGHT],
) {
    for row in (0..SAND_HEIGHT - 1).rev() {
        for col in 0..SAND_WIDTH {
            let Some(color) = board[row][col] else {
                continue;
            };

            if board[row + 1][col].is_none() {
                board[row + 1][col] = Some(color);
                board[row][col] = None;
                continue;
            }

            let left_first = rand::random::<bool>();
            let diagonal_cols = if left_first {
                [col.checked_sub(1), (col + 1 < SAND_WIDTH).then_some(col + 1)]
            } else {
                [(col + 1 < SAND_WIDTH).then_some(col + 1), col.checked_sub(1)]
            };
            for diagonal_col in diagonal_cols.into_iter().flatten() {
                if board[row + 1][diagonal_col].is_none() {
                    board[row + 1][diagonal_col] = Some(color);
                    board[row][col] = None;
                    break;
                }
            }
        }
    }
}

fn can_place_piece(
    board: &[[Option<PieceColor>; SAND_WIDTH]; SAND_HEIGHT],
    cells: &[(usize, usize)],
    x: usize,
    y: usize,
) -> bool {
    for &(cell_x, cell_y) in cells {
        for sand_y in 0..PARTICLE_SIZE {
            for sand_x in 0..PARTICLE_SIZE {
                let board_x = (x + cell_x) * PARTICLE_SIZE + sand_x;
                let board_y = (y + cell_y) * PARTICLE_SIZE + sand_y;
                if board[board_y][board_x].is_some() {
                    return false;
                }
            }
        }
    }
    true
}

fn lock_piece(
    board: &mut [[Option<PieceColor>; SAND_WIDTH]; SAND_HEIGHT],
    cells: &[(usize, usize)],
    x: usize,
    y: usize,
    color: PieceColor,
) {
    for &(cell_x, cell_y) in cells {
        for sand_y in 0..PARTICLE_SIZE {
            for sand_x in 0..PARTICLE_SIZE {
                let board_x = (x + cell_x) * PARTICLE_SIZE + sand_x;
                let board_y = (y + cell_y) * PARTICLE_SIZE + sand_y;
                board[board_y][board_x] = Some(color);
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
    let mut board = [[None; SAND_WIDTH]; SAND_HEIGHT];
    let mut piece_cells: Vec<(usize, usize)> = Vec::new();
    let mut piece_color = PieceColor::Blue;
    let mut piece_width = 0;
    let mut piece_height = 0;
    let mut piece_x = 0;
    let mut piece_y = 0;
    let mut piece_landed = false;
    let mut last_fall = Instant::now();
    let mut last_sand_update = Instant::now();
    //rn let dev mode be the defualt 
    let mut dev_mode = true; 
    let mut next_piece = None;
    let mut game_over = false;

    loop {
        // replace the falling piece when switching, but leave landed particles there
        if let Some((cells, width, height, color)) = next_piece.take() {
            // update the piece properties; the falling piece is drawn separately
            piece_cells = cells;
            piece_width = width;
            piece_height = height;
            piece_color = color;
            piece_x = (WIDTH - piece_width) / 2;
            piece_y = 0;
            piece_landed = false;
            last_fall = Instant::now();
            if !can_place_piece(&board, &piece_cells, piece_x, piece_y) {
                game_over = true;
            }
        }

        let mut just_landed = false;

        // handle user input and shi
        if event::poll(Duration::from_millis(10))? { //basically how fast the game will be (10 seemed good enuf)
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
                        //checking if another thing is here or not
                        if can_place_piece(&board, &piece_cells, new_x, piece_y) {
                            piece_x = new_x;
                            // update the with the new thing
                        }
                    }
                    // handle piece falling when the down key is pressed
                    if key.code == KeyCode::Down {
                        //check if falling is possible or nah
                        let can_fall = piece_y < HEIGHT - piece_height
                            && can_place_piece(&board, &piece_cells, piece_x, piece_y + 1);
                        //if it can fall, then fall, if not, then land (lowk decartes type statement )
                        if can_fall {
                            piece_y += 1;
                        } else {
                            piece_landed = true;
                            just_landed = true;
                        }
                        //tihs is to reset the last fall time so that the piece dosent fall too fast when the down key is held down
                        // update the area with the new piece position
                        last_fall = Instant::now();
                    }
                }
            }
        }
        // handle automatic piece falling every 0.2 seconds
        if !game_over
            && !piece_cells.is_empty()
            && !piece_landed
            && last_fall.elapsed() >= Duration::from_millis(200)
        {
            // check if the piece can fall down one row
            let can_fall = piece_y < HEIGHT - piece_height
                && can_place_piece(&board, &piece_cells, piece_x, piece_y + 1);
            //same thing bro 
            if can_fall {
                piece_y += 1;
            } else {
                piece_landed = true;
                just_landed = true;
            }
            last_fall = Instant::now();
        }
        //this is the main loss condition check, basically if the top row if filled then u lst
        if just_landed {
            lock_piece(&mut board, &piece_cells, piece_x, piece_y, piece_color);
            clear_spanning_groups(&mut board);
            game_over = board[0].iter().any(|cell| cell.is_some());
            if !dev_mode && !game_over {
                next_piece = Some(piece_shape(rand::random_range(1..=7)));
            }
        }

        if !game_over && last_sand_update.elapsed() >= Duration::from_millis(10) {
            clear_spanning_groups(&mut board);
            update_sand(&mut board);
            last_sand_update = Instant::now();
        }
        terminal.draw(|frame| {
            let compact_layout = frame.area().width < 80;
            let (board_area, controls_area) = if compact_layout {
                let areas = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Min(0), Constraint::Length(8)])
                    .split(frame.area());
                (areas[0], areas[1])
            } else {
                let areas = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Length(58), Constraint::Min(18)])
                    .split(frame.area());
                (areas[0], areas[1])
            };
            let available_columns = board_area.width.saturating_sub(2).max(1) as usize;
            let available_rows = board_area.height.saturating_sub(2).max(1) as usize;
            // group display pixels when needed so the full board fits, the particles still stay separate in the engine
            let scale_x = (SAND_WIDTH + available_columns * 2 - 1) / (available_columns * 2);
            let scale_y = (SAND_HEIGHT + available_rows * 4 - 1) / (available_rows * 4);
            let fit_scale = scale_x.max(scale_y).max(1);
            // keep the full board at its fit size
            let particle_scale = fit_scale;
            let visible_width = (available_columns * 2 * particle_scale).min(SAND_WIDTH);
            let visible_height = (available_rows * 4 * particle_scale).min(SAND_HEIGHT);
            let piece_center_x = if piece_cells.is_empty() {
                SAND_WIDTH / 2
            } else {
                (piece_x + piece_width / 2) * PARTICLE_SIZE
            };
            let piece_center_y = if piece_cells.is_empty() {
                SAND_HEIGHT / 2
            } else {
                (piece_y + piece_height / 2) * PARTICLE_SIZE
            };
            let view_left = piece_center_x
                .saturating_sub(visible_width / 2)
                .min(SAND_WIDTH - visible_width);
            let view_top = piece_center_y
                .saturating_sub(visible_height / 2)
                .min(SAND_HEIGHT - visible_height);
            let display_columns = (visible_width + particle_scale * 2 - 1) / (particle_scale * 2);
            let display_rows = (visible_height + particle_scale * 4 - 1) / (particle_scale * 4);
            // render the game board
            let mut board_lines = Vec::new();
            for display_row in 0..display_rows {
                let mut spans = vec![Span::styled("│", Style::default().fg(Color::Gray))];
                for display_col in 0..display_columns {
                    let mut mask = 0u32;
                    let mut color_counts = [0; 4];
                    for pixel_y in 0..4 {
                        for pixel_x in 0..2 {
                            let start_y = view_top + (display_row * 4 + pixel_y) * particle_scale;
                            let start_x = view_left + (display_col * 2 + pixel_x) * particle_scale;
                            let mut dot_color_counts = [0; 4];
                            for sand_y in start_y..(start_y + particle_scale).min(view_top + visible_height) {
                                for sand_x in start_x..(start_x + particle_scale).min(view_left + visible_width) {
                                    let active_piece_cell = !piece_landed
                                        && piece_cells.iter().any(|&(x, y)| {
                                            sand_x / PARTICLE_SIZE == piece_x + x
                                                && sand_y / PARTICLE_SIZE == piece_y + y
                                        });
                                    let color = board[sand_y][sand_x].or_else(|| {
                                        if active_piece_cell { Some(piece_color) } else { None }
                                    });
                                    if let Some(color) = color {
                                        let color_index = match color {
                                            PieceColor::Yellow => 0,
                                            PieceColor::Red => 1,
                                            PieceColor::Green => 2,
                                            PieceColor::Blue => 3,
                                        };
                                        dot_color_counts[color_index] += 1;
                                    }
                                }
                            }
                            if let Some((color_index, count)) = dot_color_counts.iter().enumerate().max_by_key(|(_, count)| *count) {
                                if *count > 0 {
                                    let dot = match (pixel_x, pixel_y) {
                                        (0, 0) => 0,
                                        (0, 1) => 1,
                                        (0, 2) => 2,
                                        (0, 3) => 6,
                                        (1, 0) => 3,
                                        (1, 1) => 4,
                                        (1, 2) => 5,
                                        _ => 7,
                                    };
                                    mask |= 1 << dot;
                                    color_counts[color_index] += count;
                                }
                            }
                        }
                    }
                    if mask == 0 {
                        spans.push(Span::raw(" "));
                    } else {
                        let color = match color_counts.iter().enumerate().max_by_key(|(_, count)| *count).unwrap().0 {
                            0 => Color::Yellow,
                            1 => Color::Red,
                            2 => Color::Green,
                            _ => Color::Blue,
                        };
                        let glyph = char::from_u32(0x2800 + mask).unwrap_or(' ');
                        spans.push(Span::styled(glyph.to_string(), Style::default().fg(color)));
                    }
                }
                spans.push(Span::styled("│", Style::default().fg(Color::Gray)));
                board_lines.push(Line::from(spans));
            }
            // add the bottom border of the board
            let board = Paragraph::new(board_lines)
                .alignment(Alignment::Center)
                .block(Block::default().title(" sand tetris ").borders(Borders::ALL));
            frame.render_widget(board, board_area);
            // text on the right side hmmm,,
            let mode = if dev_mode { "dev" } else { "random" };
            let piece_controls = if dev_mode {
                "1  I piece\n2  O piece\n3  T piece\n4  S piece\n5  Z piece\n6  J piece\n7  L piece"
            } else {
                "next: random"
            };
            let controls_text = if compact_layout {
                format!("{mode} mode   ←/→ move   ↓ drop   1-7 pieces\ng random   d dev   q quit")
            } else {
                format!("g  random\nd  dev\nmode: {mode}\n← / →  move\n↓  drop faster\nfalls every 0.2s\n{piece_controls}\nq  quit")
            };
            let controls = Paragraph::new(controls_text)
                .block(Block::default().title(" controls ").borders(Borders::ALL));
            frame.render_widget(controls, controls_area);
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
    }   
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}








