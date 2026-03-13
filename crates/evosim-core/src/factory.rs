//! Factory for translating a [`Genome`] into a physical [`Creature`].
//!
//! The genome is a flat `Vec<f32>` of values in `[-1.0, 1.0]`. Genes are read
//! sequentially by an internal cursor and remapped to physical ranges. The
//! resulting creature is bilaterally symmetric (left/right mirrored limbs).

use std::f32::consts::TAU;

use evosim_genetics::Genome;

use crate::{Creature, Muscle, Particle};

/// Maximum number of body segments (bilateral pairs) a creature can have.
const MAX_SEGMENTS: usize = 4;

/// Number of genes consumed per body segment.
const GENES_PER_SEGMENT: usize = 7;

/// Number of "header" genes read before per-segment data (num_segments + body_mass).
const HEADER_GENES: usize = 2;

// ── Per-segment parameters decoded from the genome ─────────────────────────

struct SegmentParams {
    limb_length: f32,
    limb_mass: f32,
    muscle_stiffness: f32,
    muscle_min_ratio: f32,
    phase_offset: f32,
    amplitude: f32,
}

// ── Gene reader ─────────────────────────────────────────────────────────────

/// Reads genes sequentially from a genome slice using an internal cursor.
struct GeneReader<'a> {
    genes: &'a [f32],
    cursor: usize,
}

impl<'a> GeneReader<'a> {
    fn new(genes: &'a [f32]) -> Self {
        Self { genes, cursor: 0 }
    }

    /// Returns the next raw gene value in `[-1, 1]` and advances the cursor.
    ///
    /// # Panics
    ///
    /// Panics if all genes are exhausted. This cannot happen after the
    /// upfront length check in [`CreatureFactory::build`].
    fn next(&mut self) -> f32 {
        let v = *self
            .genes
            .get(self.cursor)
            .expect("internal error: gene cursor exceeded validated genome length");
        self.cursor += 1;
        v
    }

    /// Reads the next gene and linearly remaps it from `[-1, 1]` to `[min, max]`.
    fn next_range(&mut self, min: f32, max: f32) -> f32 {
        let v = self.next();
        (v + 1.0) / 2.0 * (max - min) + min
    }

    /// Reads the next gene and maps it to a `usize` in `[min, max]` (inclusive).
    fn next_usize(&mut self, min: usize, max: usize) -> usize {
        let v = self.next();
        let t = (v + 1.0) / 2.0; // normalise to [0, 1]
        let range = (max - min + 1) as f32;
        let idx = (t * range).floor() as usize;
        (min + idx).min(max)
    }
}

// ── Public API ───────────────────────────────────────────────────────────────

/// Stateless factory that translates a [`Genome`] into a physical [`Creature`].
///
/// The genome encodes a bilaterally symmetric creature with 1–4 body segments.
/// Each segment spawns a mirrored pair of limb particles connected by an
/// active muscle, and rigid bones anchor each limb to the central body.
pub struct CreatureFactory;

impl CreatureFactory {
    /// Minimum number of genes required to successfully build any creature.
    ///
    /// Equal to `HEADER_GENES + GENES_PER_SEGMENT * MAX_SEGMENTS` (2 + 7×4 = **30**).
    /// A genome of this length is always sufficient regardless of which
    /// `num_segments` value is encoded in the first gene.
    pub fn min_genome_len() -> usize {
        HEADER_GENES + GENES_PER_SEGMENT * MAX_SEGMENTS
    }

