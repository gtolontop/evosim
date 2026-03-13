use rand::Rng;

/// A variable-length genome represented as a vector of floating-point genes.
///
/// Each gene is a plain `f32` value whose meaning is determined by the
/// decoder that maps genomes to creature morphologies.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Genome {
    genes: Vec<f32>,
}

impl Genome {
    /// Creates a new genome from the given gene values.
    pub fn new(genes: Vec<f32>) -> Self {
        Self { genes }
    }

    /// Generates a random genome with genes uniformly distributed in [-1.0, 1.0].
    ///
    /// # Arguments
    ///
    /// * `len` — The number of genes in the genome.
    /// * `rng` — The random number generator to use.
    ///
    /// # Returns
    ///
    /// A new [`Genome`] with random genes.
    pub fn random(len: usize, rng: &mut impl Rng) -> Self {
        let genes = (0..len)
            .map(|_| rng.gen_range(-1.0..=1.0))
            .collect();
        Self { genes }
    }

    /// Returns the number of genes in this genome.
    pub fn len(&self) -> usize {
        self.genes.len()
    }

    /// Returns `true` if the genome contains no genes.
    pub fn is_empty(&self) -> bool {
        self.genes.is_empty()
    }

    /// Returns the gene value at the given index.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds.
    pub fn get(&self, i: usize) -> f32 {
        self.genes[i]
    }

    /// Returns a slice over all gene values.
    pub fn genes(&self) -> &[f32] {
        &self.genes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn test_random_genome_in_range() {
        let mut rng = StdRng::seed_from_u64(42);
        let genome = Genome::random(100, &mut rng);

        assert_eq!(genome.len(), 100);
        for gene in genome.genes() {
            assert!(*gene >= -1.0 && *gene <= 1.0, "gene {} out of range", gene);
        }
    }

    #[test]
    fn test_get_returns_gene() {
        let genome = Genome::new(vec![0.5, -0.3, 0.7]);
        assert_eq!(genome.get(0), 0.5);
        assert_eq!(genome.get(1), -0.3);
        assert_eq!(genome.get(2), 0.7);
    }

    #[test]
    #[should_panic]
    fn test_get_panics_on_out_of_bounds() {
        let genome = Genome::new(vec![0.5, -0.3]);
        let _ = genome.get(5);
    }

    #[test]
    fn test_genes_returns_slice() {
        let genes_vec = vec![0.1, 0.2, 0.3];
        let genome = Genome::new(genes_vec.clone());
        assert_eq!(genome.genes(), &genes_vec);
    }
}
