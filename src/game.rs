use crate::{entity::Entity, solver::Action};
use std::fmt::Debug;

pub struct Game {
    pub humans: Vec<Entity>,
    pub zombies: Vec<Entity>,
    pub ash: Entity,
    pub score: i32,
    pub step: i32,
    pub ks: [i32; 30],
}

impl Game {
    pub fn new(humans: Vec<Entity>, zombies: Vec<Entity>, ash: Entity) -> Game {
        let mut ks: [i32; 30] = [0; 30];
        ks[0] = 1;
        ks[1] = 1;
        for i in 2..30 {
            ks[i] = ks[i - 1] + ks[i - 2];
        }
        ks[0] = 0;
        Game {
            humans,
            zombies,
            ash,
            score: 0,
            step: 0,
            ks,
        }
    }

    pub fn step(&mut self, action: Action) {
        let pair_human_zombie = self.move_zombies();
        self.move_ash(action);
        let killed_zombies = self.kill_zombies();
        self.kill_humans(pair_human_zombie);
        self.update_score(killed_zombies);
        self.step += 1;
    }

    pub fn is_over(&self) -> bool {
        self.humans.iter().all(|human| !human.alive)
            || self.zombies.iter().all(|zombie| !zombie.alive)
    }

    pub fn reset(&mut self) {
        self.humans.iter_mut().for_each(|human| human.reset());
        self.zombies.iter_mut().for_each(|zombie| zombie.reset());
        self.ash.reset();
    }

    fn move_zombies(&mut self) -> Vec<(i32, i32)> {
        // Les zombies se déplacent vers leurs cibles.
        let mut pair_human_zombie: Vec<(i32, i32)> = Vec::new();
        for zombie in self.zombies.iter_mut().filter(|zombie| zombie.alive) {
            let ash_iter = std::iter::once(&self.ash);
            let mut min_dist = std::f64::MAX;
            let mut min_human = None;
            for human in self
                .humans
                .iter()
                .filter(|human| human.alive)
                .chain(ash_iter)
            {
                let dist = zombie.sqdist(human);
                if dist < min_dist {
                    min_dist = dist;
                    min_human = Some(human);
                }
            }

            if let Some(human) = min_human {
                if min_dist < 160000.0 {
                    zombie.position.x = human.position.x;
                    zombie.position.y = human.position.y;
                    pair_human_zombie.push((human.id, zombie.id));
                } else {
                    let dx = human.position.x - zombie.position.x;
                    let dy = human.position.y - zombie.position.y;
                    let dist = min_dist.sqrt();
                    zombie.position.x += (dx / dist * 400.0).floor();
                    zombie.position.y += (dy / dist * 400.0).floor();
                }
            }
        }
        pair_human_zombie
    }

    fn move_ash(&mut self, action: Action) {
        // Ash se déplace vers sa cible.
        self.ash.position = action.to_Point(&self.ash);
    }

    fn kill_zombies(&mut self) -> i32 {
        // Tout zombie se situant dans un rayon de moins de 2000 unités est détruit.
        let mut killed = 0;
        for zombie in self.zombies.iter_mut().filter(|zombie| zombie.alive) {
            if self.ash.sqdist(zombie) < 4000000.0 {
                zombie.alive = false;
                killed += 1;
            }
        }
        killed
    }

    fn kill_humans(&mut self, pair_human_zombie: Vec<(i32, i32)>) {
        // Si un zombie se trouve sur un humain alors il le mange.
        for (human_id, zombie_id) in pair_human_zombie {
            if self.zombies.get(zombie_id as usize).unwrap().alive {
                self.humans.get_mut(human_id as usize).unwrap().alive = false;
            }
        }
    }

    fn update_score(&mut self, killed_zombies: i32) {
        // La valeur d'un zombie tué est égal au nombre d'humains encore en vie au carré et multiplié par 10, sans inclure Ash.
        // Si plusieurs zombies sont détruits pendant un même tour, la valeur du nème zombie tué est multiplié par le (n+2)ème terme de la suite de Fibonacci (1, 2, 3, 5, 8, etc). Vous avez donc tout intérêt à tuer un maximum de zombies dans un même tour !
        let alive_humans = self.humans.iter().filter(|human| human.alive).count() as i32;
        self.score +=
            self.ks.get(killed_zombies as usize).unwrap_or(&832040) * alive_humans.pow(2) * 10;
    }
}

impl Debug for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Game: score: {} - Step: {}", self.score, self.step)?;
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
