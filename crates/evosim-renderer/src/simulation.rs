use bevy::prelude::*;
use evosim_core::Creature;

/// Central simulation state held as a Bevy resource.
#[derive(Resource)]
pub struct SimulationState {
    pub creature: Creature,
    pub paused: bool,
    /// Playback speed multiplier, clamped to [0.1, 8.0].
    pub speed_multiplier: f32,
    pub step_count: u32,
    pub generation: u32,
    /// Live runtime fitness — updated every frame from `creature.fitness`.
    pub fitness: f32,
    /// Champion's recorded fitness from the JSON file — never modified after startup.
    pub champion_fitness: f32,
    /// When true, overlay particle indices, activation values, and velocity arrows.
    pub debug_mode: bool,
}

/// Advances the creature physics each frame unless paused.
pub fn simulation_step_system(mut state: ResMut<SimulationState>, time: Res<Time>) {
    if state.paused {
        return;
    }
    let dt = time.delta_secs() * state.speed_multiplier;
    state.creature.step(dt);
    state.step_count += 1;
    state.fitness = state.creature.fitness;
}
