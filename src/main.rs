mod entity;
mod game;
mod loader;
mod point;
mod solver;

use std::time::Duration;

use loader::load_state;
use solver::Solver;

fn main() {
    // Budget par tour en ms (1er argument), 100 ms max sur CodinGame (1000 ms au 1er tour).
    let turn_ms: u64 = std::env::args()
        .nth(1)
        .and_then(|arg| arg.parse().ok())
        .unwrap_or(95);

    let mut game = load_state();
    let mut solver = Solver::new(&game);
    eprintln!("{:?}", game);

    while !game.is_over() {
        let budget = Duration::from_millis(if game.turn == 0 { turn_ms * 10 } else { turn_ms });
        let target = solver.get_action(&game, budget);
        game.step(target);
        println!(
            "Turn {:3} -> {:5.0} {:5.0} | score {:8} | expected {:8} | humans {:2} zombies {:2} | sims {}",
            game.turn,
            target.x,
            target.y,
            game.score,
            solver.best_score(),
            game.humans.len(),
            game.zombies.len(),
            solver.simulations,
        );
    }

    println!("Final score: {}", game.final_score());
}
