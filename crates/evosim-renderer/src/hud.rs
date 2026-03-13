use bevy::prelude::*;

use crate::simulation::SimulationState;

/// Marker component for the HUD text entity.
#[derive(Component)]
pub struct HudText;

/// Spawns the top-left HUD panel with an empty text placeholder.
pub fn setup_hud(mut commands: Commands) {
    // Outer panel: dark semi-transparent background
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.65)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                HudText,
            ));
        });
}

/// Rewrites the HUD text every frame with current simulation stats.
pub fn update_hud_system(
    state: Res<SimulationState>,
    mut query: Query<&mut Text, With<HudText>>,
) {
    let paused_tag = if state.paused { " [PAUSED]" } else { "" };
    let content = format!(
        "evosim{}\n\
         ━━━━━━━━━━━━━━\n\
         Gen:     {:04}\n\
         Fitness: {:.3}\n\
         Step:    {:06}\n\
         Speed:   {:.1}x\n\
         ━━━━━━━━━━━━━━\n\
         [SPACE] pause\n\
         [↑↓]   speed\n\
         [R]    reset",
        paused_tag,
        state.generation,
        state.fitness,
        state.step_count,
        state.speed_multiplier,
    );

    for mut text in &mut query {
        *text = Text::new(content.clone());
    }
}
