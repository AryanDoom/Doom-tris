sand tetris
A terminal Tetris game where each landed piece turns into sand particles. I’m building it in Rust with Ratatui and Crossterm.
how the board works
There are two board sizes to keep track of:
- The Tetris grid is 14 columns by 18 rows.
- Each Tetris cell expands to a 15 × 15 patch of particles, making the sand grid 210 × 270.
The falling piece is stored separately from the sand grid. Its shape is a list of cell offsets, plus its width, height, position, and colour. When checking whether it can move or land, each piece cell is expanded to its 15 × 15 area and checked against the sand grid. When it lands, those areas are written into the grid as particles.
The grid stores each particle as either empty or a colour. On each sand update, the simulation scans from the bottom row upward. A particle moves down if the space below it is empty; if it is blocked, it tries a diagonal space. The order of the diagonal checks is randomized so particles don’t always favour the same side. Scanning bottom to top helps stop a particle from moving more than one row in a single update.
connection clearing
The clearing rule works on connected particles, not just full Tetris rows. For each unvisited particle, the game uses a flood fill to find the group of particles with the same colour connected to it. The search checks all 8 neighbours, so diagonal connections count too.
If a group reaches both the left and right edges, the whole group is removed. The game counts a cleared row when that group fully spans the particle grid within a logical 15-particle-high row. It checks for connections after a piece locks and during the regular sand updates, since moving sand can create a connection too.
update and rendering
The game loop handles input, piece gravity, landing, and the sand update. It draws the frame after those state changes so the display uses the latest board state.
The piece falls every 200 ms. The sand simulation updates every 10 ms, and input is checked on a 10 ms poll. The terminal redraws once per loop.
The board is rendered with Braille characters, which each represent 8 dots. The renderer groups particles as needed to fit the current terminal size. When the terminal is under 88 columns wide, the stats and controls move below the board.
score
The score is calculated from the stats instead of being incremented separately:
score = active seconds × 100
      + blocks placed × 250
      + rows cleared × 1,000
The timer starts when the first piece spawns. It uses elapsed time, so it doesn’t depend on a timer callback firing exactly on schedule. Pausing saves the elapsed time and resuming starts counting again. Game over stops the timer.
controls
Key	Action
← / →	Move the piece
↓	Move down one row
1–7	Spawn I, O, T, S, Z, J, or L in dev mode
g	Switch to random mode
d	Switch to dev mode
p	Pause or resume
q or Esc	Quit


The game starts empty in dev mode, so press a number to spawn a piece. In random mode, another piece is chosen after the current one lands.
code layout
- src/game.rs — game loop, input, gravity, and landing
- src/board.rs — board dimensions, piece shapes, collision checks, particle movement, and connected-group clearing
- src/scoreboard.rs — elapsed time, counters, score calculation, and stats panel
- src/ui.rs — board, controls, and game-over rendering
- src/main.rs — starts the game
run it
You need Rust and Cargo with support for the 2024 edition.
cargo run
Run the tests with:
cargo test
current limits
Pieces can move left and right, but they can’t rotate or hard drop. Random mode also doesn’t show a next-piece preview. The sand movement is intentionally basic for now: each particle only tries to move down or diagonally
