use glam::Vec2;

use crate::constants::{
    CONSTRAINT_ITERS, ENERGY_PENALTY, GRIP_DIST, IMPACT_PENALTY, MASS_PENALTY,
    MUSCLE_MASS_PENALTY, SELF_COLLISION_RADIUS, START_HEIGHT, WALL_X,
};
use crate::muscle::Muscle;
use crate::particle::Particle;

/// Number of keyframes in the movement cycle.
pub const NUM_KEYFRAMES: usize = 4;
/// Number of muscle groups (symmetric L/R).
pub const NUM_MUSCLE_GROUPS: usize = 8;

/// A soft-body creature built from particles connected by muscles.
///
/// Control: an internal clock cycles through `NUM_KEYFRAMES` keyframes.
/// Each keyframe specifies contraction ratios for `NUM_MUSCLE_GROUPS` muscle groups.
/// Left and right sides are offset by `lr_offset` for alternating climbing movement.
#[derive(Debug, Clone)]
pub struct Creature {
    pub particles: Vec<Particle>,
    pub muscles: Vec<Muscle>,
    pub clock: f32,
    pub clock_speed: f32,
    pub fitness: f32,
    pub total_energy_spent: f32,
    pub max_height: f32,
    /// `(particle_index, grip_phase_offset)` for each limb endpoint.
    pub grip_phases: Vec<(usize, f32)>,
    pub total_impact: f32,
    /// Sum of all particle masses (bone + muscle contributions).
    pub total_mass: f32,
    /// Sum of all active muscle masses.
    pub total_muscle_mass: f32,
    /// 4 keyframes × 8 contraction ratios per muscle group.
    pub keyframes: [[f32; NUM_MUSCLE_GROUPS]; NUM_KEYFRAMES],
    /// Phase offset between left and right sides.
    pub lr_offset: f32,
}

impl Creature {
    pub fn new(particles: Vec<Particle>, muscles: Vec<Muscle>) -> Self {
        let total_mass = particles.iter().map(|p| p.mass).sum();
        let total_muscle_mass = muscles.iter().map(|m| m.muscle_mass()).sum();
        Self {
            particles,
            muscles,
            clock: 0.0,
            clock_speed: 1.0,
            fitness: 0.0,
            total_energy_spent: 0.0,
            total_impact: 0.0,
            max_height: 0.0,
            grip_phases: Vec::new(),
            total_mass,
            total_muscle_mass,
            keyframes: [[0.0; NUM_MUSCLE_GROUPS]; NUM_KEYFRAMES],
            lr_offset: 0.0,
        }
    }

    /// Advances the creature by one simulation step.
    pub fn step(&mut self, dt: f32) {
        use std::f32::consts::TAU;

        // 1. Clock
        self.clock = (self.clock + self.clock_speed * dt * TAU) % TAU;

        // 2. Keyframe interpolation → muscle contractions
        self.update_muscle_contractions();

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

        // 4b. Self-collision
        self.resolve_self_collisions();

        // 5. Ground collision
        for particle in &mut self.particles {
            particle.resolve_ground();
        }

        // 6. Wall grips
        self.update_grips();

        // 7. Fitness
        self.update_fitness(step_energy);
    }

