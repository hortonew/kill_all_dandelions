use bevy::prelude::*;

use crate::GameState;

/// Plugin for handling the pause menu
pub struct PauseMenuPlugin;

impl Plugin for PauseMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<PauseState>()
            .add_systems(OnEnter(PauseState::Paused), setup_pause_menu)
            .add_systems(OnExit(PauseState::Paused), cleanup_pause_menu)
            .add_systems(
                Update,
                (handle_pause_input, pause_menu_interactions).run_if(in_state(PauseState::Paused)),
            );
    }
}

/// Local pause state within the playing state
#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PauseState {
    Playing,
    Paused,
}

impl Default for PauseState {
    fn default() -> Self {
        Self::Playing
    }
}

/// Marker component for pause menu entities
#[derive(Component)]
struct PauseMenuEntity;

/// Pause menu button types
#[derive(Component)]
enum PauseMenuButton {
    Resume,
    Restart,
}

/// Handle input while paused
fn handle_pause_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_pause_state: ResMut<NextState<PauseState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        next_pause_state.set(PauseState::Playing);
    }
}

/// Setup pause menu UI
fn setup_pause_menu(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            PauseMenuEntity,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(300.0),
                        height: Val::Px(300.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        padding: UiRect::all(Val::Px(20.0)),
                        row_gap: Val::Px(20.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                    BorderRadius::all(Val::Px(10.0)),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Game Paused"),
                        TextFont {
                            font_size: 32.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(200.0),
                                height: Val::Px(50.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                            BorderRadius::all(Val::Px(5.0)),
                            PauseMenuButton::Resume,
                            PauseMenuEntity,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("Resume Game"),
                                TextFont {
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });

                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(200.0),
                                height: Val::Px(50.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.6, 0.3, 0.3)),
                            BorderRadius::all(Val::Px(5.0)),
                            PauseMenuButton::Restart,
                            PauseMenuEntity,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("Restart Game"),
                                TextFont {
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });
                });
        });
}

/// Handle pause menu button interactions
fn pause_menu_interactions(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &PauseMenuButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_pause_state: ResMut<NextState<PauseState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    for (interaction, mut color, button_type) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => match button_type {
                PauseMenuButton::Resume => {
                    next_pause_state.set(PauseState::Playing);
                }
                PauseMenuButton::Restart => {
                    next_pause_state.set(PauseState::Playing);
                    next_game_state.set(GameState::Menu);
                }
            },
            Interaction::Hovered => match button_type {
                PauseMenuButton::Resume => *color = BackgroundColor(Color::srgb(0.5, 0.5, 0.5)),
                PauseMenuButton::Restart => *color = BackgroundColor(Color::srgb(0.8, 0.4, 0.4)),
            },
            Interaction::None => match button_type {
                PauseMenuButton::Resume => *color = BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                PauseMenuButton::Restart => *color = BackgroundColor(Color::srgb(0.6, 0.3, 0.3)),
            },
        }
    }
}

/// Cleanup pause menu entities
fn cleanup_pause_menu(
    mut commands: Commands,
    pause_entities: Query<Entity, With<PauseMenuEntity>>,
) {
    for entity in &pause_entities {
        commands.entity(entity).despawn();
    }
}
