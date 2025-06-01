use bevy::prelude::*;

use crate::GameState;
use crate::pause_menu::{PauseMenuState, PauseState};

// Constants for UI and gameplay
const TOP_UI_HEIGHT: f32 = 12.0; // Viewport height percentage
const BOTTOM_UI_HEIGHT: f32 = 8.0; // Viewport height percentage
const UI_PADDING: f32 = 2.0; // Viewport width percentage
const GRASS_BACKGROUND_COLOR: Color = Color::srgb(0.2, 0.6, 0.2);
const UI_BACKGROUND_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.8);
const COMBO_TIMER_WIDTH: f32 = 80.0;
const COMBO_TIMER_HEIGHT: f32 = 6.0;

/// Plugin for handling the main gameplay
pub struct PlayingPlugin;

impl Plugin for PlayingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), (setup_game_resources, setup_game_camera, setup_game_ui))
            .add_systems(
                Update,
                (handle_game_input, update_ui, update_combo_timer)
                    .run_if(in_state(PauseState::Playing))
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnEnter(GameState::Playing), play_level1_music)
            .add_systems(OnExit(GameState::Playing), cleanup_game)
            .add_systems(OnEnter(PauseState::Paused), toggle_level1_music)
            .add_systems(OnExit(PauseState::Paused), toggle_level1_music);
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
        self.combo = self.combo.saturating_add(1);
        self.score = self.score.saturating_add(Self::DANDELION_POINTS.saturating_mul(self.combo));

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

/// Initialize game resources
fn setup_game_resources(mut commands: Commands) {
    commands.insert_resource(GameData::new());
    info!("Game started!");
}

/// Setup the game camera and background
fn setup_game_camera(mut commands: Commands) {
    commands.spawn((Camera2d, GameEntity));

    commands.spawn((
        Sprite {
            color: GRASS_BACKGROUND_COLOR,
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, -1.0)).with_scale(Vec3::new(2000.0, 2000.0, 1.0)),
        GameEntity,
    ));
}

/// Setup the game UI layout
fn setup_game_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
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
            // Top UI panel with score, combo, and curb appeal
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Vh(TOP_UI_HEIGHT),
                        padding: UiRect::all(Val::Vw(UI_PADDING)),
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(UI_BACKGROUND_COLOR),
                ))
                .with_children(|parent| {
                    // Score display
                    parent.spawn((
                        Text::new("Score: 0"),
                        TextFont { font_size: 24.0, ..default() },
                        TextColor(Color::WHITE),
                        ScoreText,
                    ));

                    // Combo display with timer bar
                    parent
                        .spawn((Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            ..default()
                        },))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("Combo: 0x"),
                                TextFont { font_size: 20.0, ..default() },
                                TextColor(Color::srgb(1.0, 0.8, 0.2)),
                                ComboText,
                            ));

                            // Combo timer bar
                            parent
                                .spawn((
                                    Node {
                                        width: Val::Px(COMBO_TIMER_WIDTH),
                                        height: Val::Px(COMBO_TIMER_HEIGHT),
                                        border: UiRect::all(Val::Px(1.0)),
                                        ..default()
                                    },
                                    BorderColor(Color::srgb(0.5, 0.5, 0.5)),
                                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
                                ))
                                .with_children(|parent| {
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

            // Middle game area where gameplay happens
            parent.spawn((Node {
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                ..default()
            },));

            // Bottom UI panel with instructions
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Vh(BOTTOM_UI_HEIGHT),
                        padding: UiRect::all(Val::Vw(UI_PADDING)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(UI_BACKGROUND_COLOR),
                ))
                .with_children(|parent| {
                    parent
                        .spawn((Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(10.0),
                            ..default()
                        },))
                        .with_children(|parent| {
                            parent.spawn((
                                ImageNode::new(asset_server.load("dandelion_tiny.png")),
                                Node {
                                    width: Val::Px(16.0),
                                    height: Val::Px(16.0),
                                    ..default()
                                },
                            ));
                            parent.spawn((
                                Text::new("ESC: Return to Menu  |  Click dandelions to kill them!"),
                                TextFont { font_size: 16.0, ..default() },
                                TextColor(Color::srgb(0.8, 0.8, 0.8)),
                            ));
                        });
                });
        });
}

/// Handle input during gameplay
fn handle_game_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    pause_state: Res<State<PauseState>>,
    mut next_pause_state: ResMut<NextState<PauseState>>,
    mut next_pause_menu_state: ResMut<NextState<PauseMenuState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        match pause_state.get() {
            PauseState::Playing => {
                next_pause_state.set(PauseState::Paused);
                next_pause_menu_state.set(PauseMenuState::PauseMenu);
            }
            PauseState::Paused => {
                next_pause_state.set(PauseState::Playing);
            }
        }
    }
}

