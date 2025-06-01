use bevy::prelude::*;

use crate::GameState;
use crate::pause_menu::PauseState;
use crate::powerups::SelectedPowerup;

/// Plugin for handling the main gameplay
pub struct PlayingPlugin;

impl Plugin for PlayingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), (setup_game_resources, setup_game_camera, setup_game_ui))
            .add_systems(
                Update,
                (handle_game_input, update_ui, update_combo_timer, update_cursor_feedback, update_powerup_cursor)
                    .run_if(in_state(PauseState::Playing))
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnExit(GameState::Playing), cleanup_game);
    }
}

/// Marker component for game entities
#[derive(Component)]
struct GameEntity;

/// Game state resource
#[derive(Resource, Default)]
pub struct GameData {
    pub score: u32,
    pub combo: u32,
    pub combo_timer: Timer,
    pub dandelion_count: u32,
}

impl GameData {
    const DANDELION_POINTS: u32 = 10;
    const INITIAL_COMBO_TIME: f32 = 3.0;
    const MAX_COMBO_TIME: f32 = 6.0;

    fn new() -> Self {
        Self {
            score: 0,
            combo: 0,
            combo_timer: Timer::from_seconds(Self::INITIAL_COMBO_TIME, TimerMode::Once),
            dandelion_count: 0,
        }
    }

    pub fn add_dandelion_kill(&mut self) {
        self.combo += 1;
        self.score += Self::DANDELION_POINTS * self.combo;

        // Calculate new timer duration based on combo level (logarithmic growth)
        let combo_factor = (self.combo as f32).ln() + 1.0;
        let new_duration = (Self::INITIAL_COMBO_TIME + combo_factor * 0.8).min(Self::MAX_COMBO_TIME);

        self.combo_timer.set_duration(std::time::Duration::from_secs_f32(new_duration));
        self.combo_timer.reset();
    }

    pub fn reset_combo(&mut self) {
        self.combo = 0;
        self.combo_timer.set_duration(std::time::Duration::from_secs_f32(Self::INITIAL_COMBO_TIME));
        self.combo_timer.reset();
    }
}

/// UI components
#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct ComboText;

#[derive(Component)]
struct ComboTimerBar;

#[derive(Component)]
struct CurbAppealText;

#[derive(Component)]
struct PowerupCursor;

/// Initialize game resources
fn setup_game_resources(mut commands: Commands) {
    commands.insert_resource(GameData::new());
    info!("Game started!");
}

/// Setup the game camera
fn setup_game_camera(mut commands: Commands) {
    commands.spawn((Camera2d, GameEntity));

    // Add grass background sprite to cover the play area
    commands.spawn((
        Sprite {
            color: Color::srgb(0.2, 0.6, 0.2), // Green grass color
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, -1.0)).with_scale(Vec3::new(2000.0, 2000.0, 1.0)), // Large background
        GameEntity,
    ));
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
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
                ))
                .with_children(|parent| {
                    // Score display
                    parent.spawn((
                        Text::new("Score: 0"),
                        TextFont { font_size: 24.0, ..default() },
                        TextColor(Color::WHITE),
                        ScoreText,
                    ));

                    // Combo display with timer
                    parent
                        .spawn((Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            ..default()
                        },))
                        .with_children(|parent| {
                            // Combo multiplier
                            parent.spawn((
                                Text::new("Combo: 0x"),
                                TextFont { font_size: 20.0, ..default() },
                                TextColor(Color::srgb(1.0, 0.8, 0.2)),
                                ComboText,
                            ));

                            // Combo timer bar container
                            parent
                                .spawn((
                                    Node {
                                        width: Val::Px(80.0),
                                        height: Val::Px(6.0),
                                        border: UiRect::all(Val::Px(1.0)),
                                        ..default()
                                    },
                                    BorderColor(Color::srgb(0.5, 0.5, 0.5)),
                                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
                                ))
                                .with_children(|parent| {
                                    // Timer bar fill
                                    parent.spawn((
                                        Node {
                                            width: Val::Percent(0.0),
                                            height: Val::Percent(100.0),
                                            ..default()
                                        },
                                        BackgroundColor(Color::srgb(1.0, 0.8, 0.2)),
                                        ComboTimerBar,
                                    ));
                                });
                        });

                    // Curb appeal display
                    parent.spawn((
                        Text::new("Curb Appeal: 100%"),
                        TextFont { font_size: 20.0, ..default() },
                        TextColor(Color::srgb(0.3, 0.9, 0.3)),
                        CurbAppealText,
                    ));
                });

            // Middle game area (lawn) - takes remaining space, no background color
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    flex_grow: 1.0,
                    ..default()
                },
                // Removed BackgroundColor to let world sprites show through
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
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("ESC: Return to Menu  •  Click dandelions to kill them!"),
                        TextFont { font_size: 16.0, ..default() },
                        TextColor(Color::srgb(0.8, 0.8, 0.8)),
                    ));
                });

            // Powerup cursor (initially hidden)
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Px(32.0),
                    height: Val::Px(32.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                PowerupCursor,
                Visibility::Hidden,
            ));
        });
}

