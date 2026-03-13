use crate::creature::Creature;

/// Top-level simulation state holding all creatures and global parameters.
///
/// The world tracks the current generation number and the fixed time-step
/// used for physics integration.
#[derive(Debug, Clone)]
pub struct World {
    /// All creatures currently being simulated.
    pub creatures: Vec<Creature>,
    /// Current generation counter (incremented after each selection cycle).
    pub generation: u32,
    /// Fixed time-step for physics integration (in seconds).
    pub dt: f32,
}

impl World {
    /// Creates a new, empty world with the given time-step.
    ///
    /// Starts at generation `0` with no creatures.
    pub fn new(dt: f32) -> Self {
        Self {
            creatures: Vec::new(),
            generation: 0,
            dt,
        }
    }
}