    /// Interpolates keyframes and sets muscle contraction ratios.
    fn update_muscle_contractions(&mut self) {
        use std::f32::consts::TAU;

        let quarter = TAU / NUM_KEYFRAMES as f32;

        // Helper: sample the keyframe interpolation at a given phase
        let sample = |phase: f32, keyframes: &[[f32; NUM_MUSCLE_GROUPS]; NUM_KEYFRAMES], group: usize| -> f32 {
            let p = phase % TAU;
            let segment = p / quarter;
            let kf_a = segment.floor() as usize % NUM_KEYFRAMES;
            let kf_b = (kf_a + 1) % NUM_KEYFRAMES;
            let t = segment.fract();
            keyframes[kf_a][group] + (keyframes[kf_b][group] - keyframes[kf_a][group]) * t
        };

        let clock = self.clock;
        let lr_offset = self.lr_offset;

        for muscle in &mut self.muscles {
            if muscle.is_bone {
                continue;
            }
            let group = muscle.group as usize;
            if group >= NUM_MUSCLE_GROUPS {
                continue;
            }

            // Determine if this is a right-side muscle (groups are symmetric,
            // but right-side muscles are at indices 27–34 in the spring list,
            // i.e. the second half of active muscles).
            // We detect right-side by checking if group index is the same
            // but the muscle is the "second" one for that group.
            // Simpler: right-side muscles connect to even-numbered limb particles
            // (shoulder_R=4, elbow_R=6, hand_R=8, hip_R=10, knee_R=12, foot_R=14).
            let is_right = muscle.a == 0 && (muscle.b == 6)        // deltoid_R
                || muscle.a == 1 && (muscle.b == 6)                // dorsal_R
                || muscle.a == 4 && (muscle.b == 8)                // bicep_R
                || muscle.a == 1 && (muscle.b == 8)                // tricep_R
                || muscle.a == 1 && (muscle.b == 12)               // hip_flexor_R
                || muscle.a == 2 && (muscle.b == 12)               // glute_R
                || muscle.a == 10 && (muscle.b == 14)              // quad_R
                || muscle.a == 2 && (muscle.b == 14);              // hamstring_R

            let phase = if is_right { clock + lr_offset } else { clock };
            let contraction = sample(phase, &self.keyframes, group);
            muscle.set_contraction(contraction);
        }
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

    /// Repulse non-connected particle pairs closer than SELF_COLLISION_RADIUS.
    fn resolve_self_collisions(&mut self) {
        let n = self.particles.len();
        for i in 0..n {
            for j in (i + 1)..n {
                if self.particles[i].pinned && self.particles[j].pinned {
                    continue;
                }
                let connected = self.muscles.iter().any(|m| {
                    (m.a == i && m.b == j) || (m.a == j && m.b == i)
                });
                if connected {
                    continue;
                }

                let delta = self.particles[j].pos - self.particles[i].pos;
                let dist_sq = delta.length_squared();
                let min_dist = SELF_COLLISION_RADIUS;
                if dist_sq < min_dist * min_dist && dist_sq > 1e-8 {
                    let dist = dist_sq.sqrt();
                    let overlap = min_dist - dist;
                    let dir = delta / dist;
                    let mass_i = self.particles[i].mass;
                    let mass_j = self.particles[j].mass;
                    let total = mass_i + mass_j;
                    if !self.particles[i].pinned {
                        self.particles[i].pos -= dir * overlap * (mass_j / total);
                    }
                    if !self.particles[j].pinned {
                        self.particles[j].pos += dir * overlap * (mass_i / total);
                    }
                }
            }
        }
    }

    /// Clock-based grip: each endpoint has its own phase offset.
    fn update_grips(&mut self) {
        for &(p_idx, phase) in &self.grip_phases {
            let grip_signal = (self.clock + phase).sin() * 0.5 + 0.5;
            let near_wall = self.particles[p_idx].pos.x <= WALL_X + GRIP_DIST;
            let should_grip = near_wall && grip_signal > 0.5;

            if should_grip && !self.particles[p_idx].pinned {
                let vel = self.particles[p_idx].pos - self.particles[p_idx].prev_pos;
                self.total_impact += vel.length_squared();
                self.particles[p_idx].pos.x = WALL_X;
                self.particles[p_idx].prev_pos = self.particles[p_idx].pos;
            }
            if !should_grip && self.particles[p_idx].pinned {
                self.particles[p_idx].pos.x = WALL_X + GRIP_DIST + 0.02;
                self.particles[p_idx].prev_pos = self.particles[p_idx].pos;
            }

            self.particles[p_idx].pinned = should_grip;
        }
    }

    /// Fitness = Climb − K1·Energy − K2·Mass − K3·Impact − K4·MuscleMass
    pub fn update_fitness(&mut self, energy_cost: f32) {
        self.total_energy_spent += energy_cost;
        let com = self.center_of_mass();

        let grip_count = self.grip_phases.iter()
            .filter(|&&(i, _)| self.particles[i].pinned)
            .count();
        let has_grip = grip_count > 0;

        // Spine orientation check (particle 0 = neck, 2 = pelvis).
        let upright = if self.particles.len() >= 3 {
            self.particles[0].pos.y - self.particles[2].pos.y > 0.3
        } else {
            true
        };

        let min_grips = self.grip_phases.len().min(2);
        if has_grip && upright && grip_count >= min_grips && com.y > self.max_height {
            self.max_height = com.y;
        }

        let climbed = (self.max_height - START_HEIGHT).max(0.0);
        self.fitness = climbed
            - self.total_energy_spent * ENERGY_PENALTY
            - self.total_mass * MASS_PENALTY
            - self.total_impact * IMPACT_PENALTY
            - self.total_muscle_mass * MUSCLE_MASS_PENALTY;
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
    fn muscle_mass_tracked() {
        let particles = vec![
            Particle::new(0.0, 0.0, 1.0),
            Particle::new(1.0, 0.0, 1.0),
        ];
        let muscles = vec![Muscle::muscle(0, 1, 1.0, 0.5, 0)];
        let creature = Creature::new(particles, muscles);
        assert!(creature.total_muscle_mass > 0.0);
    }
}