/// Handle input during gameplay
fn handle_game_input(keyboard_input: Res<ButtonInput<KeyCode>>, pause_state: Res<State<PauseState>>, mut next_pause_state: ResMut<NextState<PauseState>>) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        match pause_state.get() {
            PauseState::Playing => next_pause_state.set(PauseState::Paused),
            PauseState::Paused => next_pause_state.set(PauseState::Playing),
            PauseState::PowerupHelp => next_pause_state.set(PauseState::Paused),
        }
    }
}

/// Update game UI elements
fn update_ui(
    game_data: Res<GameData>,
    mut score_query: Query<&mut Text, (With<ScoreText>, Without<ComboText>, Without<CurbAppealText>)>,
    mut combo_query: Query<&mut Text, (With<ComboText>, Without<ScoreText>, Without<CurbAppealText>)>,
    mut combo_timer_bar_query: Query<&mut Node, With<ComboTimerBar>>,
    mut curb_appeal_query: Query<&mut Text, (With<CurbAppealText>, Without<ScoreText>, Without<ComboText>)>,
    windows: Query<&Window>,
    area_tracker: Res<crate::enemies::DandelionAreaTracker>,
) {
    if let Ok(mut text) = score_query.single_mut() {
        **text = format!("Score: {}", game_data.score);
    }

    if let Ok(mut text) = combo_query.single_mut() {
        **text = format!("Combo: {}x", game_data.combo);
    }

    if let Ok(mut node) = combo_timer_bar_query.single_mut() {
        if game_data.combo > 0 {
            let progress = game_data.combo_timer.remaining_secs() / game_data.combo_timer.duration().as_secs_f32();
            node.width = Val::Percent(progress * 100.0);
        } else {
            node.width = Val::Percent(0.0);
        }
    }

    if let Ok(mut text) = curb_appeal_query.single_mut() {
        let curb_appeal = if let Ok(window) = windows.single() {
            // Calculate playable area (excluding UI panels)
            let margin = 30.0;
            let top_ui_height = window.height() * 0.12; // 12vh for top panel
            let bottom_ui_height = window.height() * 0.08; // 8vh for bottom panel

            let playable_width = window.width() - (margin * 2.0);
            let playable_height = window.height() - top_ui_height - bottom_ui_height - (margin * 2.0);
            let total_lawn_area = playable_width * playable_height;

            // Calculate total dandelion coverage area
            let total_dandelion_area = area_tracker.total_area;

            // Calculate coverage percentage and curb appeal
            let coverage_percentage = (total_dandelion_area / total_lawn_area) * 100.0;
            let curb_appeal = (100.0 - coverage_percentage.min(100.0)).max(0.0);
            curb_appeal as i32
        } else {
            100 // Default if no window found
        };
        **text = format!("Curb Appeal: {}%", curb_appeal);
    }
}

/// Update combo timer and reset combo when it expires
fn update_combo_timer(mut game_data: ResMut<GameData>, time: Res<Time>) {
    if game_data.combo > 0 {
        game_data.combo_timer.tick(time.delta());

        if game_data.combo_timer.finished() {
            game_data.reset_combo();
            info!("Combo expired! Reset to 0");
        }
    }
}

/// Update cursor feedback based on selected powerup
fn update_cursor_feedback(selected_powerup: Res<SelectedPowerup>, mut windows: Query<&mut Window>) {
    if let Ok(mut window) = windows.single_mut() {
        match selected_powerup.powerup_type {
            Some(_powerup_type) => {
                // Hide cursor when powerup is selected
                window.cursor_options.visible = false;
            }
            None => {
                // Show default cursor
                window.cursor_options.visible = true;
            }
        }
    }
}

/// Update powerup cursor position and visibility
fn update_powerup_cursor(
    selected_powerup: Res<SelectedPowerup>,
    windows: Query<&Window>,
    mut cursor_query: Query<(Entity, &mut Node, &mut Visibility), With<PowerupCursor>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    if let Ok(window) = windows.single() {
        if let Ok((entity, mut node, mut visibility)) = cursor_query.single_mut() {
            match selected_powerup.powerup_type {
                Some(powerup_type) => {
                    // Show powerup cursor
                    *visibility = Visibility::Visible;

                    // Update cursor position to follow mouse
                    if let Some(cursor_pos) = window.cursor_position() {
                        node.left = Val::Px(cursor_pos.x - 16.0); // Center the 32px cursor
                        node.top = Val::Px(cursor_pos.y - 16.0);
                    }

                    // Update cursor image if changed
                    // Note: This is a simplified approach - in a real game you'd cache the image
                    if selected_powerup.is_changed() {
                        commands.entity(entity).insert(ImageNode::new(asset_server.load(powerup_type.asset_path())));
                    }
                }
                None => {
                    // Hide powerup cursor
                    *visibility = Visibility::Hidden;
                }
            }
        }
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
