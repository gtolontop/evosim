use rand::Rng;
use rand_distr::Normal;

use crate::genome::Genome;

/// Per-gene probability of a macro-mutation (complete randomisation).
const MACRO_MUTATION_RATE: f32 = 0.02;

/// Produces a mutated copy of the given genome.
///
/// Each gene can be affected by:
/// 1. **Macro-mutation** (2% chance): completely randomised to [-1, 1]
/// 2. **Normal mutation** (`rate` chance): Gaussian noise added
///
/// Macro-mutations create occasional large jumps that can unlock entirely
/// new body plans or movement strategies.
pub fn mutate(genome: &Genome, rate: f32, strength: f32, rng: &mut impl Rng) -> Genome {
    let normal = Normal::new(0.0, strength)
        .expect("failed to create normal distribution for mutation");

    let mutated_genes = genome
        .genes()
        .iter()
        .map(|&gene| {
            if rng.gen::<f32>() < MACRO_MUTATION_RATE {
                rng.gen_range(-1.0_f32..=1.0)
            } else if rng.gen::<f32>() < rate {
                let noise: f32 = rng.sample(normal);
                (gene + noise).clamp(-1.0, 1.0)
            } else {
                gene
            }
        })
        .collect();

    Genome::new(mutated_genes)
}

/// Produces an offspring genome through uniform crossover.
///
/// Each gene of the offspring is randomly selected from either parent with
/// equal probability.
///
/// # Arguments
///
/// * `a`   — The first parent genome.
/// * `b`   — The second parent genome.
/// * `rng` — The random number generator to use.
///
/// # Panics
///
/// Panics if the two parent genomes have different lengths.
///
/// # Returns
///
/// A new [`Genome`] that is a blend of both parents.
pub fn crossover(a: &Genome, b: &Genome, rng: &mut impl Rng) -> Genome {
    assert_eq!(
        a.len(),
        b.len(),
        "parent genomes must have the same length for crossover"
    );

    let offspring_genes = a
        .genes()
        .iter()
        .zip(b.genes().iter())
        .map(|(&gene_a, &gene_b)| {
            if rng.gen_bool(0.5) {
                gene_a
            } else {
                gene_b
            }
        })
        .collect();

    Genome::new(offspring_genes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn test_mutate_with_rate_one_differs() {
        let mut rng = StdRng::seed_from_u64(42);
        let genome = Genome::random(10, &mut rng);
        let mutated = mutate(&genome, 1.0, 0.1, &mut rng);

        // With rate=1.0, every gene should be mutated, so genomes should differ
        assert_ne!(genome.genes(), mutated.genes());
    }

    #[test]
    fn test_crossover_produces_blend() {
        let mut rng = StdRng::seed_from_u64(42);
        let parent_a = Genome::new(vec![1.0, 1.0, 1.0, 1.0]);
        let parent_b = Genome::new(vec![-1.0, -1.0, -1.0, -1.0]);

        let offspring = crossover(&parent_a, &parent_b, &mut rng);

        // Each gene should come from one parent
        for gene in offspring.genes() {
            assert!(*gene == 1.0 || *gene == -1.0);
        }
    }

    #[test]
    fn test_crossover_length_mismatch_panics() {
        let mut rng = StdRng::seed_from_u64(42);
        let parent_a = Genome::new(vec![1.0, 1.0]);
        let parent_b = Genome::new(vec![-1.0, -1.0, -1.0]);

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            crossover(&parent_a, &parent_b, &mut rng);
        }));

        assert!(result.is_err());
    }
}
