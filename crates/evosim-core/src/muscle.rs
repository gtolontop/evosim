use crate::constants::{MAX_CONTRACTION, MUSCLE_DENSITY};
use crate::particle::Particle;

/// A spring-like connection between two [`Particle`]s.
///
/// When `is_bone` is true the muscle acts as a rigid structural element.
/// Active muscles have a `max_force` that also determines their mass
/// (heavier muscles = more force but more weight).
#[derive(Debug, Clone)]
pub struct Muscle {
    pub a: usize,
    pub b: usize,
    pub rest_len: f32,
    pub stiffness: f32,
    pub is_bone: bool,
    /// Maximum force this muscle can exert. Also determines muscle mass.
    pub max_force: f32,
    /// Current contraction ratio in [0, 1]. Driven by keyframe interpolation.
    pub current_contraction: f32,
    /// Which muscle group this belongs to (0–7), or `u8::MAX` for bones.
    pub group: u8,
}

impl Muscle {
    /// Creates a rigid bone constraint.
    pub fn bone(a: usize, b: usize, rest_len: f32) -> Self {
        Self {
            a,
            b,
            rest_len,
            stiffness: 1.0,
            is_bone: true,
            max_force: 0.0,
            current_contraction: 0.0,
            group: u8::MAX,
        }
    }

    /// Creates an active muscle with a maximum force.
    pub fn muscle(a: usize, b: usize, rest_len: f32, max_force: f32, group: u8) -> Self {
        Self {
            a,
            b,
            rest_len,
            stiffness: max_force,
            is_bone: false,
            max_force,
            current_contraction: 0.0,
            group,
        }
    }

    /// Sets the contraction ratio (called by the keyframe interpolation system).
    #[inline]
    pub fn set_contraction(&mut self, contraction: f32) {
        if self.is_bone {
            return;
        }
        self.current_contraction = contraction.clamp(0.0, 1.0);
    }

    /// Current target length based on contraction.
    /// Fully relaxed (0) → rest_len. Fully contracted (1) → rest_len * (1 - MAX_CONTRACTION).
    #[inline]
    pub fn target_len(&self) -> f32 {
        self.rest_len * (1.0 - self.current_contraction * MAX_CONTRACTION)
    }

    /// Mass of this muscle: force × rest_len × density. Zero for bones.
    #[inline]
    pub fn muscle_mass(&self) -> f32 {
        if self.is_bone {
            0.0
        } else {
            self.max_force * self.rest_len * MUSCLE_DENSITY
        }
    }

    /// Resolves the constraint by adjusting connected particles.
    /// Returns the energy cost (0 for bones).
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
            self.current_contraction * error.abs() * self.stiffness
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
            "bone should pull particles closer, got dist={new_dist}"
        );
        assert_eq!(energy, 0.0);
    }

    #[test]
    fn muscle_contraction_changes_target() {
        let mut m = Muscle::muscle(0, 1, 2.0, 0.5, 0);
        assert!((m.target_len() - 2.0).abs() < 1e-5);
        m.set_contraction(1.0);
        assert!(m.target_len() < 2.0);
    }

    #[test]
    fn muscle_mass_is_positive() {
        let m = Muscle::muscle(0, 1, 2.0, 0.5, 0);
        assert!(m.muscle_mass() > 0.0);
    }

    #[test]
    fn bone_has_zero_muscle_mass() {
        let b = Muscle::bone(0, 1, 1.0);
        assert_eq!(b.muscle_mass(), 0.0);
    }
}
