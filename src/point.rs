use std::fmt::Debug;

#[derive(Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Point {
        Point { x, y }
    }

    pub fn sqdist(&self, other: &Point) -> f64 {
        (self.x - other.x).powi(2) + (self.y - other.y).powi(2)
    }

    /// Point reached when moving from `self` toward `target` by at most `speed`,
    /// coordinates rounded down (game rule).
    pub fn move_toward(&self, target: &Point, speed: f64) -> Point {
        let d2 = self.sqdist(target);
        if d2 <= speed * speed {
            return *target;
        }
        let d = d2.sqrt();
        Point::new(
            self.x + ((target.x - self.x) / d * speed).floor(),
            self.y + ((target.y - self.y) / d * speed).floor(),
        )
    }
}

impl Debug for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.0}, {:.0}", self.x, self.y)
    }
}
