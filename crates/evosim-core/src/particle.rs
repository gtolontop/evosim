use glam::Vec2;

use crate::constants::{DAMPING, GRAVITY, GROUND_FRICTION, GROUND_RESTITUTION, GROUND_Y};

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
    /// Creates a new particle at `(x, y)` with the specified mass.
    ///
    /// The particle starts at rest (`prev_pos == pos`) and is not pinned.
    pub fn new(x: f32, y: f32, mass: f32) -> Self {
        let pos = Vec2::new(x, y);
        Self {
            pos,
            prev_pos: pos,
            mass,
            pinned: false,
        }
    }

    /// Advances the particle one time-step using Verlet integration.
    ///
    /// Applies [`DAMPING`] to the implicit velocity and adds [`GRAVITY`] as
    /// a downward acceleration. Pinned particles are not moved.
    pub fn integrate(&mut self, dt: f32) {
        if self.pinned {
            return;
        }

        let velocity = (self.pos - self.prev_pos) * DAMPING;
        self.prev_pos = self.pos;
        self.pos += velocity + Vec2::new(0.0, GRAVITY) * dt * dt;
    }

    /// Resolves collision with the ground plane at [`GROUND_Y`].
    ///
    /// If the particle is below the ground, it is pushed up and its vertical
    /// velocity is reflected with [`GROUND_RESTITUTION`]. Horizontal velocity
    /// is scaled by [`GROUND_FRICTION`].
    pub fn resolve_ground(&mut self) {
        if self.pos.y < GROUND_Y {
            self.pos.y = GROUND_Y;

            // Reflect vertical velocity with restitution
            let vy = self.pos.y - self.prev_pos.y;
            self.prev_pos.y = self.pos.y + vy * GROUND_RESTITUTION;

            // Apply friction to horizontal velocity
            let vx = self.pos.x - self.prev_pos.x;
            self.prev_pos.x = self.pos.x - vx * GROUND_FRICTION;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integrate_moves_particle_downward() {
        let mut p = Particle::new(0.0, 5.0, 1.0);
        p.integrate(1.0 / 60.0);
        assert!(
            p.pos.y < 5.0,
            "particle should move downward due to gravity, got y={}",
            p.pos.y
        );
    }

    #[test]
    fn pinned_particle_does_not_move() {
        let mut p = Particle::new(1.0, 2.0, 1.0);
        p.pinned = true;
        p.integrate(1.0 / 60.0);
        assert_eq!(p.pos, Vec2::new(1.0, 2.0));
    }

    #[test]
    fn resolve_ground_keeps_particle_above_ground() {
        let mut p = Particle::new(0.0, -1.0, 1.0);
        p.resolve_ground();
        assert!(
            p.pos.y >= GROUND_Y,
            "particle should be at or above ground, got y={}",
            p.pos.y
        );
    }
}
