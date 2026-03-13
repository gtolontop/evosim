/// A spring-like connection between two [`super::particle::Particle`]s.
///
/// Muscles can oscillate over time to produce locomotion. When `is_bone` is
/// `true` the muscle acts as a rigid structural element that does not
/// oscillate.
#[derive(Debug, Clone)]
pub struct Muscle {
    /// Index of the first particle in the owning creature's particle list.
    pub a: usize,
    /// Index of the second particle in the owning creature's particle list.
    pub b: usize,
    /// Natural (rest) length of the muscle when no oscillation is applied.
    pub rest_len: f32,
    /// Minimum allowed length the muscle can contract to.
    pub min_len: f32,
    /// Spring stiffness coefficient. Higher values make the muscle stiffer.
    pub stiffness: f32,
    /// Phase offset (in radians) for the oscillation cycle.
    pub phase_offset: f32,
    /// Amplitude of length oscillation around `rest_len`.
    pub amplitude: f32,
    /// If `true`, this muscle is a rigid bone and does not oscillate.
    pub is_bone: bool,
}

impl Muscle {
    /// Creates a new muscle connecting particles at indices `a` and `b`.
    ///
    /// The muscle is initialized with the given `rest_len` and `stiffness`,
    /// zero oscillation, and `is_bone` set to `false`.
    pub fn new(a: usize, b: usize, rest_len: f32, stiffness: f32) -> Self {
        Self {
            a,
            b,
            rest_len,
            min_len: rest_len * 0.5,
            stiffness,
            phase_offset: 0.0,
            amplitude: 0.0,
            is_bone: false,
        }
    }
}
