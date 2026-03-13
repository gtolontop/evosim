/// Gravitational acceleration applied each physics tick (pointing downward).
pub const GRAVITY: f32 = -9.81;

/// Velocity damping factor applied per tick to simulate air resistance.
///
/// A value of `1.0` means no damping; lower values slow particles faster.
pub const DAMPING: f32 = 0.99;

/// The Y-coordinate of the ground plane. Particles are constrained to stay
/// above this value.
pub const GROUND_Y: f32 = 0.0;

/// Number of constraint-solver iterations per physics step.
///
/// Higher values produce stiffer, more stable muscles at the cost of
/// performance.
pub const CONSTRAINT_ITERS: u32 = 8;
