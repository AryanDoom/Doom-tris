//error for the error handling and io for the input/output handling
//crossterm for terminal manipulation and ratatui for building the UI
use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, Event, KeyCode}, //keycode is for keeb events
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode}, //all the funcs we need for now
};
use ratatui::{Terminal, backend::CrosstermBackend};

use crate::{
    board::{
        HEIGHT, PieceColor, SAND_HEIGHT, SAND_WIDTH, SandBoard, WIDTH, can_place_piece,
        clear_spanning_groups, lock_piece, piece_shape, update_sand,
    },
    scoreboard::GameStats,
    ui,
};
pub fn run() -> Result<(), Box<dyn Error>> {
    match enable_raw_mode() {
        //launch the term (in RAW mode)
        Ok(()) => {}
        Err(error) => return Err(error.into()),
    }
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?; // so that i can see if sum fked while running not in the og shell ()
    let backend = CrosstermBackend::new(stdout); // call backed
    let mut terminal = Terminal::new(backend)?; //call terminal
    let mut board: SandBoard = [[None; SAND_WIDTH]; SAND_HEIGHT];
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
    let mut paused = false;
    let mut stats = GameStats::new();

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
            stats.start(last_fall);
            if !can_place_piece(&board, &piece_cells, piece_x, piece_y) {
                game_over = true;
                stats.stop(Instant::now());
            }
        }

        let mut just_landed = false;

        // handle user input and shi
        if event::poll(Duration::from_millis(10))? {
            //basically how fast the game will be (10 seemed good enuf)
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                    break;
                }
                if key.code == KeyCode::Char('p') || key.code == KeyCode::Char('P') {
                    if !game_over && stats.is_started() {
                        paused = !paused;
                        let now = Instant::now();
                        if paused {
                            stats.pause(now);
                        } else {
                            stats.resume(now);
                            last_fall = now;
                            last_sand_update = now;
                        }
                    }
                    continue;
                }
                if game_over || paused {
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
            && !paused
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
            stats.record_block_placed();
            stats.record_lines_cleared(clear_spanning_groups(&mut board));
            game_over = board[0].iter().any(|cell| cell.is_some());
            if !dev_mode && !game_over {
                next_piece = Some(piece_shape(rand::random_range(1..=7)));
            }
        }

        if !game_over && !paused && last_sand_update.elapsed() >= Duration::from_millis(10) {
            update_sand(&mut board);
            stats.record_lines_cleared(clear_spanning_groups(&mut board));
            last_sand_update = Instant::now();
        }
        if game_over {
            stats.stop(Instant::now());
        }
        let now = Instant::now();
        ui::draw(
            &mut terminal,
            &board,
            &piece_cells,
            piece_color,
            piece_width,
            piece_height,
            piece_x,
            piece_y,
            piece_landed,
            dev_mode,
            paused,
            game_over,
            &stats,
            now,
        )?;
    }
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
