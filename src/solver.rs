use rand::Rng;

use crate::{entity::Entity, game::Game, point::Point};

pub struct Action {
    pub radius: f64,
    pub angle: f64, // in radians
}

impl Action {
    pub fn new(radius: f64, angle: f64) -> Action {
        Action {
            radius,
            angle: angle.to_radians(),
        }
    }

    pub fn to_point(&self, entity: &Entity) -> Point {
        Point::new(
            entity.position.x + self.radius * self.angle.cos(),
            entity.position.y + self.radius * self.angle.sin(),
        )
    }
}

impl Clone for Action {
    fn clone(&self) -> Self {
        Action {
            radius: self.radius,
            angle: self.angle,
        }
    }
}

pub struct Solver {
    pub game: Game,
    pub depth: i32,
    pub rng: rand::rngs::ThreadRng,
}

impl Solver {
    pub fn new(game: Game) -> Solver {
        Solver {
            game,
            depth: 4,
            rng: rand::thread_rng(),
        }
    }

    pub fn get_action(&mut self) -> Action {
        let mut actions: Vec<Action> = Vec::new();
        let mut best_actions: Vec<Action> = Vec::new();
        let mut best_score = 0i32;

        for simulation in 0..10000 {
            for _ in 0..self.depth {
                let last_action = self.get_random_action();
                actions.push(last_action.clone());

                self.game.step(&last_action);

                if self.game.is_over() {
                    break;
                }
            }

            let current_score = self.game.score + self.game.possible_score();

            if current_score > best_score
                || (current_score == best_score && actions.len() < best_actions.len())
            {
                eprintln!("New best score: {} @{}", current_score, simulation);
                best_score = current_score;
                best_actions = actions.to_vec();
            }
            self.game.reset();
        }
        best_actions
            .first()
            .unwrap_or(&Action::new(0.0, 0.0))
            .clone()
    }

    fn get_random_action(&mut self) -> Action {
        let radius = self.rng.gen_range(1..=4) as f64 * 500.0;
        let angle = self.rng.gen_range(0..8) as f64 * 45.0;
        Action::new(radius, angle)
    }
}
