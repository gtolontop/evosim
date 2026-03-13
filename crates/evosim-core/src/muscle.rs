use crate::particle::Particle;

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
    /// Current activation level in `[0, amplitude]`, driven by the CPG signal.
    pub current_activation: f32,
}

/// Creates a rigid bone constraint between two particles.
///
/// Bones have full stiffness (`1.0`), no oscillation, and `is_bone` set to
/// `true`.
impl Muscle {
    pub fn bone(a: usize, b: usize, rest_len: f32) -> Self {
        Self {
            a,
            b,
            rest_len,
            min_len: rest_len,
            stiffness: 1.0,
            phase_offset: 0.0,
            amplitude: 0.0,
            is_bone: true,
            current_activation: 0.0,
        }
    }

    /// Creates an oscillating muscle between two particles.
    ///
    /// The muscle contracts between `rest_len` and `min_len` according to a
    /// sinusoidal CPG signal controlled by `phase_offset` and `amplitude`.
    pub fn muscle(
        a: usize,
        b: usize,
        rest_len: f32,
        min_len: f32,
        stiffness: f32,
        phase_offset: f32,
        amplitude: f32,
    ) -> Self {
        Self {
            a,
            b,
            rest_len,
            min_len,
            stiffness,
            phase_offset,
            amplitude,
            is_bone: false,
            current_activation: 0.0,
        }
    }

    /// Updates the muscle's activation level using a sinusoidal CPG signal.
    ///
    /// The activation oscillates between `0` and `amplitude` based on the
    /// creature's internal `clock` plus this muscle's `phase_offset`.
    /// Bones are unaffected.
    pub fn update_activation(&mut self, clock: f32) {
        if self.is_bone {
            return;
        }
        self.current_activation =
            ((clock + self.phase_offset).sin() * 0.5 + 0.5) * self.amplitude;
    }

    /// Returns the current target length based on activation.
    ///
    /// Linearly interpolates between `rest_len` (at activation 0) and
    /// `min_len` (at activation 1).
    pub fn target_len(&self) -> f32 {
        self.rest_len + (self.min_len - self.rest_len) * self.current_activation
    }

    /// Resolves the constraint by adjusting the two connected particles.
    ///
    /// Applies mass-weighted position corrections scaled by `stiffness` to
    /// push/pull the particles toward the target length. Returns the energy
    /// cost of this activation (always `0.0` for bones).
    pub fn resolve(&self, particles: &mut [Particle]) -> f32 {
        let pa = particles[self.a].pos;
        let pb = particles[self.b].pos;

        let delta = pb - pa;
        let distance = delta.length();

        if distance < 1e-8 {
            return 0.0;
        }

        let target = self.target_len();
        let error = distance - target;
        let correction = delta / distance * error * self.stiffness;

        let mass_a = particles[self.a].mass;
        let mass_b = particles[self.b].mass;
        let total_mass = mass_a + mass_b;

        if !particles[self.a].pinned {
            particles[self.a].pos += correction * (mass_b / total_mass);
        }
        if !particles[self.b].pinned {
            particles[self.b].pos -= correction * (mass_a / total_mass);
        }

        if self.is_bone {
            0.0
        } else {
            self.current_activation * error.abs() * self.stiffness
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bone_resolves_stretched_constraint() {
        let mut particles = vec![
            Particle::new(0.0, 0.0, 1.0),
            Particle::new(3.0, 0.0, 1.0),
        ];
        let bone = Muscle::bone(0, 1, 1.0);

        let energy = bone.resolve(&mut particles);

        let new_dist = (particles[1].pos - particles[0].pos).length();
        assert!(
            (new_dist - 1.0).abs() < 0.1,
            "bone should pull particles closer to rest_len, got dist={}",
            new_dist
        );
        assert_eq!(energy, 0.0, "bones should have zero energy cost");
    }

    #[test]
    fn muscle_activation_oscillates() {
        let mut m = Muscle::muscle(0, 1, 2.0, 1.0, 0.5, 0.0, 1.0);

        m.update_activation(0.0);
        let a0 = m.current_activation;

        m.update_activation(std::f32::consts::FRAC_PI_2);
        let a1 = m.current_activation;

        assert!(
            (a1 - a0).abs() > 0.01,
            "activation should change with clock, a0={a0}, a1={a1}"
        );
    }
}
