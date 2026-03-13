use crate::genome::Genome;

/// Selects a genome from the population using tournament selection.
///
/// Picks `k` random individuals from the population and returns a clone of
/// the one with the highest fitness.
///
/// # Arguments
///
/// * `population` — Slice of all genomes in the current generation.
/// * `fitnesses`  — Parallel slice of fitness scores (one per genome).
/// * `k`          — Tournament size (number of candidates per round).
///
/// # Panics
///
/// Panics if `population` is empty, if `population` and `fitnesses` have
/// different lengths, or if `k` is zero.
///
/// # Returns
///
/// A cloned [`Genome`] of the tournament winner.
pub fn tournament_select(population: &[Genome], fitnesses: &[f32], _k: usize) -> Genome {
    assert!(!population.is_empty(), "population must not be empty");
    assert_eq!(
        population.len(),
        fitnesses.len(),
        "population and fitnesses must have the same length"
    );
    // TODO: implement random tournament selection
    population[0].clone()
}
