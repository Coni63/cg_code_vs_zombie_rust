use std::fmt::Debug;

use crate::point::Point;

pub struct Entity {
    pub id: i32,
    pub position: Point,
    pub startPosition: Point,
    pub alive: bool,
}

impl Entity {
    pub fn new(id: i32, x: f64, y: f64) -> Entity {
        Entity {
            id,
            position: Point::new(x, y),
            startPosition: Point::new(x, y),
            alive: true,
        }
    }

    pub fn dist(&self, other: &Entity) -> f64 {
        self.position.dist(&other.position)
    }

    pub fn sqdist(&self, other: &Entity) -> f64 {
        self.position.sqdist(&other.position)
    }

    pub fn reset(&mut self) {
        self.position.x = self.startPosition.x;
        self.position.y = self.startPosition.y;
    }
}

impl Debug for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "#{}: {:?} - alive: {}",
            self.id, self.position, self.alive
        )
    }
}
