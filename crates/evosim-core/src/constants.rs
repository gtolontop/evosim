/// Gravitational acceleration applied each physics tick (pointing downward).
pub const GRAVITY: f32 = -9.81;

/// Velocity damping factor applied per tick.
pub const DAMPING: f32 = 0.995;

/// The Y-coordinate of the ground plane.
pub const GROUND_Y: f32 = 0.0;

/// Coefficient of restitution for ground collisions.
pub const GROUND_RESTITUTION: f32 = 0.2;

/// Friction coefficient applied to horizontal velocity on ground contact.
pub const GROUND_FRICTION: f32 = 0.9;

/// Number of constraint-solver iterations per physics step.
pub const CONSTRAINT_ITERS: u32 = 15;

// ── Wall / grip ──────────────────────────────────────────────────────────────

/// X-coordinate of the vertical wall that creatures climb.
pub const WALL_X: f32 = 0.0;

/// A particle is considered "at the wall" when its X ≤ WALL_X + GRIP_DIST.
pub const GRIP_DIST: f32 = 0.25;

/// Y height at which creatures spawn on the wall.
pub const START_HEIGHT: f32 = 3.0;

/// Fitness penalty per unit of total creature mass (K2).
pub const MASS_PENALTY: f32 = 0.002;

/// Fitness penalty per unit of accumulated energy spent (K1).
pub const ENERGY_PENALTY: f32 = 0.001;

/// Fitness penalty per unit of accumulated grip-impact v² (K3).
pub const IMPACT_PENALTY: f32 = 0.1;

/// Fitness penalty per unit of total muscle mass (K4).
pub const MUSCLE_MASS_PENALTY: f32 = 0.005;

/// Density factor: muscle_mass = max_force × rest_len × MUSCLE_DENSITY.
pub const MUSCLE_DENSITY: f32 = 0.5;

/// Maximum contraction ratio — muscles can shorten to (1 - MAX_CONTRACTION) × rest_len.
pub const MAX_CONTRACTION: f32 = 0.5;

/// Minimum distance between non-connected particles (self-collision).
pub const SELF_COLLISION_RADIUS: f32 = 0.2;
