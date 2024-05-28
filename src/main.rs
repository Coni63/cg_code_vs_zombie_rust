mod entity;
mod game;
mod loader;
mod point;
mod solver;

use game::Game;
use loader::load_state;
use solver::{Action, Solver};

fn main() {
    let mut game = load_state();
    println!("{:?}", game);

    loop {
        let solver = Solver::new(game.clone());
        let action = solver.get_action();
        game.step(&action);

        println!("Action taken: {:?}", action.to_point(&game.ash));

        if game.is_over() {
            break;
        }
        println!("{:?}", game);
    }
}
