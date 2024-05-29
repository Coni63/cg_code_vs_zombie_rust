use crate::{entity::Entity, game::Game};

use std::io;

macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}

pub fn load_state() -> Game {
    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();
    let inputs = input_line.split(' ').collect::<Vec<_>>();
    let x = parse_input!(inputs[0], f64);
    let y = parse_input!(inputs[1], f64);

    let ash = Entity::new(0, x, y);
    let mut humans: Vec<Entity> = Vec::new();
    let mut zombies: Vec<Entity> = Vec::new();

    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();
    let human_count = parse_input!(input_line, i32);
    for _ in 0..human_count as usize {
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let inputs = input_line.split(' ').collect::<Vec<_>>();
        let human_id = parse_input!(inputs[0], i32);
        let human_x = parse_input!(inputs[1], f64);
        let human_y = parse_input!(inputs[2], f64);

        humans.push(Entity::new(human_id, human_x, human_y));
    }
    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();
    let zombie_count = parse_input!(input_line, i32);
    for _ in 0..zombie_count as usize {
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let inputs = input_line.split(' ').collect::<Vec<_>>();
        let zombie_id = parse_input!(inputs[0], i32);
        let zombie_x = parse_input!(inputs[1], f64);
        let zombie_y = parse_input!(inputs[2], f64);

        zombies.push(Entity::new(zombie_id, zombie_x, zombie_y));
    }

    Game::new(humans, zombies, ash)
}
