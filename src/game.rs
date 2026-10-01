use crate::{entity::Entity, point::Point};
use std::fmt::Debug;

pub const WIDTH: f64 = 16000.0;
pub const HEIGHT: f64 = 9000.0;
pub const ASH_SPEED: f64 = 1000.0;
const ZOMBIE_SPEED: f64 = 400.0;
const ASH_RANGE_SQ: f64 = 2000.0 * 2000.0;
const MAX_ENTITIES: usize = 128;
const NO_VICTIM: usize = usize::MAX;

// COMBO[k] = somme des multiplicateurs quand k zombies meurent dans le même tour (1, 2, 3, 5, 8, ...)
const COMBO: [i64; MAX_ENTITIES] = combo_table();

const fn combo_table() -> [i64; MAX_ENTITIES] {
    let mut table = [0i64; MAX_ENTITIES];
    let (mut a, mut b) = (1i64, 2i64);
    let mut k = 1;
    while k < MAX_ENTITIES {
        table[k] = table[k - 1].saturating_add(a);
        let c = a.saturating_add(b);
        a = b;
        b = c;
        k += 1;
    }
    table
}

/// Les entités mortes sont retirées des vecteurs : un humain/zombie présent est vivant.
#[derive(Clone)]
pub struct Game {
    pub humans: Vec<Entity>,
    pub zombies: Vec<Entity>,
    pub ash: Entity,
    pub score: i64,
    pub turn: i32,
}

impl Game {
    pub fn new(humans: Vec<Entity>, zombies: Vec<Entity>, ash: Entity) -> Game {
        Game {
            humans,
            zombies,
            ash,
            score: 0,
            turn: 0,
        }
    }

    /// Copie `other` dans `self` en réutilisant les allocations.
    pub fn copy_from(&mut self, other: &Game) {
        self.humans.clear();
        self.humans.extend_from_slice(&other.humans);
        self.zombies.clear();
        self.zombies.extend_from_slice(&other.zombies);
        self.ash = other.ash;
        self.score = other.score;
        self.turn = other.turn;
    }

    pub fn step(&mut self, target: Point) {
        // 1. Les zombies se déplacent vers l'humain le plus proche (Ash inclus).
        let n = self.zombies.len();
        let mut victims = [NO_VICTIM; MAX_ENTITIES];
        for (i, victim) in victims.iter_mut().enumerate().take(n) {
            let (position, target) = self.zombie_next(&self.zombies[i].position);
            self.zombies[i].position = position;
            *victim = target;
        }

        // 2. Ash se déplace vers sa cible.
        self.ash.position = self.ash.position.move_toward(&target, ASH_SPEED);

        // 3. Ash détruit les zombies à <= 2000 unités.
        let mut kept = 0;
        for i in 0..n {
            if self.zombies[i].position.sqdist(&self.ash.position) > ASH_RANGE_SQ {
                self.zombies[kept] = self.zombies[i];
                victims[kept] = victims[i];
                kept += 1;
            }
        }
        let killed = n - kept;
        self.zombies.truncate(kept);
        if killed > 0 {
            let h = self.humans.len() as i64;
            self.score = self
                .score
                .saturating_add((10 * h * h).saturating_mul(COMBO[killed]));
        }

        // 4. Les zombies survivants mangent les humains atteints.
        let mut eaten: u128 = 0;
        for &victim in &victims[..kept] {
            if victim != NO_VICTIM {
                eaten |= 1 << victim;
            }
        }
        if eaten != 0 {
            let mut idx = 0;
            self.humans.retain(|_| {
                let keep = (eaten >> idx) & 1 == 0;
                idx += 1;
                keep
            });
        }

        self.turn += 1;
    }

