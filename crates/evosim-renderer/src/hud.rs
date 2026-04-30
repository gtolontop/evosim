use bevy::prelude::*;

use crate::simulation::SimulationState;

/// Marker component for the HUD text entity.
#[derive(Component)]
pub struct HudText;

/// Spawns the top-left HUD panel with an empty text placeholder.
pub fn setup_hud(mut commands: Commands) {
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
    let height = state.creature.max_height;
    let climbed = (height - 3.0_f32).max(0.0); // climbed above START_HEIGHT
    let com_y = state.creature.center_of_mass().y;
    let time_s = state.step_count as f32 * 0.016;
    let max_time = state.max_steps as f32 * 0.016;

    let status = if state.evaluation_done {
        "DONE".to_string()
    } else if state.paused {
        "▶ [SPACE]".to_string()
    } else {
        format!("{:.1}s / {:.1}s", time_s, max_time)
    };

    let content = format!(
        "Gen {:04}  fit {:.1} (champ {:.1})\nGrimpe: {:.1}m  (max {:.1}m  COM {:.1}m)\n{status}  [R] reset  [D] debug",
        state.generation,
        state.fitness,
        state.champion_fitness,
        climbed,
        height,
        com_y,
    );

    for mut text in &mut query {
        *text = Text::new(content.clone());
    }
}
