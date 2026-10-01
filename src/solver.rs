use std::time::{Duration, Instant};

use crate::{
    game::{Game, ASH_SPEED, HEIGHT, WIDTH},
    point::Point,
};

/// Nombre de coups encodés ; au-delà, Ash chasse le zombie le plus proche.
const GENOME_LEN: usize = 100;
// Garde-fou : une partie se termine toujours bien avant.
const MAX_ROLLOUT_TURNS: i32 = 300;

/// xorshift64* : rapide, sans dépendance.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn below(&mut self, n: usize) -> usize {
        (((self.next() >> 32) * n as u64) >> 32) as usize
    }

    fn unit(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1u64 << 53) as f64
    }
}

/// Recuit simulé sur la suite des déplacements relatifs d'Ash (dx, dy), |d| <= 1000.
/// Les états de la solution courante sont gardés en cache à chaque tour : muter le coup i
/// ne re-simule que la fin de la partie depuis l'état i. Les mutations mêlent retouches
/// locales (un coup, un bloc constant, insertion/suppression) et réécriture de toute la fin
/// du plan par un rollout heuristique (segments d'intention puis chasse).
pub struct Solver {
    rng: Rng,
    /// Solution courante du recuit, ses états avant chaque coup, son score.
    genes: Vec<Point>,
    states: Vec<Game>,
    used: usize,
    score: i64,
    candidate: Vec<Point>,
    sim: Game,
    best_genes: Vec<Point>,
    best_score: i64,
    pub simulations: u64,
    /// Nombre de mutations acceptées.
    pub generations: u64,
}

