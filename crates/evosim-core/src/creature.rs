use glam::Vec2;

use crate::constants::CONSTRAINT_ITERS;
use crate::muscle::Muscle;
use crate::particle::Particle;

/// A soft-body creature built from particles connected by muscles.
///
/// The creature's internal `clock` drives muscle oscillations, producing
/// locomotion. Fitness is evaluated externally (e.g. horizontal distance
/// traveled) and stored here for selection.
#[derive(Debug, Clone)]
pub struct Creature {
    /// The point-mass nodes that make up the creature's body.
    pub particles: Vec<Particle>,
    /// Spring connections between particles.
    pub muscles: Vec<Muscle>,
    /// Internal oscillation clock (advances by `clock_speed` each tick).
    pub clock: f32,
    /// Rate at which the internal clock advances per physics step.
    pub clock_speed: f32,
    /// Fitness score assigned after evaluation (e.g. distance traveled).
    pub fitness: f32,
    /// Cumulative energy spent by muscle activations.
    pub total_energy_spent: f32,
}

impl Creature {
    /// Creates a new creature with the given particles and muscles.
    ///
    /// The clock starts at `0.0` with a default `clock_speed` of `1.0`, and
    /// fitness is initialized to `0.0`.
    pub fn new(particles: Vec<Particle>, muscles: Vec<Muscle>) -> Self {
        Self {
            particles,
            muscles,
            clock: 0.0,
            clock_speed: 1.0,
            fitness: 0.0,
            total_energy_spent: 0.0,
        }
    }

    /// Advances the creature by one simulation step.
    ///
    /// 1. Advances the internal clock.
    /// 2. Updates all muscle activations from the CPG signal.
    /// 3. Integrates all particles (Verlet).
    /// 4. Runs [`CONSTRAINT_ITERS`] iterations of constraint resolution.
    /// 5. Resolves ground collisions on all particles.
    pub fn step(&mut self, dt: f32) {
        use std::f32::consts::TAU;

        // 1. Advance clock
        self.clock = (self.clock + self.clock_speed * dt * TAU) % TAU;

        // 2. Update muscle activations
        for muscle in &mut self.muscles {
            muscle.update_activation(self.clock);
        }

        // 3. Integrate all particles
        for particle in &mut self.particles {
            particle.integrate(dt);
        }

        // 4. Constraint resolution iterations
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

        // Accumulate energy and update fitness
        self.update_fitness(step_energy);
    }

    /// Computes the mass-weighted center of mass of all particles.
    pub fn center_of_mass(&self) -> Vec2 {
        let mut total_mass = 0.0_f32;
        let mut weighted_sum = Vec2::ZERO;

        for p in &self.particles {
            weighted_sum += p.pos * p.mass;
            total_mass += p.mass;
        }

        if total_mass > 0.0 {
            weighted_sum / total_mass
        } else {
            Vec2::ZERO
        }
    }

    /// Updates the fitness score based on horizontal distance and energy cost.
    ///
    /// Fitness rewards rightward movement and penalizes energy expenditure:
    /// `fitness = max(center_of_mass.x, 0) - total_energy_spent * 0.01`
    pub fn update_fitness(&mut self, energy_cost: f32) {
        self.total_energy_spent += energy_cost;
        self.fitness = self.center_of_mass().x.max(0.0) - self.total_energy_spent * 0.01;
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

        // Run several steps to ensure stability
        for _ in 0..100 {
            creature.step(1.0 / 60.0);
        }

        // Particles should still have finite positions
        for p in &creature.particles {
            assert!(p.pos.x.is_finite(), "particle x should be finite");
            assert!(p.pos.y.is_finite(), "particle y should be finite");
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
        assert!((com.x - 2.0).abs() < 1e-5, "center of mass x should be 2.0, got {}", com.x);
    }
}
