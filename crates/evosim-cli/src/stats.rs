//! Per-generation statistics and terminal reporting.

/// Summary statistics for one generation of evolution.
pub struct GenerationStats {
    /// Zero-based generation index.
    pub generation: u32,
    /// Fitness of the best individual.
    pub best_fitness: f32,
    /// Mean fitness across the population.
    pub mean_fitness: f32,
    /// Fitness of the worst individual.
    pub worst_fitness: f32,
    /// Population standard deviation of fitnesses.
    pub std_dev: f32,
    /// Wall-clock time taken to evaluate this generation (milliseconds).
    pub elapsed_ms: u64,
}

impl GenerationStats {
    /// Computes statistics from a slice of fitness values.
    ///
    /// Finite values are used for all aggregates; non-finite values
    /// (e.g. `f32::NEG_INFINITY` for failed builds) are included so that
    /// `worst_fitness` accurately reflects eliminated individuals.
    pub fn compute(generation: u32, fitnesses: &[f32], elapsed_ms: u64) -> Self {
        let best_fitness = fitnesses
            .iter()
            .copied()
            .fold(f32::NEG_INFINITY, f32::max);

        let worst_fitness = fitnesses
            .iter()
            .copied()
            .fold(f32::INFINITY, f32::min);

        // Mean over finite values only to avoid NaN propagation.
        let finite: Vec<f32> = fitnesses.iter().copied().filter(|f| f.is_finite()).collect();
        let mean_fitness = if finite.is_empty() {
            0.0
        } else {
            finite.iter().sum::<f32>() / finite.len() as f32
        };

        let std_dev = if finite.len() < 2 {
            0.0
        } else {
            let var = finite
                .iter()
                .map(|f| (f - mean_fitness).powi(2))
                .sum::<f32>()
                / finite.len() as f32;
            var.sqrt()
        };

        Self {
            generation,
            best_fitness,
            mean_fitness,
            worst_fitness,
            std_dev,
            elapsed_ms,
        }
    }

    /// Prints a single-line summary to stdout.
    ///
    /// Format:
    /// ```text
    /// [Gen 0042] best: 12.847  mean: 3.201  worst: -0.412  std: 2.341  time: 234ms
    /// ```
    pub fn print(&self) {
        println!(
            "[Gen {:04}] best: {:>8.3}  mean: {:>8.3}  worst: {:>8.3}  std: {:>6.3}  time: {}ms",
            self.generation,
            self.best_fitness,
            self.mean_fitness,
            self.worst_fitness,
            self.std_dev,
            self.elapsed_ms,
        );
    }

    /// Prints a milestone summary every 50 generations.
    ///
    /// Format:
    /// ```text
    /// ━━━ Milestone gen 50 ━━━ improvement: +847.3% since gen 0
    /// ```
    pub fn print_milestone(&self, baseline_best: f32) {
        let pct = if baseline_best.abs() < 1e-6 {
            0.0
        } else {
            (self.best_fitness - baseline_best) / baseline_best.abs() * 100.0
        };
        println!(
            "━━━ Milestone gen {} ━━━ improvement: {:+.1}% since gen 0",
            self.generation, pct
        );
    }
}
