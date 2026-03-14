use bevy::prelude::*;

use crate::simulation::SimulationState;

// ── Humanoid colour palette ─────────────────────────────────────────────────

const BODY_COLOR: Color = Color::srgb(0.30, 0.65, 0.30);
const SPINE_COLOR: Color = Color::srgba(0.45, 0.70, 0.35, 0.75);
const BONE_COLOR: Color = Color::srgb(0.85, 0.85, 0.75);
const LIMB_COLOR: Color = Color::srgb(0.85, 0.40, 0.40);
const JOINT_COLOR: Color = Color::srgb(0.40, 0.75, 0.35);
const SHOULDER_HIP_COLOR: Color = Color::srgb(0.35, 0.65, 0.30);
const ENDPOINT_COLOR: Color = Color::srgb(1.0, 0.85, 0.3);
const ENDPOINT_GRIP_COLOR: Color = Color::srgb(1.0, 0.4, 0.2);
const TORSO_FILL_COLOR: Color = Color::srgba(0.35, 0.60, 0.30, 0.70);

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
#[derive(Component)]
pub(crate) struct TorsoMarker;

#[derive(Resource, Default)]
pub struct MuscleRenderCache {
    /// One entry per `creature.muscles`.
    pub muscle_entries: Vec<Option<(Entity, Handle<ColorMaterial>)>>,
    /// One entity per `creature.particles`.
    pub particle_entities: Vec<Entity>,
    /// Material handle per particle — needed to dynamically update endpoint grip colours.
    pub particle_materials: Vec<Handle<ColorMaterial>>,
    /// One `Text2d` entity per particle (shown only in debug mode).
    pub debug_label_entities: Vec<Entity>,
    /// Dedicated torso body quad (shoulders → hips).
    pub torso_entity: Option<Entity>,
}

// ── Setup (Startup) ───────────────────────────────────────────────────────────

