//hardset widht and hiehgt of the game board
pub const WIDTH: usize = 14;
pub const HEIGHT: usize = 18;
pub const PARTICLE_SIZE: usize = 15;
pub const SAND_WIDTH: usize = WIDTH * PARTICLE_SIZE;
pub const SAND_HEIGHT: usize = HEIGHT * PARTICLE_SIZE;

#[derive(Clone, Copy, PartialEq)] //i did not know this 
pub enum PieceColor {
    //there are 4 colours in the sandd tris thing
    Yellow,
    Red,
    Green,
    Blue,
}
pub type SandBoard = [[Option<PieceColor>; SAND_WIDTH]; SAND_HEIGHT];
//all the pieces in the game and thier shapes
pub fn piece_shape(number: usize) -> (Vec<(usize, usize)>, usize, usize, PieceColor) {
    match number {
        1 => (vec![(0, 0), (1, 0), (2, 0), (3, 0)], 4, 1, PieceColor::Blue),
        2 => (
            vec![(0, 0), (1, 0), (0, 1), (1, 1)],
            2,
            2,
            PieceColor::Yellow,
        ),
        3 => (
            vec![(0, 0), (1, 0), (2, 0), (1, 1)],
            3,
            2,
            PieceColor::Green,
        ),
        4 => (vec![(1, 0), (2, 0), (0, 1), (1, 1)], 3, 2, PieceColor::Red),
        5 => (
            vec![(0, 0), (1, 0), (1, 1), (2, 1)],
            3,
            2,
            PieceColor::Green,
        ),
        6 => (vec![(0, 0), (0, 1), (1, 1), (2, 1)], 3, 2, PieceColor::Blue),
        _ => (
            vec![(2, 0), (0, 1), (1, 1), (2, 1)],
            3,
            2,
            PieceColor::Yellow,
        ),
    }
}

//the function name is clear dumahh
pub fn clear_spanning_groups(board: &mut SandBoard) -> usize {
    let mut checked = [[false; SAND_WIDTH]; SAND_HEIGHT];
    let mut rows_cleared = 0;
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
                // check all 8 neighboring cells (including diagonals)(cuz i remeber it like that)
                for next_row in
                    current_row.saturating_sub(1)..=(current_row + 1).min(SAND_HEIGHT - 1)
                {
                    for next_col in
                        current_col.saturating_sub(1)..=(current_col + 1).min(SAND_WIDTH - 1)
                    {
                        if board[next_row][next_col] == Some(color) && !checked[next_row][next_col]
                        {
                            checked[next_row][next_col] = true;
                            pending.push((next_row, next_col));
                        }
                    }
                }
            }
            // if the group touches both the left and right edges, clear it
            if touches_left && touches_right {
                let mut row_counts = [0; SAND_HEIGHT];
                for (row, col) in group {
                    row_counts[row] += 1;
                    board[row][col] = None;
                }
                let mut counted_game_rows = [false; HEIGHT];
                for (row, count) in row_counts.into_iter().enumerate() {
                    if count == SAND_WIDTH {
                        let game_row = row / PARTICLE_SIZE;
                        if !counted_game_rows[game_row] {
                            counted_game_rows[game_row] = true;
                            rows_cleared += 1;
                        }
                    }
                }
            }
        }
    }
    rows_cleared
}
//this  is the main thing that checks if the sand can fall down or nah or can it go left or righ or nah
pub fn update_sand(board: &mut SandBoard) {
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
                [
                    col.checked_sub(1),
                    (col + 1 < SAND_WIDTH).then_some(col + 1),
                ]
            } else {
                [
                    (col + 1 < SAND_WIDTH).then_some(col + 1),
                    col.checked_sub(1),
                ]
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

//to check if a block can be placed at said position
pub fn can_place_piece(board: &SandBoard, cells: &[(usize, usize)], x: usize, y: usize) -> bool {
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


//to lock a block in place on the board
pub fn lock_piece(
    board: &mut SandBoard,
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
