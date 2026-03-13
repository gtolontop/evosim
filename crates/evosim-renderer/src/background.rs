use bevy::prelude::*;

/// Draws a subtle dot grid and a glowing ground line every frame.
///
/// The grid is clamped to the visible camera frustum
/// (±20 world units wide, ±12 world units tall).
pub fn draw_background_grid(
    mut gizmos: Gizmos,
    camera_q: Query<&Transform, With<Camera2d>>,
) {
    let Ok(cam_tr) = camera_q.get_single() else {
        return;
    };
    let cam_pos = cam_tr.translation.truncate();

    // Dot grid — integer world-unit spacing within the visible frustum.
    let min_x = (cam_pos.x - 20.0).floor() as i32;
    let max_x = (cam_pos.x + 20.0).ceil() as i32;
    let min_y = (cam_pos.y - 12.0).floor() as i32;
    let max_y = (cam_pos.y + 12.0).ceil() as i32;

    let dot_color = Color::srgba(0.15, 0.15, 0.2, 0.4);
    for ix in min_x..=max_x {
        for iy in min_y..=max_y {
            gizmos.circle_2d(Vec2::new(ix as f32, iy as f32), 0.03, dot_color);
        }
    }

    // Ground line at y = 0.
    let x0 = cam_pos.x - 21.0;
    let x1 = cam_pos.x + 21.0;

    // Main line — solid blue-white.
    gizmos.line_2d(
        Vec2::new(x0, 0.0),
        Vec2::new(x1, 0.0),
        Color::srgba(0.4, 0.6, 1.0, 0.5),
    );
    // Soft glow duplicate — same path, lower alpha.
    gizmos.line_2d(
        Vec2::new(x0, 0.0),
        Vec2::new(x1, 0.0),
        Color::srgba(0.4, 0.6, 1.0, 0.15),
    );
}
