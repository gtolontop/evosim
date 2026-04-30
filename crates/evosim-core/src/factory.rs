//! Factory for translating a [`Genome`] into a 16-particle humanoid wall-climbing [`Creature`].
//!
//! ## Morphology
//!
//! A symmetric humanoid skeleton with clavicles, arms, spine, pelvis, legs, and a decorative tail.
//! Matches the anatomical reference images: bones in cream, muscles in salmon pink.
//!
//! ```text
//!  WALL            body
//!    |
//!    [hand_L]─[elbow_L]─[shoulder_L]──┐
//!    |                                 [neck]
//!    [hand_R]─[elbow_R]─[shoulder_R]──┘  │
//!    |                               [spine_mid]
//!    |                                   │
//!    [foot_L]─[knee_L]──[hip_L]──────[pelvis]──[tail]
//!    |                       ╲          ╱
//!    [foot_R]─[knee_R]──[hip_R]─────┘
//! ```
//!
//! ## Particle layout (16 particles)
//!
//! | #  | Name          | Role              |
//! |----|---------------|-------------------|
//! | 0  | neck          | Top of spine      |
//! | 1  | spine_mid     | Mid spine         |
//! | 2  | pelvis_center | Bottom of spine   |
//! | 3  | shoulder_L    | Left shoulder     |
//! | 4  | shoulder_R    | Right shoulder    |
//! | 5  | elbow_L       | Left elbow        |
//! | 6  | elbow_R       | Right elbow       |
//! | 7  | hand_L        | Left hand (grip)  |
//! | 8  | hand_R        | Right hand (grip) |
//! | 9  | hip_L         | Left hip          |
//! | 10 | hip_R         | Right hip         |
//! | 11 | knee_L        | Left knee         |
//! | 12 | knee_R        | Right knee        |
//! | 13 | foot_L        | Left foot (grip)  |
//! | 14 | foot_R        | Right foot (grip) |
//! | 15 | tail_tip      | Decorative tail   |
//!
//! ## Spring layout (35 total)
//!
//! | #     | Type              | Description                          |
//! |-------|-------------------|--------------------------------------|
//! | 0–16  | Bones (rigid)     | Skeleton structure + tail            |
//! | 17–18 | Cross-stability   | Torso diagonals (thin, structural)   |
//! | 19–26 | Left muscles      | Deltoid, dorsal, bicep, tricep, etc. |
//! | 27–34 | Right muscles     | Mirror of left side                  |
//!
//! ## Gene layout (52 genes)
//!
//! | #     | Field              | Range        |
//! |-------|--------------------|--------------|
//! | 0     | spine_length       | [1.0, 3.0]   |
//! | 1     | shoulder_spread    | [0.3, 1.2]   |
//! | 2     | hip_spread         | [0.2, 1.0]   |
//! | 3     | upper_arm_len      | [0.3, 1.5]   |
//! | 4     | forearm_len        | [0.3, 1.5]   |
//! | 5     | thigh_len          | [0.3, 1.5]   |
//! | 6     | shin_len           | [0.3, 1.5]   |
//! | 7     | tail_len           | [0.3, 1.0]   |
//! | 8–15  | muscle max_force   | [0.1, 1.0]   |
//! | 16    | clock_speed        | [0.3, 4.0]   |
//! | 17    | grip_phase_hands   | [0.0, TAU]   |
//! | 18    | grip_phase_feet    | [0.0, TAU]   |
//! | 19    | lr_offset          | [0.0, TAU]   |
//! | 20–51 | keyframes (4×8)    | [0.0, 1.0]   |

use std::f32::consts::TAU;

use evosim_genetics::Genome;

use crate::{
    constants::{MUSCLE_DENSITY, START_HEIGHT, WALL_X},
    creature::{NUM_KEYFRAMES, NUM_MUSCLE_GROUPS},
    Creature, Muscle, Particle,
};

pub struct CreatureFactory;

impl CreatureFactory {
    pub fn min_genome_len() -> usize {
        52
    }

