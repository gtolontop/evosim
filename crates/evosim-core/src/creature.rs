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
        }
    }
}
