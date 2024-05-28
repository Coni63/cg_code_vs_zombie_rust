use crate::{
    entity::{self, Entity},
    point::Point,
};

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

    pub fn to_Point(&self, entity: &Entity) -> Point {
        Point::new(
            entity.position.x + self.radius * self.angle.cos(),
            entity.position.y + self.radius * self.angle.sin(),
        )
    }
}