    pub fn build(genome: &Genome) -> Result<Creature, String> {
        if genome.len() < Self::min_genome_len() {
            return Err(format!(
                "genome too short: need at least {}, got {}",
                Self::min_genome_len(),
                genome.len(),
            ));
        }

        let g = genome.genes();
        let r = |idx: usize, min: f32, max: f32| -> f32 {
            (g[idx] + 1.0) / 2.0 * (max - min) + min
        };

        // ── Body dimensions ─────────────────────────────────────────────
        let spine_length    = r(0, 1.0, 3.0);
        let shoulder_spread = r(1, 0.3, 1.2);
        let hip_spread      = r(2, 0.2, 1.0);
        let upper_arm_len   = r(3, 0.3, 1.5);
        let forearm_len     = r(4, 0.3, 1.5);
        let thigh_len       = r(5, 0.3, 1.5);
        let shin_len        = r(6, 0.3, 1.5);
        let tail_len        = r(7, 0.3, 1.0);

        // ── Muscle max forces (8 groups) ────────────────────────────────
        let muscle_forces: [f32; 8] = [
            r(8,  0.1, 1.0), // 0: deltoid
            r(9,  0.1, 1.0), // 1: grand dorsal
            r(10, 0.1, 1.0), // 2: bicep
            r(11, 0.1, 1.0), // 3: tricep
            r(12, 0.1, 1.0), // 4: hip flexor
            r(13, 0.1, 1.0), // 5: glute
            r(14, 0.1, 1.0), // 6: quad
            r(15, 0.1, 1.0), // 7: hamstring
        ];

        // ── Clock + grip ────────────────────────────────────────────────
        let clock_speed      = r(16, 0.3, 4.0);
        let grip_phase_hands = r(17, 0.0, TAU);
        let grip_phase_feet  = r(18, 0.0, TAU);
        let lr_offset        = r(19, 0.0, TAU);

        // ── Keyframes (4 × 8) ──────────────────────────────────────────
        let mut keyframes = [[0.0_f32; NUM_MUSCLE_GROUPS]; NUM_KEYFRAMES];
        for kf in 0..NUM_KEYFRAMES {
            for grp in 0..NUM_MUSCLE_GROUPS {
                keyframes[kf][grp] = r(20 + kf * NUM_MUSCLE_GROUPS + grp, 0.0, 1.0);
            }
        }

        // ── Build particles (16) ────────────────────────────────────────
        let body_x = 1.0; // fixed distance from wall
        let half_spine = spine_length * 0.5;
        let neck_y = START_HEIGHT + half_spine;
        let pelvis_y = START_HEIGHT - half_spine;

        let mut particles = vec![
            // Spine
            Particle::new(body_x, neck_y, 1.0),                        // 0: neck
            Particle::new(body_x, START_HEIGHT, 1.0),                   // 1: spine_mid
            Particle::new(body_x, pelvis_y, 1.0),                      // 2: pelvis_center

            // Shoulders (spread horizontally from neck, slightly toward wall)
            Particle::new(body_x - shoulder_spread * 0.5, neck_y, 1.0), // 3: shoulder_L
            Particle::new(body_x + shoulder_spread * 0.5, neck_y, 1.0), // 4: shoulder_R

            // Elbows: positioned along arm, ratio = upper_arm / (upper_arm + forearm)
            {
                let t = upper_arm_len / (upper_arm_len + forearm_len);
                let sh_x = body_x - shoulder_spread * 0.5;
                Particle::new(
                    sh_x + (WALL_X - sh_x) * t,
                    neck_y - upper_arm_len * 0.2,
                    1.0,
                ) // 5: elbow_L
            },
            {
                let t = upper_arm_len / (upper_arm_len + forearm_len);
                let sh_x = body_x + shoulder_spread * 0.5;
                Particle::new(
                    sh_x + (WALL_X - sh_x) * t,
                    neck_y + forearm_len * 0.3 * t - upper_arm_len * 0.2,
                    1.0,
                ) // 6: elbow_R
            },

            // Hands (on wall)
            Particle::new(WALL_X + 0.02, neck_y, 1.0),                 // 7: hand_L
            Particle::new(WALL_X + 0.02, neck_y + forearm_len * 0.3, 1.0), // 8: hand_R

            // Hips (spread horizontally from pelvis)
            Particle::new(body_x - hip_spread * 0.5, pelvis_y, 1.0),   // 9: hip_L
            Particle::new(body_x + hip_spread * 0.5, pelvis_y, 1.0),   // 10: hip_R

            // Knees (below hips)
            Particle::new(
                body_x - hip_spread * 0.5 - thigh_len * 0.3,
                pelvis_y - thigh_len * 0.5,
                1.0,
            ), // 11: knee_L
            Particle::new(
                body_x + hip_spread * 0.5 + thigh_len * 0.3,
                pelvis_y - thigh_len * 0.5,
                1.0,
            ), // 12: knee_R

            // Feet (on wall, below knees)
            Particle::new(WALL_X + 0.02, pelvis_y - thigh_len * 0.3, 1.0), // 13: foot_L
            Particle::new(WALL_X + 0.02, pelvis_y - thigh_len * 0.3 - shin_len * 0.3, 1.0), // 14: foot_R

            // Tail (hanging from pelvis, away from wall)
            Particle::new(body_x + tail_len * 0.7, pelvis_y - tail_len * 0.3, 1.0), // 15: tail
        ];

        // Pin grip endpoints to wall
        for &idx in &[7usize, 8, 13, 14] {
            particles[idx].pinned = true;
            particles[idx].pos.x = WALL_X;
            particles[idx].prev_pos = particles[idx].pos;
        }

        // ── Distance helper ─────────────────────────────────────────────
        let dist = |a: usize, b: usize| -> f32 {
            (particles[b].pos - particles[a].pos).length()
        };

        // ── Bones (indices 0–18) ────────────────────────────────────────
        let mut muscles = Vec::new();

        // Spine
        muscles.push(Muscle::bone(0, 1, dist(0, 1)));     //  0: upper spine
        muscles.push(Muscle::bone(1, 2, dist(1, 2)));     //  1: lower spine

        // Clavicles
        muscles.push(Muscle::bone(0, 3, dist(0, 3)));     //  2: clavicle_L
        muscles.push(Muscle::bone(0, 4, dist(0, 4)));     //  3: clavicle_R

        // Shoulder bar
        muscles.push(Muscle::bone(3, 4, dist(3, 4)));     //  4: shoulder bar

        // Arms
        muscles.push(Muscle::bone(3, 5, dist(3, 5)));     //  5: humerus_L
        muscles.push(Muscle::bone(4, 6, dist(4, 6)));     //  6: humerus_R
        muscles.push(Muscle::bone(5, 7, dist(5, 7)));     //  7: forearm_L
        muscles.push(Muscle::bone(6, 8, dist(6, 8)));     //  8: forearm_R

        // Pelvis
        muscles.push(Muscle::bone(2, 9, dist(2, 9)));     //  9: pelvis_L
        muscles.push(Muscle::bone(2, 10, dist(2, 10)));   // 10: pelvis_R

        // Hip bar
        muscles.push(Muscle::bone(9, 10, dist(9, 10)));   // 11: hip bar

        // Legs
        muscles.push(Muscle::bone(9, 11, dist(9, 11)));   // 12: femur_L
        muscles.push(Muscle::bone(10, 12, dist(10, 12))); // 13: femur_R
        muscles.push(Muscle::bone(11, 13, dist(11, 13))); // 14: tibia_L
        muscles.push(Muscle::bone(12, 14, dist(12, 14))); // 15: tibia_R

        // Tail
        muscles.push(Muscle::bone(2, 15, dist(2, 15)));   // 16: tail

        // Cross-stability diagonals
        muscles.push(Muscle::bone(3, 10, dist(3, 10)));   // 17: diag L (shoulder_L → hip_R)
        muscles.push(Muscle::bone(4, 9, dist(4, 9)));     // 18: diag R (shoulder_R → hip_L)

        // ── Active muscles (indices 19–34) ──────────────────────────────
        // Left side (19–26)
        muscles.push(Muscle::muscle(0, 5,  dist(0, 5),  muscle_forces[0], 0)); // 19: deltoid_L
        muscles.push(Muscle::muscle(1, 5,  dist(1, 5),  muscle_forces[1], 1)); // 20: grand_dorsal_L
        muscles.push(Muscle::muscle(3, 7,  dist(3, 7),  muscle_forces[2], 2)); // 21: bicep_L
        muscles.push(Muscle::muscle(1, 7,  dist(1, 7),  muscle_forces[3], 3)); // 22: tricep_L
        muscles.push(Muscle::muscle(1, 11, dist(1, 11), muscle_forces[4], 4)); // 23: hip_flexor_L
        muscles.push(Muscle::muscle(2, 11, dist(2, 11), muscle_forces[5], 5)); // 24: glute_L
        muscles.push(Muscle::muscle(9, 13, dist(9, 13), muscle_forces[6], 6)); // 25: quad_L
        muscles.push(Muscle::muscle(2, 13, dist(2, 13), muscle_forces[7], 7)); // 26: hamstring_L

        // Right side (27–34) — same groups, mirrored particles
        muscles.push(Muscle::muscle(0, 6,  dist(0, 6),  muscle_forces[0], 0)); // 27: deltoid_R
        muscles.push(Muscle::muscle(1, 6,  dist(1, 6),  muscle_forces[1], 1)); // 28: grand_dorsal_R
        muscles.push(Muscle::muscle(4, 8,  dist(4, 8),  muscle_forces[2], 2)); // 29: bicep_R
        muscles.push(Muscle::muscle(1, 8,  dist(1, 8),  muscle_forces[3], 3)); // 30: tricep_R
        muscles.push(Muscle::muscle(1, 12, dist(1, 12), muscle_forces[4], 4)); // 31: hip_flexor_R
        muscles.push(Muscle::muscle(2, 12, dist(2, 12), muscle_forces[5], 5)); // 32: glute_R
        muscles.push(Muscle::muscle(10, 14, dist(10, 14), muscle_forces[6], 6)); // 33: quad_R
        muscles.push(Muscle::muscle(2, 14, dist(2, 14), muscle_forces[7], 7)); // 34: hamstring_R

        // ── Particle masses: bone base + muscle contribution ────────────
        // Base mass from connected bone lengths
        let mut link_sums = vec![0.0_f32; particles.len()];
        for muscle in &muscles {
            if muscle.is_bone {
                let len = (particles[muscle.b].pos - particles[muscle.a].pos).length();
                link_sums[muscle.a] += len;
                link_sums[muscle.b] += len;
            }
        }
        let max_link = link_sums.iter().cloned().fold(0.0_f32, f32::max);
        if max_link > 0.0 {
            for (i, p) in particles.iter_mut().enumerate() {
                p.mass = 0.1 + (link_sums[i] / max_link) * 1.0; // bone mass: 0.1 to 1.1
            }
        }

        // Add muscle mass contributions to endpoints
        for muscle in &muscles {
            if !muscle.is_bone {
                let mm = muscle.max_force * muscle.rest_len * MUSCLE_DENSITY;
                let half = mm * 0.5;
                particles[muscle.a].mass += half;
                particles[muscle.b].mass += half;
            }
        }

        // ── Assemble creature ───────────────────────────────────────────
        let mut creature = Creature::new(particles, muscles);
        creature.clock_speed = clock_speed;
        creature.lr_offset = lr_offset;
        creature.keyframes = keyframes;
        creature.grip_phases = vec![
            (7,  grip_phase_hands),
            (8,  grip_phase_hands + std::f32::consts::PI), // opposite phase for alternating
            (13, grip_phase_feet),
            (14, grip_phase_feet + std::f32::consts::PI),
        ];

        Ok(creature)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_genome(val: f32) -> Genome {
        Genome::new(vec![val; CreatureFactory::min_genome_len()])
    }

    #[test]
    fn build_succeeds() {
        assert!(CreatureFactory::build(&flat_genome(0.0)).is_ok());
    }

    #[test]
    fn build_fails_short() {
        let short = Genome::new(vec![0.0; CreatureFactory::min_genome_len() - 1]);
        assert!(CreatureFactory::build(&short).is_err());
    }

    #[test]
    fn has_16_particles() {
        let c = CreatureFactory::build(&flat_genome(0.0)).unwrap();
        assert_eq!(c.particles.len(), 16);
    }

    #[test]
    fn has_35_muscles() {
        let c = CreatureFactory::build(&flat_genome(0.0)).unwrap();
        assert_eq!(c.muscles.len(), 35);
    }

    #[test]
    fn grip_phases_set() {
        let c = CreatureFactory::build(&flat_genome(0.0)).unwrap();
        assert_eq!(c.grip_phases.len(), 4);
    }

    #[test]
    fn endpoints_start_pinned() {
        let c = CreatureFactory::build(&flat_genome(0.0)).unwrap();
        for &idx in &[7usize, 8, 13, 14] {
            assert!(c.particles[idx].pinned, "particle {idx} should start pinned");
        }
    }

    #[test]
    fn neck_above_pelvis() {
        let c = CreatureFactory::build(&flat_genome(0.0)).unwrap();
        assert!(c.particles[0].pos.y > c.particles[2].pos.y, "neck should be above pelvis");
    }

    #[test]
    fn has_keyframes() {
        let c = CreatureFactory::build(&flat_genome(0.0)).unwrap();
        // All keyframes should be populated (from genome)
        for kf in &c.keyframes {
            for &v in kf {
                assert!(v >= 0.0 && v <= 1.0);
            }
        }
    }

    #[test]
    fn muscle_mass_positive() {
        let c = CreatureFactory::build(&flat_genome(0.0)).unwrap();
        assert!(c.total_muscle_mass > 0.0);
    }

    #[test]
    fn step_100_no_panic() {
        let mut c = CreatureFactory::build(&flat_genome(0.0)).unwrap();
        for _ in 0..100 {
            c.step(0.016);
        }
        for p in &c.particles {
            assert!(p.pos.x.is_finite());
            assert!(p.pos.y.is_finite());
        }
    }
}
