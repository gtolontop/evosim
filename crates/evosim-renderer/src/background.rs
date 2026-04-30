use bevy::prelude::*;

#[derive(Component)]
pub(crate) struct TrunkMarker;

/// Spawns a subtle wall surface behind the creature at z = -10.
/// Thin vertical strip — the creature climbs along this surface.
pub fn setup_trunk(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Thin wall surface — just a subtle backdrop, not a prominent trunk.
    let wall_width = 1.2_f32;
    let wall_height = 500.0_f32;
    let wall_center_x = 0.0_f32;
    let mesh = meshes.add(Rectangle::new(wall_width, wall_height));
    let mat = materials.add(ColorMaterial {
        color: Color::srgba(0.25, 0.25, 0.28, 0.3),
        ..default()
    });
    commands.spawn((
        Mesh2d(mesh),
        MeshMaterial2d(mat),
        Transform::from_translation(Vec3::new(wall_center_x, wall_height / 2.0, -10.0)),
        TrunkMarker,
    ));
}

/// Draws background: altitude grid lines and a ground marker.
/// The tree trunk is handled by `setup_trunk` (mesh entity).
pub fn draw_background(
    mut gizmos: Gizmos,
    camera_q: Query<&Transform, (With<Camera2d>, Without<TrunkMarker>)>,
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
}
