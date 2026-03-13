use bevy::prelude::*;

use crate::simulation::SimulationState;

/// Stores floating particle effect positions and their lifetimes.
#[derive(Resource, Default)]
pub struct ParticleEffects {
    /// Vec of (position, lifetime_remaining)
    pub particles: Vec<(Vec2, f32)>,
}

impl ParticleEffects {
    const MAX_PARTICLES: usize = 200;
    const PARTICLE_LIFETIME: f32 = 0.5;
}

/// Updates particle effects — decays lifetime and spawns new effects for active muscles.
pub fn update_particle_effects_system(
    state: Res<SimulationState>,
    mut effects: ResMut<ParticleEffects>,
) {
    // Decay existing particles
    effects
        .particles
        .retain_mut(|(_, lifetime)| {
            *lifetime -= 0.016; // Approximate frame time
            *lifetime > 0.0
        });

    // Spawn new particles for active muscles
    for muscle in &state.creature.muscles {
        if muscle.is_bone || muscle.amplitude < 1e-6 {
            continue;
        }

        // Spawn particles proportional to muscle activation
        let activation = (muscle.current_activation / muscle.amplitude).clamp(0.0, 1.0);
        if activation > 0.3 && effects.particles.len() < ParticleEffects::MAX_PARTICLES {
            let pa = state.creature.particles[muscle.a].pos;
            let pb = state.creature.particles[muscle.b].pos;
            let mid = (pa + pb) * 0.5;

            // Add some randomness to spawn position
            let offset = Vec2::new(
                activation * 0.1 - 0.05,
                activation * 0.1 - 0.05,
            );

            effects.particles.push((
                mid + offset,
                ParticleEffects::PARTICLE_LIFETIME,
            ));
        }
    }
}

/// Renders particle effects as small glowing circles that fade out.
pub fn render_particle_effects_system(effects: Res<ParticleEffects>, mut gizmos: Gizmos) {
    for &(pos, lifetime) in &effects.particles {
        let age = 1.0 - (lifetime / ParticleEffects::PARTICLE_LIFETIME);
        let alpha = (1.0 - age) * 0.8;
        let size = 0.02 + age * 0.03;

        // Warm glow: yellow-orange
        gizmos.circle_2d(
            pos,
            size,
            Color::srgba(1.0, 0.7, 0.2, alpha),
        );
    }
}
