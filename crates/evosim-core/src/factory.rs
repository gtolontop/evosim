//! Factory for translating a [`Genome`] into a 15-particle humanoid wall-climbing [`Creature`].
//!
//! ## Morphology
//!
//! A vertical spine close to a wall at x=0.  Two shoulders branch from the
//! upper spine; two hips branch from the lower spine.  Each limb has a joint
//! (elbow/knee) and an endpoint (hand/foot) that can grip the wall.
//!
//! ```text
//!  WALL            body
//!    |
//!    [hand_A]─[elbow_A]─[shoulder_A]
//!    |                       ╲
//!    [hand_B]─[elbow_B]─[shoulder_B]─[spine_top]
//!    |                                   │
//!    |                               [spine_mid]
//!    |                                   │
//!    [foot_A]─[knee_A]──[hip_A]──────[spine_bot]
//!    |                       ╱
//!    [foot_B]─[knee_B]──[hip_B]
//! ```
//!
//! ## Particle layout (15 particles)
//!
//! | #  | Name        | Role            |
//! |----|-------------|-----------------|
//! | 0  | spine_top   | Upper spine     |
//! | 1  | spine_mid   | Mid spine       |
//! | 2  | spine_bot   | Lower spine     |
//! | 3  | shoulder_A  | Shoulder A      |
//! | 4  | shoulder_B  | Shoulder B      |
//! | 5  | elbow_A     | Elbow A         |
//! | 6  | hand_A      | Hand A (grip)   |
//! | 7  | elbow_B     | Elbow B         |
//! | 8  | hand_B      | Hand B (grip)   |
//! | 9  | hip_A       | Hip A           |
//! | 10 | hip_B       | Hip B           |
//! | 11 | knee_A      | Knee A          |
//! | 12 | foot_A      | Foot A (grip)   |
//! | 13 | knee_B      | Knee B          |
//! | 14 | foot_B      | Foot B (grip)   |
//!
//! ## Gene layout (40 genes)
//!
//! | #     | Field              | Range      |
//! |-------|--------------------|------------|
//! | 0     | spine_length       | [1.0, 3.0] |
//! | 1     | spine_split        | [0.3, 0.7] |
//! | 2     | shoulder_spread    | [0.3, 1.2] |
//! | 3     | hip_spread         | [0.2, 1.0] |
//! | 4     | body_offset_x      | [0.5, 1.5] |
//! | 5     | clock_speed        | [0.3, 4.0] |
//! | 6–13  | arm A              | per-limb   |
//! | 14–21 | arm B              | per-limb   |
//! | 22–29 | leg A              | per-limb   |
//! | 30–37 | leg B              | per-limb   |
//! | 38–39 | reserved           |            |

use std::f32::consts::TAU;

use evosim_genetics::Genome;

use crate::{
    constants::{START_HEIGHT, WALL_X},
    Creature, Muscle, Particle,
};

pub struct CreatureFactory;

