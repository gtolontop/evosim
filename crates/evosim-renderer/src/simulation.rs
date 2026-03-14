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
    /// Maximum physics steps before auto-pause (matches headless evaluator).
    pub max_steps: u32,
    pub generation: u32,
    /// Live runtime fitness — updated every frame from `creature.fitness`.
    pub fitness: f32,
    /// Champion's recorded fitness from the JSON file — never modified after startup.
    pub champion_fitness: f32,
    /// When true, overlay particle indices, activation values, and velocity arrows.
    pub debug_mode: bool,
    /// Accumulated time not yet consumed by fixed-step physics.
    pub time_accumulator: f32,
    /// True once the evaluation window is finished.
    pub evaluation_done: bool,
}

/// Fixed physics dt — must match the headless evolution (config.rs default).
const PHYSICS_DT: f32 = 0.016;
/// Maximum frame time to accumulate — prevents spiral-of-death.
const MAX_FRAME_DT: f32 = 0.1;

/// Advances the creature physics each frame using a fixed-timestep accumulator.
///
/// This ensures identical behaviour to the headless evaluator regardless of
/// frame rate or first-frame spikes.
pub fn simulation_step_system(mut state: ResMut<SimulationState>, time: Res<Time>) {
    if state.paused {
        return;
    }
    let frame_dt = time.delta_secs().min(MAX_FRAME_DT) * state.speed_multiplier;
    state.time_accumulator += frame_dt;

    while state.time_accumulator >= PHYSICS_DT {
        if state.step_count >= state.max_steps {
            state.paused = true;
            state.evaluation_done = true;
            state.time_accumulator = 0.0;
            break;
        }
        state.creature.step(PHYSICS_DT);
        state.step_count += 1;
        state.time_accumulator -= PHYSICS_DT;
    }
    state.fitness = state.creature.fitness;
}
