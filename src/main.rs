mod entity;
mod game;
mod loader;
mod point;
mod solver;

use game::Game;
use loader::load_state;
use solver::Action;

fn main() {
    let mut game = load_state();
    println!("{:?}", game);

    loop {
        let action = Action::new(0.0, 0.0);
        game.step(action);

        if game.is_over() {
            break;
        }
        println!("{:?}", game);
    }
}