impl Solver {
    pub fn new(game: &Game) -> Solver {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x9E37_79B9_7F4A_7C15)
            | 1;
        let mut solver = Solver {
            rng: Rng(seed),
            genes: Vec::new(),
            states: vec![game.clone(); GENOME_LEN + 1],
            used: 0,
            score: -1,
            candidate: Vec::new(),
            sim: game.clone(),
            best_genes: Vec::new(),
            best_score: -1,
            simulations: 0,
            generations: 0,
        };
        solver.genes = (0..GENOME_LEN).map(|_| solver.random_gene()).collect();
        solver
    }

    pub fn best_score(&self) -> i64 {
        self.best_score
    }

    pub fn get_action(&mut self, root: &Game, budget: Duration) -> Point {
        let start = Instant::now();

        // Repart du meilleur plan connu (déjà avancé d'un coup au tour précédent).
        if !self.best_genes.is_empty() {
            self.genes.clone_from(&self.best_genes);
        }
        self.states[0].copy_from(root);
        let (score, used) = self.replay(0);
        self.score = score;
        self.used = used;
        self.best_score = score;
        self.best_genes.clone_from(&self.genes);

        let (t0, t1) = (0.3f64, 0.005f64);
        let budget_s = budget.as_secs_f64();
        let mut iteration = 0u64;
        let mut temperature = t0;
        loop {
            if iteration & 63 == 0 {
                let progress = start.elapsed().as_secs_f64() / budget_s;
                if progress >= 1.0 {
                    break;
                }
                temperature = t0 * (t1 / t0).powf(progress);
            }
            iteration += 1;
            self.step(temperature);
        }

        let gene = self.best_genes[0];
        self.best_genes.remove(0);
        let filler = self.random_gene();
        self.best_genes.push(filler);
        clamp_to_board(root.ash.position.x + gene.x, root.ash.position.y + gene.y)
    }

    /// Une itération du recuit : mutation, évaluation depuis le cache, acceptation.
    fn step(&mut self, temperature: f64) {
        self.candidate.clone_from(&self.genes);
        let from = self.mutate();
        let (score, _) = self.simulate(from, false);
        self.simulations += 1;

        let delta = ln_score(score) - ln_score(self.score);
        if delta >= 0.0 || self.rng.unit() < (delta / temperature).exp() {
            std::mem::swap(&mut self.genes, &mut self.candidate);
            let (score, used) = self.replay(from);
            self.score = score;
            self.used = used;
            if score > self.best_score {
                self.best_score = score;
                self.best_genes.clone_from(&self.genes);
            }
            self.generations += 1;
        }
    }

    /// Modifie `candidate` et renvoie le premier indice modifié.
    fn mutate(&mut self) -> usize {
        let limit = self.used.max(1);
        let i = self.rng.below(limit);
        if self.rng.below(10) < 4 {
            // Réécrire toute la fin du plan avec un rollout heuristique (depuis le début parfois).
            let i = if self.rng.below(5) == 0 { 0 } else { i };
            self.rollout_tail(i);
            return i;
        }
        match self.rng.below(6) {
            0 => self.candidate[i] = self.random_gene(),
            1 => {
                let g = self.candidate[i];
                let speed = (g.x * g.x + g.y * g.y).sqrt();
                let speed = if speed < 1.0 || self.rng.below(3) == 0 {
                    ASH_SPEED * self.rng.unit()
                } else {
                    speed
                };
                let angle = g.y.atan2(g.x) + (self.rng.unit() - 0.5) * 0.8;
                self.candidate[i] = polar(angle, speed);
            }
            // Bloc constant : tenir une direction (se faire suivre) ou attendre.
            2 | 3 => {
                let gene = if self.rng.below(2) == 0 {
                    self.random_gene()
                } else {
                    self.candidate[i]
                };
                let end = (i + 1 + self.rng.below(12)).min(GENOME_LEN);
                self.candidate[i..end].fill(gene);
            }
            4 => {
                self.candidate.pop();
                let gene = self.random_gene();
                self.candidate.insert(i, gene);
            }
            _ => {
                self.candidate.remove(i);
                let gene = self.random_gene();
                self.candidate.push(gene);
            }
        }
        i
    }

    /// Re-simule la solution courante depuis l'état `from` en mettant le cache à jour.
    fn replay(&mut self, from: usize) -> (i64, usize) {
        std::mem::swap(&mut self.candidate, &mut self.genes);
        let result = self.simulate(from, true);
        std::mem::swap(&mut self.candidate, &mut self.genes);
        result
    }

    /// Simule `candidate` depuis l'état en cache `from` ; renvoie (score, coups joués).
    fn simulate(&mut self, from: usize, record: bool) -> (i64, usize) {
        let start_turn = self.states[0].turn;
        self.sim.copy_from(&self.states[from]);
        let mut used = from;
        while !self.sim.is_over() && used < GENOME_LEN {
            let ash = self.sim.ash.position;
            let g = self.candidate[used];
            self.sim.step(clamp_to_board(ash.x + g.x, ash.y + g.y));
            used += 1;
            if record {
                self.states[used].copy_from(&self.sim);
            }
        }
        while !self.sim.is_over() && self.sim.turn - start_turn < MAX_ROLLOUT_TURNS {
            let zombie = self.sim.zombies[nearest_zombie(&self.sim)].position;
            let target = self.sim.zombie_next(&zombie).0;
            self.sim.step(target);
        }
        (self.sim.final_score(), used)
    }

    /// Réécrit `candidate[from..]` : quelques segments d'intention puis chasse de zombies
    /// choisis au hasard, en enregistrant les déplacements effectifs d'Ash.
    fn rollout_tail(&mut self, from: usize) {
        self.sim.copy_from(&self.states[from]);
        let mut len = from;
        for _ in 0..self.rng.below(5) {
            self.play_segment(&mut len);
        }
        let mut prey: Option<i32> = None;
        while !self.sim.is_over() && len < GENOME_LEN {
            let zombie = match prey.and_then(|id| self.sim.zombies.iter().find(|z| z.id == id)) {
                Some(z) => z.position,
                None => {
                    let idx = if self.rng.below(4) == 0 {
                        nearest_zombie(&self.sim)
                    } else {
                        self.rng.below(self.sim.zombies.len())
                    };
                    prey = Some(self.sim.zombies[idx].id);
                    self.sim.zombies[idx].position
                }
            };
            let target = self.sim.zombie_next(&zombie).0;
            self.record(&mut len, target);
        }
        for i in len..GENOME_LEN {
            self.candidate[i] = self.random_gene();
        }
    }

    /// Garde une même intention pendant 1 à 12 tours.
    fn play_segment(&mut self, len: &mut usize) {
        if self.sim.is_over() {
            return;
        }
        let duration = 1 + self.rng.below(12);
        let kind = self.rng.below(6);
        let fixed = match kind {
            0 => self.sim.ash.position,
            1 | 2 => Point::new(
                self.rng.below(WIDTH as usize) as f64,
                self.rng.below(HEIGHT as usize) as f64,
            ),
            5 => self.sim.humans[self.rng.below(self.sim.humans.len())].position,
            _ => Point::new(0.0, 0.0),
        };
        for _ in 0..duration {
            if self.sim.is_over() || *len >= GENOME_LEN {
                return;
            }
            let ash = self.sim.ash.position;
            let target = match kind {
                // Fuir le centre des zombies : ils se regroupent derrière Ash.
                3 => {
                    let c = zombie_centroid(&self.sim);
                    let (dx, dy) = (ash.x - c.x, ash.y - c.y);
                    let d = (dx * dx + dy * dy).sqrt().max(1.0);
                    clamp_to_board(ash.x + dx / d * ASH_SPEED, ash.y + dy / d * ASH_SPEED)
                }
                4 => zombie_centroid(&self.sim),
                _ => fixed,
            };
            self.record(len, target);
        }
    }

    /// Joue `target` et enregistre le déplacement effectif d'Ash dans `candidate`.
    fn record(&mut self, len: &mut usize, target: Point) {
        let before = self.sim.ash.position;
        self.sim.step(target);
        let after = self.sim.ash.position;
        self.candidate[*len] = Point::new(after.x - before.x, after.y - before.y);
        *len += 1;
    }

    fn random_gene(&mut self) -> Point {
        let angle = self.rng.unit() * std::f64::consts::TAU;
        match self.rng.below(10) {
            0 | 1 => Point::new(0.0, 0.0),
            9 => polar(angle, ASH_SPEED * self.rng.unit()),
            _ => polar(angle, ASH_SPEED),
        }
    }
}

fn nearest_zombie(game: &Game) -> usize {
    let ash = &game.ash.position;
    let mut best = 0;
    let mut best_dist = f64::MAX;
    for (i, z) in game.zombies.iter().enumerate() {
        let d = z.position.sqdist(ash);
        if d < best_dist {
            best_dist = d;
            best = i;
        }
    }
    best
}

fn zombie_centroid(game: &Game) -> Point {
    let n = game.zombies.len() as f64;
    let (sx, sy) = game
        .zombies
        .iter()
        .fold((0.0, 0.0), |(sx, sy), z| (sx + z.position.x, sy + z.position.y));
    Point::new((sx / n).floor(), (sy / n).floor())
}

fn ln_score(score: i64) -> f64 {
    ((score.max(0) + 1) as f64).ln()
}

/// Déplacement entier de norme <= `speed`.
fn polar(angle: f64, speed: f64) -> Point {
    Point::new((speed * angle.cos()).trunc(), (speed * angle.sin()).trunc())
}

/// Cible entière à l'intérieur du terrain.
fn clamp_to_board(x: f64, y: f64) -> Point {
    Point::new(x.floor().clamp(0.0, WIDTH - 1.0), y.floor().clamp(0.0, HEIGHT - 1.0))
}
