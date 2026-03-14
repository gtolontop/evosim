use bevy::prelude::*;
use evosim_core::CreatureFactory;
use evosim_genetics::Genome;

use crate::{
    background::{draw_background, setup_trunk},
    camera::{camera_follow_system, setup_camera},
    hud::{setup_hud, update_hud_system},
    input::{input_system, StoredGenome},
    render::{render_creature_system, setup_render_cache, MuscleRenderCache},
    simulation::{simulation_step_system, SimulationState},
    trails::{trail_render_system, trail_update_system, TrailBuffer},
};

/// Builds and runs the Bevy app, blocking until the window is closed.
///
/// # Errors (printed, then process continues)
/// If the genome cannot produce a valid creature, a descriptive message is
/// printed and the function returns immediately.
pub fn run_renderer(genome: &Genome, generation: u32, fitness: f32) {
    let creature = match CreatureFactory::build(genome) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: could not build creature from genome: {e}");
            return;
        }
    };

    let title = format!("evosim — gen {generation:04} — fitness {fitness:.3}");

    App::new()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title,
                    ..default()
                }),
                ..default()
            }),
        )
        // Deep dusk sky — creatures glow nicely against it with bloom
        .insert_resource(ClearColor(Color::srgb(0.04, 0.09, 0.20)))
        .insert_resource(SimulationState {
            creature,
            paused: false,
            speed_multiplier: 1.0,
            step_count: 0,
            max_steps: 600,
            generation,
            fitness: 0.0,
            champion_fitness: fitness,
            debug_mode: false,
            time_accumulator: 0.0,
            evaluation_done: false,
        })
        .insert_resource(StoredGenome(genome.clone()))
        .insert_resource(MuscleRenderCache::default())
        .insert_resource(TrailBuffer::default())
        .add_systems(Startup, (setup_camera, setup_hud, setup_render_cache, setup_trunk))
        .add_systems(
            Update,
            (
                input_system,
                simulation_step_system,
                trail_update_system,
                render_creature_system,
                draw_background,
                trail_render_system,
                camera_follow_system,
                update_hud_system,
            )
                .chain(),
        )
        .run();
}
