use std::time::{Duration, Instant};

use crate::{
    game::{Game, ASH_SPEED, HEIGHT, WIDTH},
    point::Point,
};

// Garde-fou : une partie se termine toujours bien avant (Ash finit par rattraper les zombies).
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

#[derive(Clone, Copy)]
enum Kind {
    /// Rejoue le meilleur plan puis le complète par la politique de chasse.
    Replay,
    /// Quelques coups aléatoires puis chasse.
    Fresh,
    /// Préfixe du meilleur plan, quelques coups aléatoires, puis chasse.
    Prefix,
    /// Meilleur plan dont quelques coups sont remplacés aléatoirement.
    Perturb,
}

/// Monte Carlo sur parties complètes : chaque simulation joue jusqu'à la fin de la partie
/// et le meilleur plan (suite de cibles d'Ash) est conservé d'un tour à l'autre.
pub struct Solver {
    rng: Rng,
    sim: Game,
    plan: Vec<Point>,
    best_plan: Vec<Point>,
    best_score: i64,
    /// Le rollout courant ne peut plus battre `best_score`.
    pruned: bool,
    pub simulations: u64,
}

impl Solver {
    pub fn new(game: &Game) -> Solver {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x9E37_79B9_7F4A_7C15)
            | 1;
        Solver {
            rng: Rng(seed),
            sim: game.clone(),
            plan: Vec::new(),
            best_plan: Vec::new(),
            best_score: -1,
            pruned: false,
            simulations: 0,
        }
    }

    pub fn best_score(&self) -> i64 {
        self.best_score
    }

    pub fn get_action(&mut self, root: &Game, budget: Duration) -> Point {
        let start = Instant::now();

        // Le plan précédent est ré-évalué depuis l'état courant (identique si la simulation est exacte).
        self.best_score = -1;
        self.try_rollout(root, Kind::Replay);

        while start.elapsed() < budget {
            let kind = match self.rng.below(10) {
                0..=4 => Kind::Fresh,
                5..=7 => Kind::Prefix,
                _ => Kind::Perturb,
            };
            self.try_rollout(root, kind);
        }

        if self.best_plan.is_empty() {
            return root.ash.position;
        }
        self.best_plan.remove(0)
    }

    fn try_rollout(&mut self, root: &Game, kind: Kind) {
        let score = self.rollout(root, kind);
        self.simulations += 1;
        if score > self.best_score {
            self.best_score = score;
            std::mem::swap(&mut self.plan, &mut self.best_plan);
        }
    }

    fn rollout(&mut self, root: &Game, kind: Kind) -> i64 {
        self.sim.copy_from(root);
        self.plan.clear();
        self.pruned = false;

        let best_len = self.best_plan.len();
        let (prefix_len, segments, perturb_rate) = match kind {
            Kind::Replay => (best_len, 0, 0.0),
            Kind::Fresh => (0, self.rng.below(7), 0.0),
            Kind::Prefix => (self.rng.below(best_len + 1), 1 + self.rng.below(2), 0.0),
            Kind::Perturb => (best_len, 0, 2.0 / (best_len.max(1) as f64)),
        };

        for i in 0..prefix_len {
            if self.done() {
                break;
            }
            let target = if perturb_rate > 0.0 && self.rng.unit() < perturb_rate {
                if self.rng.below(2) == 0 {
                    self.random_target()
                } else {
                    self.jitter(self.best_plan[i])
                }
            } else {
                self.best_plan[i]
            };
            self.play(target);
        }

        for _ in 0..segments {
            self.play_segment();
        }

        // Politique de chasse : on poursuit un zombie (aléatoire ou le plus proche) jusqu'à sa mort.
        let mut prey: Option<i32> = None;
        while !self.done() && self.sim.turn - root.turn < MAX_ROLLOUT_TURNS {
            let zombie = match prey.and_then(|id| self.sim.zombies.iter().find(|z| z.id == id)) {
                Some(z) => z.position,
                None => {
                    let idx = if self.rng.below(4) == 0 {
                        self.nearest_zombie()
                    } else {
                        self.rng.below(self.sim.zombies.len())
                    };
                    prey = Some(self.sim.zombies[idx].id);
                    self.sim.zombies[idx].position
                }
            };
            // On vise la position du zombie au prochain tour.
            let target = self.sim.zombie_next(&zombie).0;
            self.play(target);
        }

        if self.pruned {
            return -1;
        }
        self.sim.final_score()
    }

    fn done(&self) -> bool {
        self.pruned || self.sim.is_over()
    }

    fn play(&mut self, target: Point) {
        self.plan.push(target);
        self.sim.step(target);
        self.pruned = self.sim.score_upper_bound() <= self.best_score;
    }

    /// Garde une même intention pendant 1 à 8 tours.
    fn play_segment(&mut self) {
        if self.done() {
            return;
        }
        let duration = 1 + self.rng.below(12);
        let kind = self.rng.below(6);
        let fixed = match kind {
            0 => self.sim.ash.position,
            1 | 2 => self.random_target(),
            5 => self.sim.humans[self.rng.below(self.sim.humans.len())].position,
            _ => Point::new(0.0, 0.0),
        };
        for _ in 0..duration {
            if self.done() {
                return;
            }
            let ash = self.sim.ash.position;
            let target = match kind {
                // Fuir le centre des zombies : ils se regroupent derrière Ash.
                3 => {
                    let c = self.zombie_centroid();
                    let (dx, dy) = (ash.x - c.x, ash.y - c.y);
                    let d = (dx * dx + dy * dy).sqrt().max(1.0);
                    clamp_to_board(ash.x + dx / d * ASH_SPEED, ash.y + dy / d * ASH_SPEED)
                }
                4 => self.zombie_centroid(),
                _ => fixed,
            };
            self.play(target);
        }
    }

    fn zombie_centroid(&self) -> Point {
        let n = self.sim.zombies.len() as f64;
        let (sx, sy) = self
            .sim
            .zombies
            .iter()
            .fold((0.0, 0.0), |(sx, sy), z| (sx + z.position.x, sy + z.position.y));
        Point::new((sx / n).floor(), (sy / n).floor())
    }

    fn nearest_zombie(&self) -> usize {
        let ash = &self.sim.ash.position;
        let mut best = 0;
        let mut best_dist = f64::MAX;
        for (i, z) in self.sim.zombies.iter().enumerate() {
            let d = z.position.sqdist(ash);
            if d < best_dist {
                best_dist = d;
                best = i;
            }
        }
        best
    }

    /// Décale légèrement une cible du plan.
    fn jitter(&mut self, target: Point) -> Point {
        let radius = [100.0, 400.0, 1500.0][self.rng.below(3)];
        clamp_to_board(
            target.x + (self.rng.unit() * 2.0 - 1.0) * radius,
            target.y + (self.rng.unit() * 2.0 - 1.0) * radius,
        )
    }

    fn random_target(&mut self) -> Point {
        let ash = self.sim.ash.position;
        match self.rng.below(3) {
            // Rester sur place : laisse les zombies se regrouper pour un combo.
            0 => ash,
            // Pas complet dans une direction aléatoire.
            1 => {
                let angle = self.rng.unit() * std::f64::consts::TAU;
                clamp_to_board(ash.x + ASH_SPEED * angle.cos(), ash.y + ASH_SPEED * angle.sin())
            }
            // Point aléatoire du terrain.
            _ => Point::new(
                self.rng.below(WIDTH as usize) as f64,
                self.rng.below(HEIGHT as usize) as f64,
            ),
        }
    }
}

/// Cible entière à l'intérieur du terrain.
fn clamp_to_board(x: f64, y: f64) -> Point {
    Point::new(x.floor().clamp(0.0, WIDTH - 1.0), y.floor().clamp(0.0, HEIGHT - 1.0))
}
