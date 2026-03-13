use crate::genome::Genome;

/// Produces a mutated copy of the given genome.
///
/// Each gene has a probability of `rate` (0.0–1.0) of being perturbed.
/// The mutation strategy (e.g. Gaussian noise, uniform replacement) will be
/// implemented in a future version.
///
/// # Arguments
///
/// * `genome` — The source genome to mutate.
/// * `rate`   — Per-gene mutation probability in the range `[0.0, 1.0]`.
///
/// # Returns
///
/// A new [`Genome`] with mutations applied.
pub fn mutate(genome: &Genome, _rate: f32) -> Genome {
    // TODO: apply per-gene Gaussian perturbation
    genome.clone()
}
