//! Serialisation of champion genomes to disk.

use evosim_genetics::Genome;
use serde::Serialize;

#[derive(Serialize)]
struct ChampionRecord<'a> {
    generation: u32,
    fitness: f32,
    genes: &'a [f32],
}

/// Saves the best genome of a generation to `{dir}/gen_{generation:04}.json`.
///
/// The JSON object has three fields:
/// ```json
/// { "generation": 42, "fitness": 12.847, "genes": [...] }
/// ```
///
/// The output directory is created if it does not exist. IO errors are
/// silently swallowed and reported as warnings on stderr.
pub fn save_champion(genome: &Genome, generation: u32, fitness: f32, dir: &str) {
    if let Err(e) = std::fs::create_dir_all(dir) {
        eprintln!("warning: could not create champion dir '{dir}': {e}");
        return;
    }

    let path = format!("{dir}/gen_{generation:04}.json");
    let record = ChampionRecord {
        generation,
        fitness,
        genes: genome.genes(),
    };

    match serde_json::to_string_pretty(&record) {
        Ok(json) => {
            if let Err(e) = std::fs::write(&path, json) {
                eprintln!("warning: could not write champion to '{path}': {e}");
            }
        }
        Err(e) => {
            eprintln!("warning: could not serialise champion for gen {generation}: {e}");
        }
    }
}
