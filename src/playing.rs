use bevy::prelude::*;

use crate::GameState;
use crate::pause_menu::PauseState;

/// Plugin for handling the main gameplay
pub struct PlayingPlugin;

impl Plugin for PlayingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), (setup_game_resources, setup_game_camera, setup_game_ui))
            .add_systems(
                Update,
                (handle_game_input, update_ui.run_if(in_state(PauseState::Playing))).run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnExit(GameState::Playing), cleanup_game);
    }
}

/// Marker component for game entities
#[derive(Component)]
struct GameEntity;

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

/// Initialize game resources
fn setup_game_resources(mut commands: Commands) {
    commands.insert_resource(GameData::default());
    info!("Game started!");
}

/// Setup the game camera
fn setup_game_camera(mut commands: Commands) {
    commands.spawn((Camera2d, GameEntity));
}

/// Setup the game UI layout
fn setup_game_ui(mut commands: Commands) {
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
}

/// Handle input during gameplay
fn handle_game_input(keyboard_input: Res<ButtonInput<KeyCode>>, pause_state: Res<State<PauseState>>, mut next_pause_state: ResMut<NextState<PauseState>>) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        match pause_state.get() {
            PauseState::Playing => next_pause_state.set(PauseState::Paused),
            PauseState::Paused => next_pause_state.set(PauseState::Playing),
        }
    }
}

/// Update game UI elements
fn update_ui(
    game_data: Res<GameData>,
    mut score_query: Query<&mut Text, (With<ScoreText>, Without<ComboText>, Without<CurbAppealText>)>,
    mut combo_query: Query<&mut Text, (With<ComboText>, Without<ScoreText>, Without<CurbAppealText>)>,
    mut curb_appeal_query: Query<&mut Text, (With<CurbAppealText>, Without<ScoreText>, Without<ComboText>)>,
) {
    if let Ok(mut text) = score_query.single_mut() {
        **text = format!("Score: {}", game_data.score);
    }

    if let Ok(mut text) = combo_query.single_mut() {
        **text = format!("Combo: {}x", game_data.combo);
    }

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
