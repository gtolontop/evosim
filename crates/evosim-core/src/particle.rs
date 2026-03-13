use glam::Vec2;

/// A point-mass particle used as a node in soft-body creatures.
///
/// Particles store their current and previous positions so that Verlet
/// integration can derive velocity implicitly.
#[derive(Debug, Clone)]
pub struct Particle {
    /// Current position in world space.
    pub pos: Vec2,
    /// Position from the previous physics step (used for Verlet integration).
    pub prev_pos: Vec2,
    /// Mass of the particle in arbitrary units. Heavier particles resist
    /// forces more.
    pub mass: f32,
    /// Whether this particle is pinned in place and ignores physics forces.
    pub pinned: bool,
}

impl Particle {
    /// Creates a new particle at the given position with the specified mass.
    ///
    /// The particle starts at rest (`prev_pos == pos`) and is not pinned.
    pub fn new(pos: Vec2, mass: f32) -> Self {
        Self {
            pos,
            prev_pos: pos,
            mass,
            pinned: false,
        }
    }
}