impl CreatureFactory {
    pub fn min_genome_len() -> usize {
        40
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

        // ── Header ────────────────────────────────────────────────────────
        let spine_length    = r(0, 1.0, 3.0);
        let spine_split     = r(1, 0.3, 0.7);
        let shoulder_spread = r(2, 0.3, 1.2);
        let hip_spread      = r(3, 0.2, 1.0);
        let body_offset_x   = r(4, 0.5, 1.5);
        let clock_speed     = r(5, 0.3, 4.0);

        // ── Spine positions ───────────────────────────────────────────────
        let spine_top_y = START_HEIGHT + spine_length * spine_split;
        let spine_bot_y = START_HEIGHT - spine_length * (1.0 - spine_split);

        // ── Per-limb gene decoding ────────────────────────────────────────
        struct LimbGenes {
            upper_len: f32,
            lower_len: f32,
            upper_force: f32,
            lower_force: f32,
            amplitude: f32,
            phase: f32,
            grip_phase: f32,
            contraction: f32,
        }

        let decode = |base: usize| LimbGenes {
            upper_len:   r(base,     0.3, 1.5),
            lower_len:   r(base + 1, 0.3, 1.5),
            upper_force: r(base + 2, 0.1, 1.0),
            lower_force: r(base + 3, 0.1, 1.0),
            amplitude:   r(base + 4, 0.1, 1.0),
            phase:       r(base + 5, 0.0, TAU),
            grip_phase:  r(base + 6, 0.0, TAU),
            contraction: r(base + 7, 0.3, 0.8),
        };

        let arm_a = decode(6);
        let arm_b = decode(14);
        let leg_a = decode(22);
        let leg_b = decode(30);

        // ── Build particles (15) ──────────────────────────────────────────
        // Spine: vertical at body_offset_x
        let spine_top = Particle::new(body_offset_x, spine_top_y, 1.0);
        let spine_mid = Particle::new(body_offset_x, START_HEIGHT, 1.0);
        let spine_bot = Particle::new(body_offset_x, spine_bot_y, 1.0);

        // Shoulders: offset vertically from spine_top, slightly toward wall
        let sh_x = body_offset_x * 0.7;
        let shoulder_a = Particle::new(sh_x, spine_top_y + shoulder_spread * 0.5, 1.0);
        let shoulder_b = Particle::new(sh_x, spine_top_y - shoulder_spread * 0.5, 1.0);

        // Hips: offset vertically from spine_bot, slightly toward wall
        let hip_x = body_offset_x * 0.7;
        let hip_a = Particle::new(hip_x, spine_bot_y + hip_spread * 0.5, 1.0);
        let hip_b = Particle::new(hip_x, spine_bot_y - hip_spread * 0.5, 1.0);

        // Helper: place joint (elbow/knee) between attachment and wall endpoint.
        // Both upper_len and lower_len influence the vertical spread of the limb.
        let mk_limb = |attach: &Particle, lg: &LimbGenes, y_offset_sign: f32|
            -> (Particle, Particle)
        {
            // Hand/foot: on the wall, offset vertically using both segment lengths
            let end_y = attach.pos.y + y_offset_sign * (lg.upper_len + lg.lower_len) * 0.2;
            let end = Particle::new(WALL_X + 0.02, end_y, 1.0);

            // Joint: offset from attachment by upper_len fraction toward endpoint
            let t = lg.upper_len / (lg.upper_len + lg.lower_len);
            let jx = attach.pos.x + (end.pos.x - attach.pos.x) * t;
            let jy = attach.pos.y + (end.pos.y - attach.pos.y) * t;
            let joint = Particle::new(jx, jy, 1.0);

            (joint, end)
        };

        // Arms reach upward from shoulders
        let (elbow_a, hand_a) = mk_limb(&shoulder_a, &arm_a, 1.0);
        let (elbow_b, hand_b) = mk_limb(&shoulder_b, &arm_b, 1.0);
        // Legs reach downward from hips
        let (knee_a, foot_a) = mk_limb(&hip_a, &leg_a, -1.0);
        let (knee_b, foot_b) = mk_limb(&hip_b, &leg_b, -1.0);

        let mut particles = vec![
            spine_top,   // 0
            spine_mid,   // 1
            spine_bot,   // 2
            shoulder_a,  // 3
            shoulder_b,  // 4
            elbow_a,     // 5
            hand_a,      // 6
            elbow_b,     // 7
            hand_b,      // 8
            hip_a,       // 9
            hip_b,       // 10
            knee_a,      // 11
            foot_a,      // 12
            knee_b,      // 13
            foot_b,      // 14
        ];

        // Pin grip endpoints to wall
        for &idx in &[6usize, 8, 12, 14] {
            particles[idx].pinned = true;
            particles[idx].pos.x = WALL_X;
            particles[idx].prev_pos = particles[idx].pos;
        }

        // ── Helper: distance between two particles ────────────────────────
        let dist = |a: usize, b: usize| -> f32 {
            (particles[b].pos - particles[a].pos).length()
        };

        // ── Muscles ───────────────────────────────────────────────────────
        let mut muscles = Vec::new();

        // Structural bones (indices 0–7)
        muscles.push(Muscle::bone(0, 1, dist(0, 1)));   // 0: upper spine
        muscles.push(Muscle::bone(1, 2, dist(1, 2)));   // 1: lower spine
        muscles.push(Muscle::bone(0, 3, dist(0, 3)));   // 2: clavicle A
        muscles.push(Muscle::bone(0, 4, dist(0, 4)));   // 3: clavicle B
        muscles.push(Muscle::bone(2, 9, dist(2, 9)));   // 4: pelvis A
        muscles.push(Muscle::bone(2, 10, dist(2, 10))); // 5: pelvis B
        muscles.push(Muscle::bone(3, 4, dist(3, 4)));   // 6: shoulder bar
        muscles.push(Muscle::bone(9, 10, dist(9, 10))); // 7: hip bar

        // Active muscles (indices 8–15)
        let add_limb = |muscles: &mut Vec<Muscle>,
                        attach: usize,
                        joint: usize,
                        end: usize,
                        lg: &LimbGenes| {
            let prox_rest = dist(attach, joint);
            let dist_rest = dist(joint, end);
            // Proximal: evolved phase and force
            muscles.push(Muscle::muscle(
                attach, joint,
                prox_rest, prox_rest * lg.contraction,
                lg.upper_force, lg.phase, lg.amplitude,
            ));
            // Distal: phase + PI/2, evolved force
            muscles.push(Muscle::muscle(
                joint, end,
                dist_rest, dist_rest * lg.contraction,
                lg.lower_force, lg.phase + std::f32::consts::FRAC_PI_2, lg.amplitude,
            ));
        };

        add_limb(&mut muscles, 3, 5, 6, &arm_a);    // 8, 9: arm A
        add_limb(&mut muscles, 4, 7, 8, &arm_b);    // 10, 11: arm B
        add_limb(&mut muscles, 9, 11, 12, &leg_a);   // 12, 13: leg A
        add_limb(&mut muscles, 10, 13, 14, &leg_b);  // 14, 15: leg B

        // Cross-stability bones (indices 16–17)
        muscles.push(Muscle::bone(3, 9, dist(3, 9)));   // 16: left torso
        muscles.push(Muscle::bone(4, 10, dist(4, 10))); // 17: right torso

        // ── Mass from link lengths ────────────────────────────────────────
        let mut link_sums = vec![0.0_f32; particles.len()];
        for muscle in &muscles {
            let len = (particles[muscle.b].pos - particles[muscle.a].pos).length();
            link_sums[muscle.a] += len;
            link_sums[muscle.b] += len;
        }
        let max_link = link_sums.iter().cloned().fold(0.0_f32, f32::max);
        if max_link > 0.0 {
            for (i, p) in particles.iter_mut().enumerate() {
                p.mass = 0.1 + (link_sums[i] / max_link) * 1.7;
            }
        }

        // ── Assemble creature ─────────────────────────────────────────────
        let mut creature = Creature::new(particles, muscles);
        creature.clock_speed = clock_speed;
        creature.grip_phases = vec![
            (6,  arm_a.grip_phase),
            (8,  arm_b.grip_phase),
            (12, leg_a.grip_phase),
            (14, leg_b.grip_phase),
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
    fn has_15_particles() {
        let c = CreatureFactory::build(&flat_genome(0.0)).unwrap();
        assert_eq!(c.particles.len(), 15);
    }

    #[test]
    fn grip_phases_set() {
        let c = CreatureFactory::build(&flat_genome(0.0)).unwrap();
        assert_eq!(c.grip_phases.len(), 4);
    }

    #[test]
    fn endpoints_start_pinned() {
        let c = CreatureFactory::build(&flat_genome(0.0)).unwrap();
        for &idx in &[6usize, 8, 12, 14] {
            assert!(c.particles[idx].pinned, "particle {idx} should start pinned");
        }
    }

    #[test]
    fn shoulders_above_hips() {
        let c = CreatureFactory::build(&flat_genome(0.0)).unwrap();
        let shoulder_avg_y = (c.particles[3].pos.y + c.particles[4].pos.y) / 2.0;
        let hip_avg_y = (c.particles[9].pos.y + c.particles[10].pos.y) / 2.0;
        assert!(shoulder_avg_y > hip_avg_y, "shoulders should be above hips");
    }

    #[test]
    fn mass_derived_from_links() {
        let c = CreatureFactory::build(&flat_genome(0.0)).unwrap();
        // Masses should vary (not all equal)
        let masses: Vec<f32> = c.particles.iter().map(|p| p.mass).collect();
        let min = masses.iter().cloned().fold(f32::INFINITY, f32::min);
        let max = masses.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        assert!(max > min + 0.01, "masses should vary, got min={min}, max={max}");
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
