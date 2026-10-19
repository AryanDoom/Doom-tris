mod board;
mod game;
mod scoreboard;
mod ui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    game::run()
}

//this used to be 400 lines 
//now it is 5 files and like 500 lines but dsa maam said modularity is good