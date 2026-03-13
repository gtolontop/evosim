use std::collections::VecDeque;

use bevy::prelude::*;

use crate::simulation::SimulationState;

/// Ring buffer of the creature's last N centre-of-mass positions.
#[derive(Resource)]
pub struct TrailBuffer {
    pub positions: VecDeque<Vec2>,
    pub max_len: usize,
}

impl Default for TrailBuffer {
    fn default() -> Self {
        Self {
            positions: VecDeque::new(),
            max_len: 30,
        }
    }
}

/// Pushes the current COM into the trail buffer, evicting the oldest entry
/// when the buffer is full.  Skipped while paused so the trail stays stable.
pub fn trail_update_system(state: Res<SimulationState>, mut trail: ResMut<TrailBuffer>) {
    if state.paused {
        return;
    }
    let com = state.creature.center_of_mass();
    trail.positions.push_back(com);
    if trail.positions.len() > trail.max_len {
        trail.positions.pop_front();
    }
}

/// Draws the trail as fading dots using Gizmos.
///
/// Newest sample → white, alpha 0.6.
/// Oldest sample → white, alpha 0.0.
pub fn trail_render_system(trail: Res<TrailBuffer>, mut gizmos: Gizmos) {
    let n = trail.positions.len();
    if n < 2 {
        return;
    }
    for (i, &pos) in trail.positions.iter().enumerate() {
        let t = i as f32 / (n - 1) as f32; // 0 = oldest, 1 = newest
        let alpha = t * 0.6;
        gizmos.circle_2d(pos, 0.04, Color::srgba(1.0, 1.0, 1.0, alpha));
    }
}
