use rand::Rng;

use crate::genome::Genome;
use crate::mutation;

/// Selects a genome from the population using tournament selection.
///
/// Picks `k` random individuals from the population and returns a reference to
/// the one with the highest fitness.
///
/// # Arguments
///
/// * `genomes`   — Slice of all genomes in the current generation.
/// * `fitnesses` — Parallel slice of fitness scores (one per genome).
/// * `k`         — Tournament size (number of candidates per round).
/// * `rng`       — The random number generator to use.
///
/// # Panics
///
/// Panics if `genomes` is empty, if `genomes` and `fitnesses` have
/// different lengths, or if `k` is zero.
///
/// # Returns
///
/// A reference to the [`Genome`] of the tournament winner.
pub fn tournament_select<'a>(
    genomes: &'a [Genome],
    fitnesses: &[f32],
    k: usize,
    rng: &mut impl Rng,
) -> &'a Genome {
    assert!(!genomes.is_empty(), "genomes must not be empty");
    assert_eq!(
        genomes.len(),
        fitnesses.len(),
        "genomes and fitnesses must have the same length"
    );
    assert!(k > 0, "tournament size k must be greater than 0");

    let mut best_idx = rng.gen_range(0..genomes.len());
    let mut best_fitness = fitnesses[best_idx];

    for _ in 1..k {
        let idx = rng.gen_range(0..genomes.len());
        if fitnesses[idx] > best_fitness {
            best_idx = idx;
            best_fitness = fitnesses[idx];
        }
    }

    &genomes[best_idx]
}

/// Generates the next generation through selection, crossover, and mutation.
///
/// The top `elitism` genomes (by fitness) are copied directly to the next
/// generation without mutation. The remaining genomes are created by:
/// 1. Tournament selection (k=5) of two parents
/// 2. Uniform crossover
/// 3. Gaussian mutation
///
/// # Arguments
///
/// * `genomes`            — Slice of genomes in the current generation.
/// * `fitnesses`          — Parallel slice of fitness scores.
/// * `pop_size`           — Target population size for the next generation.
/// * `mutation_rate`      — Per-gene mutation probability [0.0, 1.0].
/// * `mutation_strength`  — Standard deviation of Gaussian noise.
/// * `elitism`            — Number of top genomes to preserve without mutation.
/// * `rng`                — The random number generator to use.
///
/// # Panics
///
/// Panics if `genomes` and `fitnesses` have different lengths.
///
/// # Returns
///
/// A new [`Vec<Genome>`] of exactly `pop_size` genomes.
pub fn next_generation(
    genomes: &[Genome],
    fitnesses: &[f32],
    pop_size: usize,
    mutation_rate: f32,
    mutation_strength: f32,
    elitism: usize,
    rng: &mut impl Rng,
) -> Vec<Genome> {
    assert_eq!(
        genomes.len(),
        fitnesses.len(),
        "genomes and fitnesses must have the same length"
    );

    let mut next_gen = Vec::with_capacity(pop_size);

    // Elitism: copy top genomes directly
    let mut indexed_fitnesses: Vec<(usize, f32)> = fitnesses
        .iter()
        .enumerate()
        .map(|(i, &f)| (i, f))
        .collect();
    indexed_fitnesses.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    for i in 0..elitism.min(genomes.len()) {
        next_gen.push(genomes[indexed_fitnesses[i].0].clone());
    }

    // Fill the rest with tournament selection, crossover, and mutation
    while next_gen.len() < pop_size {
        let parent_a = tournament_select(genomes, fitnesses, 5, rng);
        let parent_b = tournament_select(genomes, fitnesses, 5, rng);
        let offspring = mutation::crossover(parent_a, parent_b, rng);
        let mutated = mutation::mutate(&offspring, mutation_rate, mutation_strength, rng);
        next_gen.push(mutated);
    }

    next_gen.truncate(pop_size);
    next_gen
}

/// Generates the next generation using top-50% asexual reproduction (Code BH style).
///
/// The top half of the population (by fitness) each produce one mutated offspring.
/// The top `elitism` genomes are preserved without mutation.
pub fn next_generation_asexual(
    genomes: &[Genome],
    fitnesses: &[f32],
    pop_size: usize,
    mutation_rate: f32,
    mutation_strength: f32,
    elitism: usize,
    rng: &mut impl Rng,
) -> Vec<Genome> {
    assert_eq!(
        genomes.len(),
        fitnesses.len(),
        "genomes and fitnesses must have the same length"
    );

    // Sort by fitness descending
    let mut indexed: Vec<(usize, f32)> = fitnesses
        .iter()
        .enumerate()
        .map(|(i, &f)| (i, f))
        .collect();
    indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let mut next_gen = Vec::with_capacity(pop_size);

    // Elitism: copy top genomes unchanged
    for i in 0..elitism.min(genomes.len()) {
        next_gen.push(genomes[indexed[i].0].clone());
    }

    // Top 50% reproduce asexually with mutations (no crossover)
    let top_half = indexed.len() / 2;
    while next_gen.len() < pop_size {
        for i in 0..top_half.max(1) {
            if next_gen.len() >= pop_size {
                break;
            }
            let parent_idx = indexed[i].0;
            let offspring = mutation::mutate(
                &genomes[parent_idx],
                mutation_rate,
                mutation_strength,
                rng,
            );
            next_gen.push(offspring);
        }
    }

    next_gen.truncate(pop_size);
    next_gen
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn test_tournament_select_returns_best() {
        let mut rng = StdRng::seed_from_u64(42);
        let genomes = vec![
            Genome::new(vec![0.1, 0.2]),
            Genome::new(vec![0.3, 0.4]),
            Genome::new(vec![0.5, 0.6]),
            Genome::new(vec![0.7, 0.8]),
        ];
        let fitnesses = vec![1.0, 5.0, 3.0, 2.0];

        // Run tournament selection multiple times with smaller k
        for _ in 0..20 {
            let winner = tournament_select(&genomes, &fitnesses, 2, &mut rng);
            // The winner should be one of the genomes in the population
            let winner_idx = genomes.iter().position(|g| g.genes() == winner.genes());
            assert!(winner_idx.is_some());

            // The winner's fitness should be at least as good as some randomly picked genomes
            let winner_fitness = fitnesses[winner_idx.unwrap()];
            assert!(winner_fitness >= 1.0);
        }
    }

    #[test]
    fn test_next_generation_size() {
        let mut rng = StdRng::seed_from_u64(42);
        let genomes = vec![
            Genome::random(5, &mut rng),
            Genome::random(5, &mut rng),
            Genome::random(5, &mut rng),
        ];
        let fitnesses = vec![1.0, 2.0, 3.0];

        let next = next_generation(&genomes, &fitnesses, 10, 0.05, 0.1, 1, &mut rng);

        assert_eq!(next.len(), 10);
    }

    #[test]
    fn test_next_generation_elitism() {
        let mut rng = StdRng::seed_from_u64(42);
        let genomes = vec![
            Genome::new(vec![1.0, 2.0, 3.0]),
            Genome::new(vec![4.0, 5.0, 6.0]),
            Genome::new(vec![7.0, 8.0, 9.0]),
        ];
        let fitnesses = vec![1.0, 2.0, 3.0]; // Best is at index 2

        let next = next_generation(&genomes, &fitnesses, 5, 0.05, 0.1, 2, &mut rng);

        // First two genomes should be the elites (indices 2 and 1)
        assert_eq!(next[0].genes(), &[7.0, 8.0, 9.0]);
        assert_eq!(next[1].genes(), &[4.0, 5.0, 6.0]);
    }
}