/// Calculate curb appeal based on dandelion count and types
fn calculate_curb_appeal(dandelion_query: &Query<&crate::enemies::Dandelion>) -> i32 {
    let mut total_impact = 0.0;

    // Count dandelions by size and calculate their curb appeal impact
    for dandelion in dandelion_query.iter() {
        let impact = match dandelion.size {
            crate::enemies::DandelionSize::Tiny => 1.0,
            crate::enemies::DandelionSize::Small => 1.5,
            crate::enemies::DandelionSize::Medium => 2.5,
            crate::enemies::DandelionSize::Large => 4.0,
            crate::enemies::DandelionSize::Huge => 6.0,
        };
        total_impact += impact;
    }

    // More forgiving formula that keeps appeal higher for longer
    // Uses square root to slow down the decline, especially at low counts
    let base_reduction = (total_impact * 2.0f32).sqrt() * 8.0f32;
    let curb_appeal = 100.0f32 - base_reduction;

    (curb_appeal.round() as i32).clamp(0, 100)
}

/// Update score display
fn update_score_display(game_data: &GameData, mut score_query: Query<&mut Text, (With<ScoreText>, Without<ComboText>, Without<CurbAppealText>)>) {
    if let Ok(mut text) = score_query.single_mut() {
        **text = format!("Score: {}", game_data.score);
    }
}

/// Update combo display
fn update_combo_display(game_data: &GameData, mut combo_query: Query<&mut Text, (With<ComboText>, Without<ScoreText>, Without<CurbAppealText>)>) {
    if let Ok(mut text) = combo_query.single_mut() {
        **text = format!("Combo: {}x", game_data.combo);
    }
}

/// Update combo timer bar
fn update_combo_timer_display(game_data: &GameData, mut combo_timer_bar_query: Query<&mut Node, With<ComboTimerBar>>) {
    if let Ok(mut node) = combo_timer_bar_query.single_mut() {
        if game_data.combo > 0 {
            let progress = game_data.combo_timer.remaining_secs() / game_data.combo_timer.duration().as_secs_f32();
            node.width = Val::Percent(progress * 100.0);
        } else {
            node.width = Val::Percent(0.0);
        }
    }
}

/// Update curb appeal display
fn update_curb_appeal_display(
    dandelion_query: Query<&crate::enemies::Dandelion>,
    mut curb_appeal_query: Query<&mut Text, (With<CurbAppealText>, Without<ScoreText>, Without<ComboText>)>,
) {
    if let Ok(mut text) = curb_appeal_query.single_mut() {
        let curb_appeal = calculate_curb_appeal(&dandelion_query);
        **text = format!("Curb Appeal: {}%", curb_appeal);
    }
}

/// Update game UI elements
fn update_ui(
    game_data: Res<GameData>,
    score_query: Query<&mut Text, (With<ScoreText>, Without<ComboText>, Without<CurbAppealText>)>,
    combo_query: Query<&mut Text, (With<ComboText>, Without<ScoreText>, Without<CurbAppealText>)>,
    combo_timer_bar_query: Query<&mut Node, With<ComboTimerBar>>,
    curb_appeal_query: Query<&mut Text, (With<CurbAppealText>, Without<ScoreText>, Without<ComboText>)>,
    dandelion_query: Query<&crate::enemies::Dandelion>,
) {
    update_score_display(&game_data, score_query);
    update_combo_display(&game_data, combo_query);
    update_combo_timer_display(&game_data, combo_timer_bar_query);
    update_curb_appeal_display(dandelion_query, curb_appeal_query);
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

/// Cleanup game entities when exiting playing state
fn cleanup_game(
    mut commands: Commands,
    game_entities: Query<Entity, With<GameEntity>>,
    mut next_pause_state: ResMut<NextState<PauseState>>,
    music: Query<Entity, With<Level1Music>>,
) {
    // Reset pause state
    next_pause_state.set(PauseState::Playing);

    // Remove game data resource
    commands.remove_resource::<GameData>();

    // Cleanup all game entities
    for entity in game_entities.iter() {
        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.despawn();
        }
    }

    for entity in &music {
        commands.entity(entity).despawn();
    }

    info!("Game ended, returning to menu");
}

#[derive(Component)]
struct Level1Music;

fn play_level1_music(asset_server: Res<AssetServer>, mut commands: Commands) {
    let music: Handle<AudioSource> = asset_server.load("audio/level1.wav");
    commands.spawn((
        AudioPlayer(music),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Loop,
            ..default()
        },
        Level1Music,
    ));
}

fn toggle_level1_music(query: Query<&AudioSink, With<Level1Music>>) {
    if let Ok(sink) = query.single() {
        sink.toggle_playback();
    }
}
