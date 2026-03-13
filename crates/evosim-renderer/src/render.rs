use bevy::prelude::*;

use crate::simulation::SimulationState;

// ── Marker components ─────────────────────────────────────────────────────────

#[derive(Component)]
pub(crate) struct MuscleMarker;

#[derive(Component)]
pub(crate) struct ParticleMarker;

#[derive(Component)]
pub(crate) struct DebugLabelMarker;

// ── Cache resource ────────────────────────────────────────────────────────────

/// Holds mesh entity handles created at startup.
///
/// Transforms and material colours are updated in-place every frame —
/// entities are never despawned or re-spawned during playback.
#[derive(Resource, Default)]
pub struct MuscleRenderCache {
    /// One entry per `creature.muscles`.  `None` = bone (drawn as gizmo).
    pub muscle_entries: Vec<Option<(Entity, Handle<ColorMaterial>)>>,
    /// One entity per `creature.particles`.
    pub particle_entities: Vec<Entity>,
    /// One `Text2d` entity per particle (shown only in debug mode).
    pub debug_label_entities: Vec<Entity>,
}

// ── Setup (Startup) ───────────────────────────────────────────────────────────

/// Spawns mesh entities for every non-bone muscle and every particle.
/// Called once at Startup after `SimulationState` is available.
pub fn setup_render_cache(
    mut commands: Commands,
    state: Res<SimulationState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut cache: ResMut<MuscleRenderCache>,
) {
    let creature = &state.creature;

    // Unit quad — scaled per-muscle to (width, length) each frame.
    let quad_mesh = meshes.add(Rectangle::new(1.0, 1.0));

    for muscle in &creature.muscles {
        if muscle.is_bone {
            cache.muscle_entries.push(None);
        } else {
            let mat = materials.add(ColorMaterial {
                color: Color::linear_rgb(0.1, 0.3, 1.5),
                ..default()
            });
            let entity = commands
                .spawn((
                    Mesh2d(quad_mesh.clone()),
                    MeshMaterial2d(mat.clone()),
                    Transform::default(),
                    MuscleMarker,
                ))
                .id();
            cache.muscle_entries.push(Some((entity, mat)));
        }
    }

    // Shared circle mesh for all particles — emissive HDR white.
    let circle_mesh = meshes.add(Circle::new(0.06));
    let particle_mat = materials.add(ColorMaterial {
        color: Color::linear_rgb(1.5, 1.5, 1.5),
        ..default()
    });

    for (i, particle) in creature.particles.iter().enumerate() {
        let entity = commands
            .spawn((
                Mesh2d(circle_mesh.clone()),
                MeshMaterial2d(particle_mat.clone()),
                Transform::from_translation(Vec3::new(particle.pos.x, particle.pos.y, 1.0)),
                ParticleMarker,
            ))
            .id();
        cache.particle_entities.push(entity);

        // Debug index label — hidden by default.
        let label = commands
            .spawn((
                Text2d::new(format!("{i}")),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.2, 1.0, 0.2)),
                Transform::from_translation(Vec3::new(
                    particle.pos.x + 0.1,
                    particle.pos.y + 0.15,
                    5.0,
                )),
                Visibility::Hidden,
                DebugLabelMarker,
            ))
            .id();
        cache.debug_label_entities.push(label);
    }
}

// ── Per-frame update (Update) ─────────────────────────────────────────────────

/// Updates mesh transforms and material colours each frame to match the
/// creature's current physics state.
///
/// Bone muscles are drawn via Gizmos (no bloom).
/// Non-bone muscles use HDR emissive `ColorMaterial` — overbright values glow.
pub fn render_creature_system(
    state: Res<SimulationState>,
    cache: Res<MuscleRenderCache>,
    mut q_transforms: Query<
        &mut Transform,
        Or<(With<MuscleMarker>, With<ParticleMarker>, With<DebugLabelMarker>)>,
    >,
    mut q_vis: Query<&mut Visibility, With<DebugLabelMarker>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut gizmos: Gizmos,
) {
    let creature = &state.creature;
    let debug = state.debug_mode;

    // Bone muscles: thin white gizmo lines.
    for muscle in &creature.muscles {
        if muscle.is_bone {
            let pa = creature.particles[muscle.a].pos;
            let pb = creature.particles[muscle.b].pos;
            gizmos.line_2d(pa, pb, Color::srgba(1.0, 1.0, 1.0, 0.6));
        }
    }

    // Non-bone muscles: update mesh transform + emissive colour.
    for (i, muscle) in creature.muscles.iter().enumerate() {
        if muscle.is_bone {
            continue;
        }
        let Some((entity, ref mat_handle)) = cache.muscle_entries[i] else {
            continue;
        };

        let pa = creature.particles[muscle.a].pos;
        let pb = creature.particles[muscle.b].pos;
        let mid = (pa + pb) * 0.5;
        let diff = pb - pa;
        let len = diff.length().max(1e-4);
        let angle = diff.y.atan2(diff.x);

        let t = if muscle.amplitude > 1e-6 {
            (muscle.current_activation / muscle.amplitude).clamp(0.0, 1.0)
        } else {
            0.0
        };
        // 1 px rest → 4 px full activation, converted to world units (scale 1/80).
        let width = (1.0 + t * 3.0) / 80.0;

        if let Ok(mut tr) = q_transforms.get_mut(entity) {
            tr.translation = Vec3::new(mid.x, mid.y, 0.0);
            tr.rotation = Quat::from_rotation_z(angle);
            tr.scale = Vec3::new(width, len, 1.0);
        }

        // t=0 → overbright blue  (0.1, 0.3, 1.5)
        // t=1 → overbright orange (2.0, 0.4, 0.05)
        if let Some(mat) = materials.get_mut(mat_handle.id()) {
            mat.color = Color::linear_rgb(0.1 + t * 1.9, 0.3 + t * 0.1, 1.5 - t * 1.45);
        }
    }

    // Particles: move circle meshes.
    for (i, particle) in creature.particles.iter().enumerate() {
        let entity = cache.particle_entities[i];
        if let Ok(mut tr) = q_transforms.get_mut(entity) {
            tr.translation = Vec3::new(particle.pos.x, particle.pos.y, 1.0);
        }
    }

    // Debug labels: update positions and toggle visibility.
    for (i, particle) in creature.particles.iter().enumerate() {
        let Some(&label_entity) = cache.debug_label_entities.get(i) else {
            continue;
        };
        if let Ok(mut tr) = q_transforms.get_mut(label_entity) {
            tr.translation = Vec3::new(particle.pos.x + 0.1, particle.pos.y + 0.15, 5.0);
        }
        if let Ok(mut vis) = q_vis.get_mut(label_entity) {
            *vis = if debug { Visibility::Visible } else { Visibility::Hidden };
        }
    }

    // Debug overlays: velocity arrows + muscle activation markers.
    if debug {
        for particle in &creature.particles {
            let vel = (particle.pos - particle.prev_pos) * 60.0;
            if vel.length_squared() > 1e-6 {
                gizmos.line_2d(
                    particle.pos,
                    particle.pos + vel * 0.05,
                    Color::srgb(1.0, 1.0, 0.0),
                );
            }
        }

        for muscle in &creature.muscles {
            if !muscle.is_bone {
                let pa = creature.particles[muscle.a].pos;
                let pb = creature.particles[muscle.b].pos;
                let mid = (pa + pb) * 0.5;
                let t = if muscle.amplitude > 1e-6 {
                    (muscle.current_activation / muscle.amplitude).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                gizmos.circle_2d(mid, 0.04, Color::srgba(1.0, t, 0.0, 0.7));
            }
        }
    }
}
