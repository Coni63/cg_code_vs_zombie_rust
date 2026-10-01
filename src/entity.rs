use std::fmt::Debug;

use crate::point::Point;

#[derive(Clone, Copy)]
pub struct Entity {
    pub id: i32,
    pub position: Point,
}

impl Entity {
    pub fn new(id: i32, x: f64, y: f64) -> Entity {
        Entity {
            id,
            position: Point::new(x, y),
        }
    }
}

impl Debug for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}: {:?}", self.id, self.position)
    }
}