    /// Builds a [`Creature`] from the given [`Genome`].
    ///
    /// Genes are consumed in this exact order:
    ///
    /// | # | Field             | Physical range      |
    /// |---|-------------------|---------------------|
    /// | 1 | `num_segments`    | `usize` in `[1, 4]` |
    /// | 2 | `body_mass`       | `f32` in `[0.5, 3.0]` |
    /// | per segment (repeated `MAX_SEGMENTS` times, values used for `num_segments`): |
    /// | a | `limb_length`     | `f32` in `[0.2, 1.5]` |
    /// | b | `limb_mass`       | `f32` in `[0.1, 1.0]` |
    /// | c | `muscle_stiffness`| `f32` in `[0.3, 1.0]` |
    /// | d | `muscle_min_ratio`| `f32` in `[0.4, 0.9]` |
    /// | e | `phase_offset`    | `f32` in `[0, TAU]`   |
    /// | f | `amplitude`       | `f32` in `[0.0, 1.0]` |
    /// | g | `clock_speed`     | `f32` in `[0.5, 4.0]` (only from segment 0) |
    ///
    /// **Particle layout**
    /// - Index `0`: central body at `(0, 1)`.
    /// - Index `1 + 2*i`: left limb of segment `i` at `(-limb_length, 1 - i*0.3)`.
    /// - Index `2 + 2*i`: right limb of segment `i` at `(+limb_length, 1 - i*0.3)`.
    ///
    /// **Constraint layout per segment**
    /// - Bone: body → left, body → right (structural).
    /// - Muscle: left → right (bilateral oscillating muscle).
    /// - Spine bone: left[i] → left[i+1], right[i] → right[i+1] (rigidity).
    ///
    /// # Errors
    ///
    /// Returns `Err` if `genome.len() < Self::min_genome_len()`.
    pub fn build(genome: &Genome) -> Result<Creature, String> {
        if genome.len() < Self::min_genome_len() {
            return Err(format!(
                "genome too short: need at least {} genes, got {}",
                Self::min_genome_len(),
                genome.len(),
            ));
        }

        let mut reader = GeneReader::new(genome.genes());

        // ── Header ──────────────────────────────────────────────────────────
        let num_segments = reader.next_usize(1, MAX_SEGMENTS);
        let body_mass = reader.next_range(0.5, 3.0);

        // ── Per-segment gene decoding ────────────────────────────────────────
        // We always consume all MAX_SEGMENTS × GENES_PER_SEGMENT genes to
        // keep the genome layout fixed, but only store `num_segments` params.
        let mut segments: Vec<SegmentParams> = Vec::with_capacity(num_segments);
        let mut clock_speed = 1.0_f32;

        for i in 0..MAX_SEGMENTS {
            let limb_length = reader.next_range(0.2, 1.5);
            let limb_mass = reader.next_range(0.1, 1.0);
            let muscle_stiffness = reader.next_range(0.3, 1.0);
            let muscle_min_ratio = reader.next_range(0.4, 0.9);
            let phase_offset = reader.next_range(0.0, TAU);
            let amplitude = reader.next_range(0.0, 1.0);
            let cs = reader.next_range(0.5, 4.0);

            if i == 0 {
                clock_speed = cs;
            }

            if i < num_segments {
                segments.push(SegmentParams {
                    limb_length,
                    limb_mass,
                    muscle_stiffness,
                    muscle_min_ratio,
                    phase_offset,
                    amplitude,
                });
            }
        }

        // ── Build particles ──────────────────────────────────────────────────
        // [0]        : central body
        // [1 + 2*i]  : left limb of segment i
        // [2 + 2*i]  : right limb of segment i
        let mut particles: Vec<Particle> = Vec::with_capacity(1 + 2 * num_segments);

        particles.push(Particle::new(0.0, 1.0, body_mass));

        for (i, seg) in segments.iter().enumerate() {
            let y = 1.0 - i as f32 * 0.3;
            particles.push(Particle::new(-seg.limb_length, y, seg.limb_mass));
            particles.push(Particle::new(seg.limb_length, y, seg.limb_mass));
        }

        // ── Build muscles / bones ────────────────────────────────────────────
        let mut muscles: Vec<Muscle> = Vec::new();

        for (i, seg) in segments.iter().enumerate() {
            let body = 0_usize;
            let left = 1 + 2 * i;
            let right = 2 + 2 * i;

            let bpos = particles[body].pos;
            let lpos = particles[left].pos;
            let rpos = particles[right].pos;

            // Structural bones: body → left, body → right
            muscles.push(Muscle::bone(body, left, (lpos - bpos).length()));
            muscles.push(Muscle::bone(body, right, (rpos - bpos).length()));

            // Bilateral oscillating muscle: left → right
            let rest_len = (rpos - lpos).length();
            let min_len = rest_len * seg.muscle_min_ratio;
            muscles.push(Muscle::muscle(
                left,
                right,
                rest_len,
                min_len,
                seg.muscle_stiffness,
                seg.phase_offset,
                seg.amplitude,
            ));

            // Spine bones between consecutive segment endpoints
            if i + 1 < num_segments {
                let next_left = 1 + 2 * (i + 1);
                let next_right = 2 + 2 * (i + 1);

                let nl_pos = particles[next_left].pos;
                let nr_pos = particles[next_right].pos;

                muscles.push(Muscle::bone(left, next_left, (nl_pos - lpos).length()));
                muscles.push(Muscle::bone(right, next_right, (nr_pos - rpos).length()));
            }
        }

        let mut creature = Creature::new(particles, muscles);
        creature.clock_speed = clock_speed;

        Ok(creature)
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::GROUND_Y;

    /// Returns a zero-filled genome of exactly `min_genome_len()` genes.
    fn min_genome() -> Genome {
        Genome::new(vec![0.0; CreatureFactory::min_genome_len()])
    }

    /// Returns a genome encoding `num_segments = 2` (gene[0] = -0.5).
    fn two_segment_genome() -> Genome {
        let mut genes = vec![0.0_f32; CreatureFactory::min_genome_len()];
        // next_usize(1, 4) with gene=-0.5:
        //   t = (-0.5+1)/2 = 0.25, range=4, idx=floor(1.0)=1, result=1+1=2
        genes[0] = -0.5;
        Genome::new(genes)
    }

    #[test]
    fn build_with_min_genome_succeeds() {
        let result = CreatureFactory::build(&min_genome());
        assert!(result.is_ok(), "expected Ok, got {:?}", result.err());
    }

    #[test]
    fn build_with_too_short_genome_returns_err() {
        let short = Genome::new(vec![0.0; CreatureFactory::min_genome_len() - 1]);
        let result = CreatureFactory::build(&short);
        assert!(result.is_err(), "expected Err for short genome");
    }

    #[test]
    fn two_segment_creature_has_correct_particle_count() {
        let creature = CreatureFactory::build(&two_segment_genome())
            .expect("build should succeed with two-segment genome");
        // 1 body + 2 limbs × 2 segments = 5
        assert_eq!(
            creature.particles.len(),
            5,
            "expected 1 + 2*2 = 5 particles, got {}",
            creature.particles.len()
        );
    }

    #[test]
    fn step_100_times_does_not_panic() {
        let mut creature = CreatureFactory::build(&min_genome())
            .expect("build should succeed with min genome");
        for _ in 0..100 {
            creature.step(0.016);
        }
        for p in &creature.particles {
            assert!(p.pos.x.is_finite(), "particle x is not finite");
            assert!(p.pos.y.is_finite(), "particle y is not finite");
        }
    }

    #[test]
    fn center_of_mass_starts_above_ground() {
        let creature = CreatureFactory::build(&min_genome())
            .expect("build should succeed with min genome");
        let com = creature.center_of_mass();
        assert!(
            com.y > GROUND_Y,
            "center of mass y ({}) should be above GROUND_Y ({})",
            com.y,
            GROUND_Y,
        );
    }
}
