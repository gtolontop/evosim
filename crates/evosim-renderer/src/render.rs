use bevy::prelude::*;

use crate::simulation::SimulationState;

// ── Humanoid colour palette (anatomy-style) ─────────────────────────────────

const MUSCLE_COLOR: Color = Color::srgba(0.90, 0.45, 0.42, 0.65); // salmon pink, semi-transparent
const BONE_COLOR: Color = Color::srgb(0.95, 0.92, 0.85);          // cream white
const SPINE_COLOR: Color = Color::srgb(0.90, 0.88, 0.82);         // slightly darker cream
const JOINT_COLOR: Color = Color::srgb(0.95, 0.90, 0.82);         // warm cream
const ENDPOINT_COLOR: Color = Color::srgb(0.98, 0.95, 0.88);      // light cream
const ENDPOINT_GRIP_COLOR: Color = Color::srgb(1.0, 0.60, 0.40);  // orange when gripping

// ── Marker components ─────────────────────────────────────────────────────────

#[derive(Component)]
pub(crate) struct MuscleMarker;

#[derive(Component)]
pub(crate) struct ParticleMarker;

#[derive(Component)]
pub(crate) struct DebugLabelMarker;

// ── Cache resource ────────────────────────────────────────────────────────────

#[derive(Resource, Default)]
pub struct MuscleRenderCache {
    pub muscle_entries: Vec<Option<(Entity, Handle<ColorMaterial>)>>,
    pub particle_entities: Vec<Entity>,
    pub particle_materials: Vec<Handle<ColorMaterial>>,
    pub debug_label_entities: Vec<Entity>,
}

/// Base widths per muscle group index (0–7).
/// Deltoid, GrandDorsal, Bicep, Tricep, HipFlexor, Glute, Quad, Hamstring
/// Thin flat bands — matching the reference images (not puffy ellipses).
const MUSCLE_BASE_WIDTHS: [f32; 8] = [0.03, 0.025, 0.02, 0.02, 0.025, 0.035, 0.03, 0.025];

// ── Setup (Startup) ───────────────────────────────────────────────────────────

pub fn setup_render_cache(
    mut commands: Commands,
    state: Res<SimulationState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut cache: ResMut<MuscleRenderCache>,
) {
    let creature = &state.creature;

    // Single quad mesh for everything — flat band look matching reference images.
    let quad_mesh = meshes.add(Rectangle::new(1.0, 1.0));

    for (i, muscle) in creature.muscles.iter().enumerate() {
        // Cross-stability diagonals (17, 18): skip — invisible, physics only.
        if i == 17 || i == 18 {
            cache.muscle_entries.push(None);
            continue;
        }

        let color = if i <= 1 {
            SPINE_COLOR
        } else if muscle.is_bone {
            BONE_COLOR
        } else {
            MUSCLE_COLOR
        };

        let mat = materials.add(ColorMaterial {
            color,
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

    // Particle meshes — cream/white joints
    for (i, particle) in creature.particles.iter().enumerate() {
        let (radius, color) = match i {
            0 | 1 | 2 => (0.025, JOINT_COLOR),         // spine
            3 | 4 => (0.025, JOINT_COLOR),              // shoulders
            9 | 10 => (0.025, JOINT_COLOR),             // hips
            5 | 6 | 11 | 12 => (0.02, JOINT_COLOR),    // elbows/knees
            7 | 8 | 13 | 14 => (0.025, ENDPOINT_COLOR), // hands/feet
            15 => (0.015, BONE_COLOR),                   // tail tip
            _ => (0.02, JOINT_COLOR),
        };

        let circle = meshes.add(Circle::new(radius));
        let mat = materials.add(ColorMaterial {
            color,
            ..default()
        });

        let entity = commands
            .spawn((
                Mesh2d(circle),
                MeshMaterial2d(mat.clone()),
                Transform::from_translation(Vec3::new(particle.pos.x, particle.pos.y, 1.0)),
                ParticleMarker,
            ))
            .id();
        cache.particle_entities.push(entity);
        cache.particle_materials.push(mat);

        // Debug index label
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

    // ── Muscles ──────────────────────────────────────────────────────────
    for (i, muscle) in creature.muscles.iter().enumerate() {
        let Some((entity, ref mat_handle)) = cache.muscle_entries[i] else {
            continue;
        };

        let pa = creature.particles[muscle.a].pos;
        let pb = creature.particles[muscle.b].pos;
        let mid = (pa + pb) * 0.5;
        let diff = pb - pa;
        let len = diff.length().max(1e-4);
        let angle = diff.y.atan2(diff.x);

        // Width — thin bands matching reference anatomy images
        let width = if i == 16 {
            0.01                                         // tail — very thin bone
        } else if i <= 1 {
            0.025                                        // spine bones
        } else if i == 4 || i == 11 {
            0.015                                        // shoulder/hip bars
        } else if muscle.is_bone {
            0.015                                        // clavicles, pelvis, limb bones
        } else {
            // Active muscle — thin band, width scales with max_force
            let group = muscle.group as usize;
            let base = if group < 8 { MUSCLE_BASE_WIDTHS[group] } else { 0.03 };
            base * (0.5 + muscle.max_force * 0.5)
        };

        // Z-order
        let z = if i <= 1 {
            -0.3  // spine
        } else if i == 16 {
            -0.2  // tail
        } else if muscle.is_bone {
            0.1   // bones
        } else {
            0.2   // active muscles — front
        };

        if let Ok(mut tr) = q_transforms.get_mut(entity) {
            tr.translation = Vec3::new(mid.x, mid.y, z);
            tr.rotation = Quat::from_rotation_z(angle);
            tr.scale = Vec3::new(len, width, 1.0);
        }

        // Activation colour modulation for active muscles
        if !muscle.is_bone {
            let t = muscle.current_contraction.clamp(0.0, 1.0);
            if let Some(mat) = materials.get_mut(mat_handle.id()) {
                let boost = t * 0.15;
                mat.color = Color::srgba(
                    0.85 + boost,
                    0.40 + boost * 0.5,
                    0.38 + boost * 0.3,
                    0.65,
                );
            }
        }
    }

    // ── Particles: move circle meshes ────────────────────────────────────
    for (i, particle) in creature.particles.iter().enumerate() {
        let z = match i {
            0 | 1 | 2 => 0.05,          // spine — behind
            3 | 4 | 9 | 10 => 0.15,     // shoulders/hips
            5 | 6 | 11 | 12 => 0.3,     // elbows/knees
            7 | 8 | 13 | 14 => 0.35,    // hands/feet — front
            15 => 0.0,                    // tail — behind everything
            _ => 0.3,
        };
        let entity = cache.particle_entities[i];
        if let Ok(mut tr) = q_transforms.get_mut(entity) {
            tr.translation = Vec3::new(particle.pos.x, particle.pos.y, z);
        }
    }

    // ── Endpoint grip colour (cream free / orange gripping) ──────────────
    for &(p_idx, _) in &creature.grip_phases {
        if let Some(mat_handle) = cache.particle_materials.get(p_idx) {
            if let Some(mat) = materials.get_mut(mat_handle.id()) {
                mat.color = if creature.particles[p_idx].pinned {
                    ENDPOINT_GRIP_COLOR
                } else {
                    ENDPOINT_COLOR
                };
            }
        }
    }

    // ── Debug labels ─────────────────────────────────────────────────────
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

    // ── Debug overlays ───────────────────────────────────────────────────
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
                let t = muscle.current_contraction.clamp(0.0, 1.0);
                gizmos.circle_2d(mid, 0.04, Color::srgba(1.0, t, 0.0, 0.7));
            }
        }
    }
}
