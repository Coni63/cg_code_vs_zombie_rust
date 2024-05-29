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

pub struct Solver {
    pub game: Game,
}

impl Solver {
    pub fn new(game: Game) -> Solver {
        Solver { game }
    }

    pub fn get_action(&mut self) -> Action {
        self.game.reset();
        Action::new(0.0, 0.0)
    }
}
