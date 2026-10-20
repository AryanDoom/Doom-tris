# sand tetris

A terminal Tetris game where each landed piece turns into sand particles. I’m building it in Rust with Ratatui and Crossterm.

## how the board works

There are two board sizes to keep track of:

- The Tetris grid is 14 columns by 18 rows.
- Each Tetris cell expands to a 15 × 15 patch of particles, making the sand grid 210 × 270.

The falling piece is stored separately from the sand grid. Its shape is a list of cell offsets, plus its width, height, position, and colour. When checking whether it can move or land, each piece cell is expanded to its 15 × 15 area and checked against the sand grid. When it lands, those areas are written into the grid as particles.

The grid stores each particle as either empty or a colour. On each sand update, the simulation scans from the bottom row upward. A particle moves down if the space below it is empty; if it is blocked, it tries a diagonal space. The order of the diagonal checks is randomized so particles don’t always favour the same side. Scanning bottom to top helps stop a particle from moving more than one row in a single update.

## connection clearing

The clearing rule works on connected particles, not just full Tetris rows. For each unvisited particle, the game uses a flood fill to find the group of particles with the same colour connected to it. The search checks all 8 neighbours, so diagonal connections count too.

If a group reaches both the left and right edges, the whole group is removed. The game counts a cleared row when that group fully spans the particle grid within a logical 15-particle-high row. It checks for connections after a piece locks and during the regular sand updates, since moving sand can create a connection too.

## update and rendering

The game loop handles input, piece gravity, landing, and the sand update. It draws the frame after those state changes so the display uses the latest board state.

The piece falls every 200 ms. The sand simulation updates every 10 ms, and input is checked on a 10 ms poll. The terminal redraws once per loop.

The board is rendered with Braille characters, which each represent 8 dots. The renderer groups particles as needed to fit the current terminal size. When the terminal is under 88 columns wide, the stats and controls move below the board.

## score

The score is calculated from the stats instead of being incremented separately:

```text
score = active seconds × 100
      + blocks placed × 250
      + rows cleared × 1,000
