//! Loading and displaying saved champion genomes.

use serde::{Deserialize, Serialize};

/// A champion genome record as stored in `gen_NNNN.json`.
#[derive(Debug, Serialize, Deserialize)]
pub struct ChampionRecord {
    /// Zero-based generation index at which this genome was saved.
    pub generation: u32,
    /// Fitness score of the champion at this generation.
    pub fitness: f32,
    /// Raw gene values in `[-1.0, 1.0]`.
    pub genes: Vec<f32>,
}

/// Loads all `gen_XXXX.json` files from `dir`, sorted ascending by generation.
///
/// Files that cannot be read or parsed are skipped with a warning on stderr.
/// Returns an empty `Vec` if the directory is missing or unreadable.
pub fn load_champions(dir: &str) -> Vec<ChampionRecord> {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("warning: could not read champions dir '{dir}': {e}");
            return Vec::new();
        }
    };

    let mut records: Vec<ChampionRecord> = entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if !name.starts_with("gen_") || !name.ends_with(".json") {
                return None;
            }
            let path = entry.path();
            let json = match std::fs::read_to_string(&path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("warning: could not read {}: {e}", path.display());
                    return None;
                }
            };
            match serde_json::from_str::<ChampionRecord>(&json) {
                Ok(r) => Some(r),
                Err(e) => {
                    eprintln!("warning: could not parse {}: {e}", path.display());
                    None
                }
            }
        })
        .collect();

    records.sort_by_key(|r| r.generation);
    records
}

/// Prints a summary table of all champion records.
///
/// Output format:
/// ```text
/// Gen   Fitness    Δ vs prev
/// 0000    0.412    —
/// 0001    1.203    +0.791
/// 0050   12.847   +11.644
/// ```
pub fn print_summary(records: &[ChampionRecord]) {
    println!("{:<6}  {:>10}  {:>12}", "Gen", "Fitness", "Δ vs prev");
    println!("{}", "─".repeat(32));

    let mut prev: Option<f32> = None;
    for rec in records {
        let delta = match prev {
            None => "—".to_string(),
            Some(p) => format!("{:+.3}", rec.fitness - p),
        };
        println!("{:04}  {:>10.3}  {:>12}", rec.generation, rec.fitness, delta);
        prev = Some(rec.fitness);
    }
}

/// Prints the single champion with the highest fitness across all generations.
///
/// Prints nothing if `records` is empty.
pub fn print_best(records: &[ChampionRecord]) {
    let best = records
        .iter()
        .max_by(|a, b| a.fitness.partial_cmp(&b.fitness).unwrap_or(std::cmp::Ordering::Equal));

    match best {
        None => println!("No champions found."),
        Some(r) => {
            println!("Best champion:");
            println!("  Generation : {:04}", r.generation);
            println!("  Fitness    : {:.3}", r.fitness);
            println!("  Genes ({})  : {:?}", r.genes.len(), r.genes);
        }
    }
}
