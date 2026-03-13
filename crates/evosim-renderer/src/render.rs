use bevy::prelude::*;

use crate::simulation::SimulationState;

/// Draws the creature and ground using Bevy Gizmos each frame.
///
/// Drawing rules:
/// - Bones: thin white lines, alpha 0.6
/// - Muscles: colour interpolated from cool-blue (0.0) to hot-orange (1.0) by activation
/// - Particles: small circles, white alpha 0.9 (yellow if pinned)
/// - Ground: horizontal line at y = 0, white alpha 0.3
pub fn render_creature_system(state: Res<SimulationState>, mut gizmos: Gizmos) {
    let creature = &state.creature;

    // Ground line
    gizmos.line_2d(
        Vec2::new(-200.0, 0.0),
        Vec2::new(200.0, 0.0),
        Color::srgba(1.0, 1.0, 1.0, 0.3),
    );

    // Muscles and bones
    for muscle in &creature.muscles {
        let pa = creature.particles[muscle.a].pos;
        let pb = creature.particles[muscle.b].pos;

        if muscle.is_bone {
            gizmos.line_2d(pa, pb, Color::srgba(1.0, 1.0, 1.0, 0.6));
        } else {
            let t = if muscle.amplitude > 1e-6 {
                (muscle.current_activation / muscle.amplitude).clamp(0.0, 1.0)
            } else {
                0.0
            };
            // Interpolate cool-blue → hot-orange-red
            let r = 0.2 + t * 0.8;
            let g = 0.4 - t * 0.1;
            let b = 0.8 - t * 0.7;
            gizmos.line_2d(pa, pb, Color::srgb(r, g, b));
        }
    }

    // Particles
    for particle in &creature.particles {
        let color = if particle.pinned {
            Color::srgb(1.0, 1.0, 0.0)
        } else {
            Color::srgba(1.0, 1.0, 1.0, 0.9)
        };
        gizmos.circle_2d(particle.pos, 0.06, color);
    }
}
