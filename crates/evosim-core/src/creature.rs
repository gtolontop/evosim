use glam::Vec2;

use crate::constants::{CONSTRAINT_ITERS, GRIP_DIST, MASS_PENALTY, START_HEIGHT, WALL_X};
use crate::muscle::Muscle;
use crate::particle::Particle;

/// A soft-body creature built from particles connected by muscles.
///
/// Each entry in `grip_phases` is `(particle_index, phase_offset)`.
/// The grip signal is clock-based — independent of muscle activation —
/// so a creature can simultaneously grip and contract to pull itself up.
#[derive(Debug, Clone)]
pub struct Creature {
    pub particles: Vec<Particle>,
    pub muscles: Vec<Muscle>,
    pub clock: f32,
    pub clock_speed: f32,
    pub fitness: f32,
    pub total_energy_spent: f32,
    /// Highest centre-of-mass Y ever reached while gripping the wall.
    pub max_height: f32,
    /// `(particle_index, grip_phase_offset)` for each limb endpoint.
    pub grip_phases: Vec<(usize, f32)>,
    /// Cached sum of all particle masses (set once at construction).
    pub total_mass: f32,
}

impl Creature {
    pub fn new(particles: Vec<Particle>, muscles: Vec<Muscle>) -> Self {
        let total_mass = particles.iter().map(|p| p.mass).sum();
        Self {
            particles,
            muscles,
            clock: 0.0,
            clock_speed: 1.0,
            fitness: 0.0,
            total_energy_spent: 0.0,
            max_height: 0.0,
            grip_phases: Vec::new(),
            total_mass,
        }
    }

    /// Advances the creature by one simulation step.
    pub fn step(&mut self, dt: f32) {
        use std::f32::consts::TAU;

        // 1. Clock
        self.clock = (self.clock + self.clock_speed * dt * TAU) % TAU;

        // 2. Muscle activations — precompute sin/cos of clock once,
        //    then use angle-addition formula: sin(clock+phase) = sin_c*cos_p + cos_c*sin_p
        let (sin_c, cos_c) = self.clock.sin_cos();
        for muscle in &mut self.muscles {
            muscle.update_activation_fast(sin_c, cos_c);
        }

        // 3. Verlet integration
        for particle in &mut self.particles {
            particle.integrate(dt);
        }

        // 4. Constraint resolution
        let mut step_energy = 0.0;
        for _ in 0..CONSTRAINT_ITERS {
            for muscle in &self.muscles {
                step_energy += muscle.resolve(&mut self.particles);
            }
        }

        // 5. Ground collision
        for particle in &mut self.particles {
            particle.resolve_ground();
        }

        // 6. Wall grips (clock-based, independent of muscles)
        self.update_grips();

        // 7. Fitness
        self.update_fitness(step_energy);
    }

    /// Mass-weighted centre of mass.
    pub fn center_of_mass(&self) -> Vec2 {
        let mut total_mass = 0.0_f32;
        let mut weighted_sum = Vec2::ZERO;
        for p in &self.particles {
            weighted_sum += p.pos * p.mass;
            total_mass += p.mass;
        }
        if total_mass > 0.0 { weighted_sum / total_mass } else { Vec2::ZERO }
    }

    /// Clock-based grip: each endpoint has its own phase offset.
    ///
    /// `grip_signal = sin(clock + phase) * 0.5 + 0.5`
    ///
    /// - signal > 0.5  → grip if near wall
    /// - signal ≤ 0.5  → release
    ///
    /// This is independent of muscle activation, so the creature can
    /// pull itself while gripping (exactly like codebh's internal clock).
    fn update_grips(&mut self) {
        for &(p_idx, phase) in &self.grip_phases {
            let grip_signal = (self.clock + phase).sin() * 0.5 + 0.5;
            let near_wall = self.particles[p_idx].pos.x <= WALL_X + GRIP_DIST;
            let should_grip = near_wall && grip_signal > 0.5;

            if should_grip && !self.particles[p_idx].pinned {
                // Just gripped — zero velocity, snap to wall.
                self.particles[p_idx].pos.x = WALL_X;
                self.particles[p_idx].prev_pos = self.particles[p_idx].pos;
            }
            if !should_grip && self.particles[p_idx].pinned {
                // Just released — small nudge away so it doesn't re-stick instantly.
                self.particles[p_idx].pos.x = WALL_X + GRIP_DIST + 0.02;
                self.particles[p_idx].prev_pos = self.particles[p_idx].pos;
            }

            self.particles[p_idx].pinned = should_grip;
        }
    }

    /// Fitness — only height ABOVE spawn counts, and only while gripping.
    ///
    /// Staying at spawn height = 0 fitness.  Must climb to score.
    pub fn update_fitness(&mut self, energy_cost: f32) {
        self.total_energy_spent += energy_cost;
        let com = self.center_of_mass();

        let has_grip = self.grip_phases.iter().any(|&(i, _)| self.particles[i].pinned);
        if has_grip && com.y > self.max_height {
            self.max_height = com.y;
        }

        let climbed = (self.max_height - START_HEIGHT).max(0.0);
        let wall_dist = com.x.max(0.0);
        self.fitness = climbed * 2.0
            - wall_dist * 5.0
            - self.total_energy_spent * 0.001
            - self.total_mass * MASS_PENALTY;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn step_does_not_panic_on_minimal_creature() {
        let particles = vec![
            Particle::new(0.0, 1.0, 1.0),
            Particle::new(1.0, 1.0, 1.0),
        ];
        let muscles = vec![Muscle::bone(0, 1, 1.0)];
        let mut creature = Creature::new(particles, muscles);
        for _ in 0..100 {
            creature.step(1.0 / 60.0);
        }
        for p in &creature.particles {
            assert!(p.pos.x.is_finite());
            assert!(p.pos.y.is_finite());
        }
    }

    #[test]
    fn center_of_mass_is_correct() {
        let particles = vec![
            Particle::new(0.0, 0.0, 1.0),
            Particle::new(4.0, 0.0, 1.0),
        ];
        let creature = Creature::new(particles, vec![]);
        let com = creature.center_of_mass();
        assert!((com.x - 2.0).abs() < 1e-5);
    }

    #[test]
    fn max_height_tracks_peak_when_gripping() {
        let particles = vec![
            Particle::new(1.0, 5.0, 1.0),
            Particle::new(0.0, 5.0, 0.2), // on wall
        ];
        let muscles = vec![Muscle::bone(0, 1, 1.0)];
        let mut creature = Creature::new(particles, muscles);
        // grip_phase = PI/2 → sin(0 + PI/2) = 1.0 → signal = 1.0 → grips
        creature.grip_phases = vec![(1, std::f32::consts::FRAC_PI_2)];
        creature.particles[1].pinned = true;
        creature.step(1.0 / 60.0);
        assert!(creature.max_height > 0.0, "should track height when gripping");
    }

    #[test]
    fn grip_pins_particle_near_wall() {
        let particles = vec![
            Particle::new(1.0, 2.0, 1.0),
            Particle::new(0.05, 2.0, 0.2),
        ];
        let muscles = vec![Muscle::bone(0, 1, 1.0)];
        let mut creature = Creature::new(particles, muscles);
        // phase = PI/2 → grip_signal starts high → should grip
        creature.grip_phases = vec![(1, std::f32::consts::FRAC_PI_2)];
        creature.step(1.0 / 60.0);
        assert!(creature.particles[1].pinned, "particle near wall should be pinned");
    }
}
