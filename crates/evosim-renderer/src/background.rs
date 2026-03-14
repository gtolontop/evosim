use bevy::prelude::*;

/// Draws background: altitude grid lines, a thick tree trunk at x=0, and a ground marker.
pub fn draw_background(
    mut gizmos: Gizmos,
    camera_q: Query<&Transform, With<Camera2d>>,
) {
    let Ok(cam_tr) = camera_q.get_single() else { return; };
    let cam_pos = cam_tr.translation.truncate();

    let x0 = cam_pos.x - 25.0;
    let x1 = cam_pos.x + 25.0;
    let y0 = cam_pos.y - 16.0;
    let y1 = cam_pos.y + 16.0;

    // Altitude marker lines every 5 world-units
    let alt_min = ((y0 / 5.0).floor() as i32) * 5;
    let alt_max = ((y1 / 5.0).ceil()  as i32) * 5;
    for iy in (alt_min..=alt_max).step_by(5) {
        if iy == 0 { continue; }
        let y = iy as f32;
        gizmos.line_2d(Vec2::new(x0, y), Vec2::new(x1, y),
            Color::srgba(0.4, 0.55, 0.75, 0.10));
    }

    // Ground line — thick green
    for offset in [-0.02_f32, 0.0, 0.02] {
        gizmos.line_2d(
            Vec2::new(x0, offset),
            Vec2::new(x1, offset),
            Color::srgba(0.35, 0.65, 0.35, 0.5),
        );
    }

    // ── Thick tree trunk at x = 0 ───────────────────────────────────────────
    // Fill a wide rectangle with many parallel vertical lines.
    let trunk_hw = 0.6_f32; // half-width — 1.2 world units total
    let steps = 24_i32;     // lines to fill the trunk

    for i in 0..=steps {
        let t = i as f32 / steps as f32;   // 0.0 to 1.0
        let x = -trunk_hw + t * (2.0 * trunk_hw);

        // Edge-to-center gradient: edges are darker, center is lighter
        let center_dist = (t - 0.5).abs() * 2.0; // 0 = center, 1 = edge
        let r = 0.38 + (1.0 - center_dist) * 0.15;
        let g = 0.22 + (1.0 - center_dist) * 0.08;
        let b = 0.07 + (1.0 - center_dist) * 0.03;
        let a = 1.0 - center_dist * 0.3;

        gizmos.line_2d(Vec2::new(x, y0), Vec2::new(x, y1), Color::srgba(r, g, b, a));
    }

    // Bark notches every 2 world-units for depth texture
    let notch_min = ((y0 / 2.0).floor() as i32) * 2;
    let notch_max = ((y1 / 2.0).ceil()  as i32) * 2;
    for iy in (notch_min..=notch_max).step_by(2) {
        let y = iy as f32;
        gizmos.line_2d(
            Vec2::new(-trunk_hw * 0.8, y),
            Vec2::new(trunk_hw * 0.8, y),
            Color::srgba(0.25, 0.14, 0.04, 0.35),
        );
        // Slightly offset secondary notch for texture
        gizmos.line_2d(
            Vec2::new(-trunk_hw * 0.5, y + 0.3),
            Vec2::new(trunk_hw * 0.5, y + 0.3),
            Color::srgba(0.28, 0.16, 0.05, 0.2),
        );
    }
}
