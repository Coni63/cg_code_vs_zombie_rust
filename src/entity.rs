use std::fmt::Debug;

use crate::point::Point;

pub struct Entity {
    pub id: i32,
    pub position: Point,
    pub start_position: Point,
    pub alive: bool,
}

impl Entity {
    pub fn new(id: i32, x: f64, y: f64) -> Entity {
        Entity {
            id,
            position: Point::new(x, y),
            start_position: Point::new(x, y),
            alive: true,
        }
    }

    pub fn sqdist(&self, other: &Entity) -> f64 {
        self.position.sqdist(&other.position)
    }

    pub fn reset(&mut self) {
        self.position.x = self.start_position.x;
        self.position.y = self.start_position.y;
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

impl Clone for Entity {
    fn clone(&self) -> Self {
        Entity {
            id: self.id,
            position: self.position.clone(),
            start_position: self.start_position.clone(),
            alive: self.alive,
        }
    }
}
