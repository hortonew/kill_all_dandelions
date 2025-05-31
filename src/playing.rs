use bevy::prelude::*;

use crate::GameState;

/// Plugin for handling the main gameplay
pub struct PlayingPlugin;

impl Plugin for PlayingPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<PauseState>()
            .add_systems(OnEnter(GameState::Playing), setup_game)
            .add_systems(
                Update,
                (
                    handle_game_input,
                    update_ui.run_if(in_state(PauseState::Playing)),
                    handle_pause_input.run_if(in_state(PauseState::Paused)),
                    pause_menu_interactions.run_if(in_state(PauseState::Paused)),
                )
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnEnter(PauseState::Paused), setup_pause_menu)
            .add_systems(OnExit(PauseState::Paused), cleanup_pause_menu)
            .add_systems(OnExit(GameState::Playing), cleanup_game);
    }
}

/// Local pause state within the playing state
#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
enum PauseState {
    Playing,
    Paused,
}

impl Default for PauseState {
    fn default() -> Self {
        Self::Playing
    }
}

/// Marker component for game entities
#[derive(Component)]
struct GameEntity;

/// Marker component for pause menu entities
#[derive(Component)]
struct PauseMenuEntity;

/// Pause menu button types
#[derive(Component)]
enum PauseMenuButton {
    Resume,
    Restart,
}

/// Game state resource
#[derive(Resource, Default)]
struct GameData {
    score: u32,
    combo: u32,
    dandelion_count: u32,
}

/// UI components
#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct ComboText;

#[derive(Component)]
struct CurbAppealText;

/// Setup the game scene and UI
fn setup_game(mut commands: Commands) {
    // Initialize game data
    commands.insert_resource(GameData::default());

    // Spawn main game camera
    commands.spawn((Camera2d, GameEntity));

    // Game UI container with flex layout
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            GameEntity,
        ))
        .with_children(|parent| {
            // Top UI panel
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Vh(12.0),
                        padding: UiRect::all(Val::Vw(2.0)),
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
                ))
                .with_children(|parent| {
                    // Score display
                    parent.spawn((
                        Text::new("Score: 0"),
                        TextFont { font_size: 24.0, ..default() },
                        TextColor(Color::WHITE),
                        ScoreText,
                    ));

                    // Combo display
                    parent.spawn((
                        Text::new("Combo: 0x"),
                        TextFont { font_size: 20.0, ..default() },
                        TextColor(Color::srgb(1.0, 0.8, 0.2)),
                        ComboText,
                    ));

                    // Curb appeal display
                    parent.spawn((
                        Text::new("Curb Appeal: 100%"),
                        TextFont { font_size: 20.0, ..default() },
                        TextColor(Color::srgb(0.3, 0.9, 0.3)),
                        CurbAppealText,
                    ));
                });

            // Middle game area (lawn) - takes remaining space
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    flex_grow: 1.0,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.2, 0.6, 0.2)),
            ));

            // Bottom UI panel
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Vh(8.0),
                        padding: UiRect::all(Val::Vw(2.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("ESC: Return to Menu  •  Click dandelions to kill them!"),
                        TextFont { font_size: 16.0, ..default() },
                        TextColor(Color::srgb(0.8, 0.8, 0.8)),
                    ));
                });
        });

    info!("Game started!");
}

/// Handle input during gameplay
fn handle_game_input(keyboard_input: Res<ButtonInput<KeyCode>>, pause_state: Res<State<PauseState>>, mut next_pause_state: ResMut<NextState<PauseState>>) {
    // Toggle pause with Escape
    if keyboard_input.just_pressed(KeyCode::Escape) {
        match pause_state.get() {
            PauseState::Playing => next_pause_state.set(PauseState::Paused),
            PauseState::Paused => next_pause_state.set(PauseState::Playing),
        }
    }
}

/// Handle input while paused
fn handle_pause_input(keyboard_input: Res<ButtonInput<KeyCode>>, mut next_pause_state: ResMut<NextState<PauseState>>) {
    // Resume with Escape
    if keyboard_input.just_pressed(KeyCode::Escape) {
        next_pause_state.set(PauseState::Playing);
    }
}

/// Setup pause menu UI
fn setup_pause_menu(mut commands: Commands) {
    // Semi-transparent overlay
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
            // Pause menu container
            parent
                .spawn((
                    Node {
                        width: Val::Px(300.0),
                        height: Val::Px(400.0),
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
                    // Pause title
                    parent.spawn((Text::new("Game Paused"), TextFont { font_size: 32.0, ..default() }, TextColor(Color::WHITE)));

                    // Resume button
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
                            parent.spawn((Text::new("Resume Game"), TextFont { font_size: 20.0, ..default() }, TextColor(Color::WHITE)));
                        });

                    // Restart button
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
                            parent.spawn((Text::new("Restart Game"), TextFont { font_size: 20.0, ..default() }, TextColor(Color::WHITE)));
                        });
                });
        });
}

/// Handle pause menu button interactions
fn pause_menu_interactions(
    mut interaction_query: Query<(&Interaction, &mut BackgroundColor, &PauseMenuButton), (Changed<Interaction>, With<Button>)>,
    mut next_pause_state: ResMut<NextState<PauseState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    for (interaction, mut color, button_type) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                match button_type {
                    PauseMenuButton::Resume => {
                        next_pause_state.set(PauseState::Playing);
                    }
                    PauseMenuButton::Restart => {
                        // Reset pause state first, then go to menu
                        next_pause_state.set(PauseState::Playing);
                        next_game_state.set(GameState::Menu);
                    }
                }
            }
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
fn cleanup_pause_menu(mut commands: Commands, pause_entities: Query<Entity, With<PauseMenuEntity>>) {
    for entity in &pause_entities {
        commands.entity(entity).despawn();
    }
}

/// Update game UI elements
fn update_ui(
    game_data: Res<GameData>,
    mut score_query: Query<&mut Text, (With<ScoreText>, Without<ComboText>, Without<CurbAppealText>)>,
    mut combo_query: Query<&mut Text, (With<ComboText>, Without<ScoreText>, Without<CurbAppealText>)>,
    mut curb_appeal_query: Query<&mut Text, (With<CurbAppealText>, Without<ScoreText>, Without<ComboText>)>,
) {
    // Update score
    if let Ok(mut text) = score_query.single_mut() {
        **text = format!("Score: {}", game_data.score);
    }

    // Update combo
    if let Ok(mut text) = combo_query.single_mut() {
        **text = format!("Combo: {}x", game_data.combo);
    }

    // Update curb appeal (simplified calculation)
    if let Ok(mut text) = curb_appeal_query.single_mut() {
        let curb_appeal = (100_i32 - (game_data.dandelion_count as i32 * 5)).max(0);
        **text = format!("Curb Appeal: {}%", curb_appeal);
    }
}

/// Cleanup game entities when exiting playing state
fn cleanup_game(mut commands: Commands, game_entities: Query<Entity, With<GameEntity>>, mut next_pause_state: ResMut<NextState<PauseState>>) {
    // Reset pause state
    next_pause_state.set(PauseState::Playing);

    // Remove game data resource
    commands.remove_resource::<GameData>();

    // Cleanup all game entities
    for entity in &game_entities {
        commands.entity(entity).despawn();
    }

    info!("Game ended, returning to menu");
}
