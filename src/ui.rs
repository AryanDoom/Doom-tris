use std::{io, time::Instant};
//we need the mouse
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};
//all the imports 
use crate::{
    board::{PARTICLE_SIZE, PieceColor, SAND_HEIGHT, SAND_WIDTH, SandBoard},
    scoreboard::{self, GameStats},
};

const LOST_TEXT: &str = "█   █  ████  █   █\n █ █  █    █ █   █\n  █   █    █ █   █\n  █   █    █ █   █\n  █    ████   ███\n\n█       ███    ███   █████\n█      █   █  █        █  \n█      █   █   ███     █  \n█      █   █      █    █  \n█████   ███   ████     █  ";
//everything tat is drawn on the screen lives here 
pub fn draw(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    board: &SandBoard,
    piece_cells: &[(usize, usize)],
    piece_color: PieceColor,
    piece_width: usize,
    piece_height: usize,
    piece_x: usize,
    piece_y: usize,
    piece_landed: bool,
    dev_mode: bool,
    paused: bool,
    game_over: bool,
    stats: &GameStats,
    now: Instant,
) -> io::Result<()> {
    terminal.draw(|frame| { //main func call that makes ratatui draw a new frame 
            let compact_layout = frame.area().width < 88; //so if the terminal is too small we use a compact layout
            let (board_area, side_area) = if compact_layout {
                let areas = Layout::default() // this is the layout for the compact mode
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Min(8), Constraint::Length(8)])
                    .split(frame.area());
                (areas[0], areas[1])
            } else {
                let areas = Layout::default() //normal one dont change this pls veryhard to trail and error ur way through this 
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Length(58), Constraint::Min(28)])
                    .split(frame.area());
                (areas[0], areas[1])
            };
            let (stats_area, controls_area) = if compact_layout { //if compact layout we dont show the controls
                (side_area, None)
            } else {
                let areas = Layout::default() //again normal one
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(13), Constraint::Min(0)])
                    .split(side_area);
                (areas[0], Some(areas[1]))
            };
            let available_columns = board_area.width.saturating_sub(2).max(1) as usize;  // thsi si the number of coloumns that are displayed on the screen tyep shi
            let available_rows = board_area.height.saturating_sub(2).max(1) as usize;
            // group display pixels when needed so the full board fits, the particles still stay separate in the engine
            let scale_x = (SAND_WIDTH + available_columns * 2 - 1) / (available_columns * 2); // #maths
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
            
            //watch this part its very important it draws the board and the pieces and the sand and everything else
            //this is sum abomination of a code but it works so dont touch it unless i know how to do it (dedbugging was done with ai and sum hopes and dreams)
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
            scoreboard::draw_stats(frame, stats_area, stats, now, paused, compact_layout);

            // text on the right side hmmm,,
            let mode = if dev_mode { "dev" } else { "random" };
            let piece_controls = if dev_mode {
                "1  I piece\n2  O piece\n3  T piece\n4  S piece\n5  Z piece\n6  J piece\n7  L piece"
            } else {
                "next: random"
            };
            if let Some(controls_area) = controls_area {
                let controls_text = format!("g  random\nd  dev\nmode: {mode}\n← / →  move\n↓  drop faster\np  pause/resume\nfalls every 0.2s\n{piece_controls}\nq  quit");
                let controls = Paragraph::new(controls_text)
                    .style(Style::default().fg(Color::Gray).bg(Color::Black))
                    .block(Block::default().title(" controls ").borders(Borders::ALL));
                frame.render_widget(controls, controls_area);
            }
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
    Ok(())
}