/// Spawns mesh entities for muscles and particles with role-based colours.
/// Called once at Startup after `SimulationState` is available.
pub fn setup_render_cache(
    mut commands: Commands,
    state: Res<SimulationState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut cache: ResMut<MuscleRenderCache>,
) {
    let creature = &state.creature;

    // Unit quad — scaled per-muscle to (length, width) each frame.
    let quad_mesh = meshes.add(Rectangle::new(1.0, 1.0));


    for (i, muscle) in creature.muscles.iter().enumerate() {
        // Role-based colour.
        let color = if i == 16 || i == 17 {
            TORSO_FILL_COLOR // cross-stability → semi-transparent body fill
        } else if i <= 1 {
            SPINE_COLOR      // spine segments
        } else if muscle.is_bone {
            BONE_COLOR       // structural + limb bones
        } else {
            LIMB_COLOR       // active muscles
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

    // Role-based particle meshes (15 particles).
    //  0,1,2    = spine          → large, BODY_COLOR
    //  3,4      = shoulders      → medium, SHOULDER_HIP_COLOR
    //  9,10     = hips           → medium, SHOULDER_HIP_COLOR
    //  5,7,11,13 = joints        → small, JOINT_COLOR
    //  6,8,12,14 = endpoints     → medium, ENDPOINT_COLOR
    for (i, particle) in creature.particles.iter().enumerate() {
        let (radius, color) = match i {
            0 | 1 | 2 => (0.09, BODY_COLOR),       // spine
            3 | 4 => (0.07, SHOULDER_HIP_COLOR),    // shoulders
            9 | 10 => (0.07, SHOULDER_HIP_COLOR),   // hips
            5 | 7 | 11 | 13 => (0.04, JOINT_COLOR), // elbows/knees
            6 | 8 | 12 | 14 => (0.05, ENDPOINT_COLOR), // hands/feet
            _ => (0.04, Color::WHITE),
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

    // Torso body quad — a single rectangle positioned between shoulders and hips.
    let torso_mesh = meshes.add(Rectangle::new(1.0, 1.0));
    let torso_mat = materials.add(ColorMaterial {
        color: TORSO_FILL_COLOR,
        ..default()
    });
    let torso = commands
        .spawn((
            Mesh2d(torso_mesh),
            MeshMaterial2d(torso_mat),
            Transform::default(),
            TorsoMarker,
        ))
        .id();
    cache.torso_entity = Some(torso);
}

// ── Per-frame update (Update) ─────────────────────────────────────────────────

/// Updates mesh transforms and material colours each frame to match the
/// creature's current physics state.
pub fn render_creature_system(
    state: Res<SimulationState>,
    cache: Res<MuscleRenderCache>,
    mut q_transforms: Query<
        &mut Transform,
        Or<(With<MuscleMarker>, With<ParticleMarker>, With<DebugLabelMarker>, With<TorsoMarker>)>,
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

        let is_cross = i == 16 || i == 17;

        // Width based on segment role.
        let width = if is_cross {
            0.02  // cross-stability — hairline
        } else if i <= 1 {
            0.14  // spine
        } else if i <= 5 {
            0.08  // clavicles + pelvis
        } else if i <= 7 {
            0.05  // shoulder bar + hip bar
        } else if i <= 15 {
            0.04  // limb bones — thin rigid segments
        } else if !muscle.is_bone {
            0.05  // active cross-joint muscles
        } else {
            0.04  // fallback
        };

        // Z-order: torso behind everything, then bones, then muscles in front.
        let z = if is_cross {
            -0.4  // cross-stability — behind bones
        } else if i <= 1 {
            -0.3  // spine
        } else if muscle.is_bone {
            0.1   // structural + limb bones
        } else {
            0.2   // active muscles — front
        };

        if let Ok(mut tr) = q_transforms.get_mut(entity) {
            tr.translation = Vec3::new(mid.x, mid.y, z);
            tr.rotation = Quat::from_rotation_z(angle);
            tr.scale = Vec3::new(len, width, 1.0);
        }

        // Activation brightness modulation for active muscles (red tones).
        if !muscle.is_bone {
            let t = if muscle.amplitude > 1e-6 {
                (muscle.current_activation / muscle.amplitude).clamp(0.0, 1.0)
            } else {
                0.0
            };
            if let Some(mat) = materials.get_mut(mat_handle.id()) {
                let boost = t * 0.25;
                mat.color = Color::srgb(
                    0.70 + boost,
                    0.30 + boost * 0.3,
                    0.30 + boost * 0.2,
                );
            }
        }
    }

    // ── Torso body quad (shoulders 3,4 → hips 9,10) ─────────────────────
    if let Some(torso_entity) = cache.torso_entity {
        if creature.particles.len() >= 15 {
            let p3 = creature.particles[3].pos;  // shoulder A
            let p4 = creature.particles[4].pos;  // shoulder B
            let p9 = creature.particles[9].pos;  // hip A
            let p10 = creature.particles[10].pos; // hip B

            let shoulder_mid = (p3 + p4) * 0.5;
            let hip_mid = (p9 + p10) * 0.5;
            let center = (shoulder_mid + hip_mid) * 0.5;

            let spine_dir = shoulder_mid - hip_mid;
            let height = spine_dir.length().max(0.01);
            let angle = spine_dir.y.atan2(spine_dir.x);

            let shoulder_width = (p3 - p4).length();
            let hip_width = (p9 - p10).length();
            let width = shoulder_width.max(hip_width) + 0.05; // slight padding

            if let Ok(mut tr) = q_transforms.get_mut(torso_entity) {
                tr.translation = Vec3::new(center.x, center.y, -1.0);
                // Align X axis along spine direction; Y axis = body width
                tr.rotation = Quat::from_rotation_z(angle);
                tr.scale = Vec3::new(height, width, 1.0);
            }
        }
    }

    // ── Particles: move circle meshes ────────────────────────────────────
    for (i, particle) in creature.particles.iter().enumerate() {
        // Z per role: body behind limbs, endpoints frontmost.
        let z = match i {
            0 | 1 | 2 => 0.05,          // spine — just above spine muscles
            3 | 4 | 9 | 10 => 0.15,     // shoulders/hips — between bones and limbs
            5 | 7 | 11 | 13 => 0.3,     // elbows/knees — in front of limb muscles
            6 | 8 | 12 | 14 => 0.35,    // hands/feet — frontmost
            _ => 0.3,
        };
        let entity = cache.particle_entities[i];
        if let Ok(mut tr) = q_transforms.get_mut(entity) {
            tr.translation = Vec3::new(particle.pos.x, particle.pos.y, z);
        }
    }

    // ── Endpoint grip colour (yellow free / orange gripping) ─────────────
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

    // ── Debug overlays: velocity arrows + muscle activation ──────────────
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