    /// Position d'un zombie après son déplacement, et l'index de l'humain qu'il atteint
    /// (NO_VICTIM s'il n'atteint personne ou s'il atteint Ash).
    pub fn zombie_next(&self, zombie: &Point) -> (Point, usize) {
        let mut victim = NO_VICTIM;
        let mut target = self.ash.position;
        let mut min_dist = f64::MAX;
        for (j, human) in self.humans.iter().enumerate() {
            let d = zombie.sqdist(&human.position);
            if d < min_dist {
                min_dist = d;
                victim = j;
                target = human.position;
            }
        }
        let d = zombie.sqdist(&self.ash.position);
        if d < min_dist {
            min_dist = d;
            victim = NO_VICTIM;
            target = self.ash.position;
        }

        if min_dist < ZOMBIE_SPEED * ZOMBIE_SPEED {
            (target, victim)
        } else {
            (zombie.move_toward(&target, ZOMBIE_SPEED), NO_VICTIM)
        }
    }

    pub fn is_over(&self) -> bool {
        self.humans.is_empty() || self.zombies.is_empty()
    }

    pub fn final_score(&self) -> i64 {
        if self.humans.is_empty() {
            0
        } else {
            self.score
        }
    }
}

impl Debug for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Game: score: {} - Turn: {}", self.score, self.turn)?;
        writeln!(f, "Ash: ")?;
        writeln!(f, "    {:?}", self.ash)?;

        writeln!(f, "Humans: ")?;
        for human in &self.humans {
            writeln!(f, "    {:?}", human)?;
        }

        writeln!(f, "Zombies: ")?;
        for zombie in &self.zombies {
            writeln!(f, "    {:?}", zombie)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_clones() {
        let mut game = Game::new(
            vec![Entity::new(0, 1500.0, 1500.0)],
            vec![Entity::new(0, 5000.0, 5000.0)],
            Entity::new(0, 0.0, 0.0),
        );

        let copy = game.clone();

        game.step(Point::new(0.0, 0.0));

        assert_eq!(game.zombies[0].position.x, 4717.0);
        assert_eq!(copy.zombies[0].position.x, 5000.0);
    }

    #[test]
    fn it_scores_combos() {
        assert_eq!(&COMBO[..5], &[0, 1, 3, 6, 11]);

        let mut game = Game::new(
            vec![Entity::new(0, 8000.0, 8000.0), Entity::new(1, 9000.0, 8000.0)],
            vec![Entity::new(0, 1000.0, 1000.0), Entity::new(1, 1500.0, 1000.0)],
            Entity::new(0, 0.0, 0.0),
        );
        game.step(Point::new(0.0, 0.0));
        assert!(game.zombies.is_empty());
        assert_eq!(game.final_score(), 10 * 4 * 3);
    }

    #[test]
    fn zombie_reaching_ash_does_not_kill_a_human() {
        let mut game = Game::new(
            vec![Entity::new(0, 15000.0, 8000.0)],
            vec![Entity::new(0, 5300.0, 5000.0)],
            Entity::new(0, 5000.0, 5000.0),
        );
        game.step(Point::new(0.0, 0.0));
        assert_eq!(game.humans.len(), 1);
    }
}

#[cfg(test)]
mod testcases {
    use super::*;

    #[test]
    fn zombie_moves_match_testcases() {
        for entry in std::fs::read_dir("testcases").unwrap() {
            let path = entry.unwrap().path();
            let text = std::fs::read_to_string(&path).unwrap();
            let mut lines = text.lines().map(|l| {
                l.split_whitespace()
                    .map(|v| v.parse::<f64>().unwrap())
                    .collect::<Vec<_>>()
            });
            let a = lines.next().unwrap();
            let ash = Entity::new(0, a[0], a[1]);
            let humans: Vec<Entity> = (0..lines.next().unwrap()[0] as usize)
                .map(|_| {
                    let h = lines.next().unwrap();
                    Entity::new(h[0] as i32, h[1], h[2])
                })
                .collect();
            let zombies: Vec<Vec<f64>> = (0..lines.next().unwrap()[0] as usize)
                .map(|_| lines.next().unwrap())
                .collect();
            let game = Game::new(
                humans,
                zombies.iter().map(|z| Entity::new(z[0] as i32, z[1], z[2])).collect(),
                ash,
            );
            for z in &zombies {
                let (next, _) = game.zombie_next(&Point::new(z[1], z[2]));
                assert_eq!((next.x, next.y), (z[3], z[4]), "{:?} zombie {}", path, z[0]);
            }
        }
    }
}
