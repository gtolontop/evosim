use bevy::prelude::*;
use evosim_core::CreatureFactory;
use evosim_genetics::Genome;

use crate::simulation::SimulationState;

/// Bevy resource holding the original genome so the simulation can be reset.
#[derive(Resource)]
pub struct StoredGenome(pub Genome);

/// Handles keyboard input each frame.
///
/// - `Space`      → toggle paused
/// - `ArrowUp`    → speed × 2 (max 8.0)
/// - `ArrowDown`  → speed ÷ 2 (min 0.1)
/// - `R`          → rebuild creature from stored genome
/// - `D`          → toggle debug mode (particle indices, velocity arrows)
pub fn input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<SimulationState>,
    genome: Res<StoredGenome>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        state.paused = !state.paused;
    }

    if keyboard.just_pressed(KeyCode::ArrowUp) {
        state.speed_multiplier = (state.speed_multiplier * 2.0).min(8.0);
    }

    if keyboard.just_pressed(KeyCode::ArrowDown) {
        state.speed_multiplier = (state.speed_multiplier / 2.0).max(0.1);
    }

    if keyboard.just_pressed(KeyCode::KeyR) {
        if let Ok(creature) = CreatureFactory::build(&genome.0) {
            state.creature = creature;
            state.step_count = 0;
            state.fitness = 0.0;
        }
    }

    if keyboard.just_pressed(KeyCode::KeyD) {
        state.debug_mode = !state.debug_mode;
    }
}
