use crate::{entity::Entity, game::Game};

use std::io::{self, Read};

pub fn load_state() -> Game {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    load_from_str(&input)
}

/// Lit l'état initial : `x y`, les humains `id x y` puis les zombies `id x y xNext yNext`.
pub fn load_from_str(input: &str) -> Game {
    let mut numbers = input
        .split_whitespace()
        .map(|v| v.parse::<f64>().unwrap());
    let mut next = || numbers.next().unwrap();

    let ash = Entity::new(0, next(), next());

    let human_count = next() as usize;
    let humans: Vec<Entity> = (0..human_count)
        .map(|_| Entity::new(next() as i32, next(), next()))
        .collect();

    let zombie_count = next() as usize;
    let zombies: Vec<Entity> = (0..zombie_count)
        .map(|_| {
            let zombie = Entity::new(next() as i32, next(), next());
            next(); // zombieXNext
            next(); // zombieYNext
            zombie
        })
        .collect();

    Game::new(humans, zombies, ash)
}
