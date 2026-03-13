/// A variable-length genome represented as a vector of floating-point genes.
///
/// Each gene is a plain `f32` value whose meaning is determined by the
/// decoder that maps genomes to creature morphologies.
#[derive(Debug, Clone)]
pub struct Genome {
    genes: Vec<f32>,
}

impl Genome {
    /// Creates a new genome from the given gene values.
    pub fn new(genes: Vec<f32>) -> Self {
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

    /// Returns the gene value at the given index, or `None` if out of bounds.
    pub fn get(&self, i: usize) -> Option<f32> {
        self.genes.get(i).copied()
    }

    /// Returns a slice over all gene values.
    pub fn as_slice(&self) -> &[f32] {
        &self.genes
    }
}
