/// Gravitational acceleration applied each physics tick (pointing downward).
pub const GRAVITY: f32 = -9.81;

/// Velocity damping factor applied per tick to simulate air resistance.
///
/// A value of `1.0` means no damping; lower values slow particles faster.
pub const DAMPING: f32 = 0.98;

/// The Y-coordinate of the ground plane. Particles are constrained to stay
/// above this value.
pub const GROUND_Y: f32 = 0.0;

/// Coefficient of restitution for ground collisions. Controls how much
/// vertical velocity is preserved on bounce (0 = no bounce, 1 = perfect bounce).
pub const GROUND_RESTITUTION: f32 = 0.3;

/// Friction coefficient applied to horizontal velocity on ground contact.
/// Lower values mean more friction (slower sliding).
pub const GROUND_FRICTION: f32 = 0.85;

/// Number of constraint-solver iterations per physics step.
///
/// Higher values produce stiffer, more stable muscles at the cost of
/// performance.
pub const CONSTRAINT_ITERS: u32 = 10;
