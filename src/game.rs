use crate::point::Point;

pub struct Game {
    pub humans: Vec<Point>,
    pub zombies: Vec<Point>,
    pub ash: Point,
    pub score: i32,
}
