use bevy::prelude::*;
use bevy::core_pipeline::bloom::Bloom;

use crate::simulation::SimulationState;

/// Spawns the 2D camera with bloom effects enabled.
///
/// Zoom is set so that 1 world unit ≈ 80 pixels.
pub fn setup_camera(mut commands: Commands) {
    let mut projection = OrthographicProjection::default_2d();
    projection.scale = 1.0 / 40.0;  // zoom out for 15-particle humanoid

    let bloom = Bloom {
        intensity: 0.3,
        ..default()
    };

    commands.spawn((Camera2d, projection, bloom));
}

/// Smoothly follows the creature upward.
///
/// X is fixed at 1.5 so the trunk (x=0) stays visible to the left while the
/// creature body (x ≈ 1.0–1.8) is always in frame.  Only Y tracks the
/// creature's centre of mass.
pub fn camera_follow_system(
    state: Res<SimulationState>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
) {
    let com = state.creature.center_of_mass();
    let target = Vec2::new(1.0, com.y + 1.0);

    for mut transform in &mut camera_query {
        let current = transform.translation.truncate();
        let smoothed = current.lerp(target, 0.05);
        transform.translation.x = smoothed.x;
        transform.translation.y = smoothed.y;
    }
}
