use bevy::prelude::*;

use crate::GameState;
use crate::levels::{LevelCompleteEvent, LevelData, LevelStartEvent};
use crate::pause_menu::{PauseMenuState, PauseState};

// Constants for UI and gameplay
const TOP_UI_HEIGHT: f32 = 12.0; // Viewport height percentage
const BOTTOM_UI_HEIGHT: f32 = 10.0; // Viewport height percentage - increased for mobile buttons
const UI_PADDING: f32 = 2.0; // Viewport width percentage
const GRASS_BACKGROUND_COLOR: Color = Color::srgb(0.2, 0.6, 0.2);
const UI_BACKGROUND_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.8);
const COMBO_TIMER_WIDTH: f32 = 80.0;
const COMBO_TIMER_HEIGHT: f32 = 6.0;

/// Plugin for handling the main gameplay
pub struct PlayingPlugin;

impl Plugin for PlayingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Playing),
            (setup_game_resources, setup_game_camera, setup_game_ui, setup_level_complete_overlay).chain(),
        )
        .add_systems(
            Update,
            (
                handle_game_input,
                handle_button_interactions,
                update_ui,
                update_button_text,
                update_combo_timer,
                update_slash_effects,
                update_delayed_slash_effects,
                handle_level_completion_events,
                handle_level_start_events,
                update_dynamic_font_sizes,
            )
                .run_if(in_state(PauseState::Playing))
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(Update, handle_level_completion_interactions.run_if(in_state(GameState::Playing)))
        .add_systems(OnEnter(GameState::Playing), play_level1_music.after(setup_game_resources))
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
    pub slash_mode: bool,
    pub slash_offset: f32,
    pub music_enabled: bool,
}

impl GameData {
    const DANDELION_POINTS: u32 = 10;
    const INITIAL_COMBO_TIME: f32 = 3.0;
    const MAX_COMBO_TIME: f32 = 6.0;
    const DEFAULT_SLASH_OFFSET: f32 = 30.0; // Distance from click point to slash endpoints (about 3 pointers)

    fn new() -> Self {
        Self {
            score: 0,
            combo: 0,
            combo_timer: Timer::from_seconds(Self::INITIAL_COMBO_TIME, TimerMode::Once),
            dandelion_count: 0,
            slash_mode: true,
            slash_offset: Self::DEFAULT_SLASH_OFFSET,
            music_enabled: true,
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

    pub fn toggle_slash_mode(&mut self) {
        self.slash_mode = !self.slash_mode;
    }

    pub fn toggle_music(&mut self) {
        self.music_enabled = !self.music_enabled;
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
struct AttackModeText;

/// Component for level progress display
#[derive(Component)]
struct LevelProgressText;

/// Component for current level display
#[derive(Component)]
struct CurrentLevelText;

/// Component for level completion overlay
#[derive(Component)]
struct LevelCompleteOverlay;

/// Component for level completion text elements
#[derive(Component)]
struct LevelCompleteText;

/// Component for level completion stars display
#[derive(Component)]
struct LevelCompleteStars;

/// Component for level completion continue button
#[derive(Component)]
struct LevelCompleteContinueButton;

/// Button for pausing the game
#[derive(Component)]
struct PauseButton;

/// Button for switching attack mode
#[derive(Component)]
struct AttackModeButton;

/// Button for toggling music
#[derive(Component)]
struct MusicButton;

/// Component for visual slash effect
#[derive(Component)]
pub struct SlashEffect {
    timer: Timer,
}

/// Component for delayed slash effect (used in double slash)
#[derive(Component)]
pub struct DelayedSlashEffect {
    delay_timer: Timer,
    slash_start: Vec2,
    slash_end: Vec2,
}

/// Marker component for dynamic font scaling
#[derive(Component)]
struct DynamicFontSize {
    base_size: f32,
}

/// Initialize game resources
fn setup_game_resources(mut commands: Commands) {
    commands.insert_resource(GameData::new());

    // Initialize level session and start it fresh
    // This ensures a clean start whether the resource exists or not
    let mut level_session = crate::levels::LevelSession::default();
    level_session.start();
    commands.insert_resource(level_session);

    info!("Game started with fresh level session!");
}

/// Calculate responsive font size based on viewport dimensions
fn calculate_font_size(base_size: f32, windows: &Query<&Window>) -> f32 {
    if let Ok(window) = windows.single() {
        let min_dimension = window.width().min(window.height());
        // Scale font based on the smaller dimension for consistency across orientations
        let scale_factor = (min_dimension / 800.0).clamp(0.6, 1.5);
        (base_size * scale_factor).round()
    } else {
        base_size
    }
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
            // Very top section with current level display
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Vh(6.0), // Small height for level display
                        padding: UiRect::all(Val::Vw(UI_PADDING)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)), // Darker background for distinction
                ))
                .with_children(|parent| {
                    // Current level display
                    parent.spawn((
                        Text::new("Level 1"),
                        TextFont { font_size: 22.0, ..default() },
                        TextColor(Color::srgb(1.0, 1.0, 0.8)), // Light yellow color
                        CurrentLevelText,
                        DynamicFontSize { base_size: 22.0 },
                    ));
                });

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
                        DynamicFontSize { base_size: 24.0 },
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
                                DynamicFontSize { base_size: 20.0 },
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
                        DynamicFontSize { base_size: 20.0 },
                    ));

                    // Attack mode display
                    parent.spawn((
                        Text::new("Click"),
                        TextFont { font_size: 18.0, ..default() },
                        TextColor(Color::srgb(0.9, 0.7, 0.3)),
                        AttackModeText,
                        DynamicFontSize { base_size: 18.0 },
                    ));

                    // Level progress display
                    parent.spawn((
                        Text::new("Progress: 0%"),
                        TextFont { font_size: 18.0, ..default() },
                        TextColor(Color::srgb(0.7, 0.9, 0.7)),
                        LevelProgressText,
                        DynamicFontSize { base_size: 18.0 },
                    ));
                });

            // Middle game area where gameplay happens
            parent.spawn((Node {
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                ..default()
            },));

            // Bottom UI panel with instructions and mobile-friendly buttons
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Vh(BOTTOM_UI_HEIGHT),
                        padding: UiRect::all(Val::Vw(UI_PADDING)),
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(UI_BACKGROUND_COLOR),
                ))
                .with_children(|parent| {
                    // Left side: Instructions with icon
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
                                    width: Val::Px(35.0),
                                    height: Val::Px(35.0),
                                    ..default()
                                },
                            ));
                            parent.spawn((
                                Text::new("Q: Pause  |  Tap buttons or dandelions!"),
                                TextFont { font_size: 15.0, ..default() },
                                TextColor(Color::srgb(0.8, 0.8, 0.8)),
                                DynamicFontSize { base_size: 15.0 },
                            ));
                        });

                    // Right side: Mobile control buttons
                    parent
                        .spawn((Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(10.0),
                            ..default()
                        },))
                        .with_children(|parent| {
                            // Pause button
                            parent
                                .spawn((
                                    Button,
                                    Node {
                                        width: Val::Px(90.0),
                                        height: Val::Px(45.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.4, 0.4, 0.6)),
                                    BorderRadius::all(Val::Px(8.0)),
                                    PauseButton,
                                    GameEntity,
                                ))
                                .with_children(|parent| {
                                    parent.spawn((
                                        Text::new("Pause"),
                                        TextFont { font_size: 16.0, ..default() },
                                        TextColor(Color::WHITE),
                                        DynamicFontSize { base_size: 16.0 },
                                    ));
                                });

                            // Attack mode toggle button
                            parent
                                .spawn((
                                    Button,
                                    Node {
                                        width: Val::Px(120.0),
                                        height: Val::Px(45.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.6, 0.4, 0.4)),
                                    BorderRadius::all(Val::Px(8.0)),
                                    AttackModeButton,
                                    GameEntity,
                                ))
                                .with_children(|parent| {
                                    parent.spawn((
                                        Text::new("Click"),
                                        TextFont { font_size: 16.0, ..default() },
                                        TextColor(Color::WHITE),
                                        DynamicFontSize { base_size: 16.0 },
                                    ));
                                });

                            // Music toggle button
                            parent
                                .spawn((
                                    Button,
                                    Node {
                                        width: Val::Px(90.0),
                                        height: Val::Px(45.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.4, 0.6, 0.4)),
                                    BorderRadius::all(Val::Px(8.0)),
                                    MusicButton,
                                    GameEntity,
                                ))
                                .with_children(|parent| {
                                    parent.spawn((
                                        Text::new("Music ON"),
                                        TextFont { font_size: 16.0, ..default() },
                                        TextColor(Color::WHITE),
                                        DynamicFontSize { base_size: 16.0 },
                                    ));
                                });
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
    mut game_data: ResMut<GameData>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyQ) {
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

    // Toggle slash mode with Tab key
    if keyboard_input.just_pressed(KeyCode::Tab) {
        game_data.toggle_slash_mode();
        let mode_text = if game_data.slash_mode { "slash" } else { "click" };
        info!("Switched to {} mode", mode_text);
    }
}

/// Handle mobile-friendly button interactions
fn handle_button_interactions(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            Option<&PauseButton>,
            Option<&AttackModeButton>,
            Option<&MusicButton>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    pause_state: Res<State<PauseState>>,
    mut next_pause_state: ResMut<NextState<PauseState>>,
    mut next_pause_menu_state: ResMut<NextState<PauseMenuState>>,
    mut game_data: ResMut<GameData>,
    music_query: Query<&AudioSink, With<Level1Music>>,
) {
    for (interaction, mut color, pause_button, attack_mode_button, music_button) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                if pause_button.is_some() {
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

                if attack_mode_button.is_some() {
                    game_data.toggle_slash_mode();
                    info!("Switched to {} mode", if game_data.slash_mode { "slash" } else { "click" });
                }

                if music_button.is_some() {
                    game_data.toggle_music();
                    if let Ok(sink) = music_query.single() {
                        if game_data.music_enabled {
                            sink.play();
                        } else {
                            sink.pause();
                        }
                    }
                    info!("Music toggled: {}", if game_data.music_enabled { "ON" } else { "OFF" });
                }
            }
            Interaction::Hovered => {
                if pause_button.is_some() {
                    *color = BackgroundColor(Color::srgb(0.5, 0.5, 0.7));
                } else if attack_mode_button.is_some() {
                    *color = BackgroundColor(Color::srgb(0.7, 0.5, 0.5));
                } else if music_button.is_some() {
                    *color = BackgroundColor(Color::srgb(0.5, 0.7, 0.5));
                }
            }
            Interaction::None => {
                if pause_button.is_some() {
                    *color = BackgroundColor(Color::srgb(0.4, 0.4, 0.6));
                } else if attack_mode_button.is_some() {
                    *color = BackgroundColor(Color::srgb(0.6, 0.4, 0.4));
                } else if music_button.is_some() {
                    *color = BackgroundColor(Color::srgb(0.4, 0.6, 0.4));
                }
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
fn update_score_display(
    game_data: &GameData,
    mut score_query: Query<
        &mut Text,
        (
            With<ScoreText>,
            Without<ComboText>,
            Without<CurbAppealText>,
            Without<AttackModeText>,
            Without<LevelProgressText>,
            Without<CurrentLevelText>,
        ),
    >,
) {
    if let Ok(mut text) = score_query.single_mut() {
        **text = format!("Score: {}", game_data.score);
    }
}

/// Update combo display
fn update_combo_display(
    game_data: &GameData,
    mut combo_query: Query<
        &mut Text,
        (
            With<ComboText>,
            Without<ScoreText>,
            Without<CurbAppealText>,
            Without<AttackModeText>,
            Without<LevelProgressText>,
            Without<CurrentLevelText>,
        ),
    >,
) {
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
    mut curb_appeal_query: Query<
        &mut Text,
        (
            With<CurbAppealText>,
            Without<ScoreText>,
            Without<ComboText>,
            Without<AttackModeText>,
            Without<LevelProgressText>,
            Without<CurrentLevelText>,
        ),
    >,
) {
    if let Ok(mut text) = curb_appeal_query.single_mut() {
        let curb_appeal = calculate_curb_appeal(&dandelion_query);
        **text = format!("Curb Appeal: {}%", curb_appeal);
    }
}

/// Update attack mode display
fn update_attack_mode_display(
    game_data: &GameData,
    level_data: &crate::levels::LevelData,
    mut mode_query: Query<
        &mut Text,
        (
            With<AttackModeText>,
            Without<ScoreText>,
            Without<ComboText>,
            Without<CurbAppealText>,
            Without<LevelProgressText>,
            Without<CurrentLevelText>,
        ),
    >,
) {
    if let Ok(mut text) = mode_query.single_mut() {
        let mode_text = if game_data.slash_mode {
            let total_stars = level_data.get_total_stars();
            if total_stars >= 15 {
                "Extended Double Slash"
            } else if total_stars >= 9 {
                "Double Slash"
            } else {
                "Slash"
            }
        } else {
            "Click"
        };
        **text = format!("Mode: {}", mode_text);
    }
}

/// Update level progress display
fn update_level_progress_display(
    game_data: &GameData,
    level_data: &crate::levels::LevelData,
    mut progress_query: Query<
        &mut Text,
        (
            With<LevelProgressText>,
            Without<ScoreText>,
            Without<ComboText>,
            Without<CurbAppealText>,
            Without<AttackModeText>,
            Without<CurrentLevelText>,
        ),
    >,
) {
    if let Ok(mut text) = progress_query.single_mut() {
        if let Some(current_level) = level_data.get_current_level() {
            let progress = (game_data.score as f32 / current_level.target_points as f32 * 100.0).min(100.0);
            **text = format!("Target: {} | Progress: {:.0}%", current_level.target_points, progress);
        } else {
            **text = format!("Progress: {:.0}%", (game_data.score as f32).min(100.0));
        }
    }
}

/// Update current level display
fn update_current_level_display(
    level_data: &crate::levels::LevelData,
    mut level_query: Query<
        &mut Text,
        (
            With<CurrentLevelText>,
            Without<ScoreText>,
            Without<ComboText>,
            Without<CurbAppealText>,
            Without<AttackModeText>,
            Without<LevelProgressText>,
        ),
    >,
) {
    if let Ok(mut text) = level_query.single_mut() {
        if let Some(current_level) = level_data.get_current_level() {
            **text = format!("Level {} - {}", current_level.id, current_level.name);
        } else {
            **text = format!("Level {}", level_data.current_level);
        }
    }
}

/// Update game UI elements
fn update_ui(
    game_data: Res<GameData>,
    level_data: Res<crate::levels::LevelData>,
    score_query: Query<
        &mut Text,
        (
            With<ScoreText>,
            Without<ComboText>,
            Without<CurbAppealText>,
            Without<AttackModeText>,
            Without<LevelProgressText>,
            Without<CurrentLevelText>,
        ),
    >,
    combo_query: Query<
        &mut Text,
        (
            With<ComboText>,
            Without<ScoreText>,
            Without<CurbAppealText>,
            Without<AttackModeText>,
            Without<LevelProgressText>,
            Without<CurrentLevelText>,
        ),
    >,
    combo_timer_bar_query: Query<&mut Node, With<ComboTimerBar>>,
    curb_appeal_query: Query<
        &mut Text,
        (
            With<CurbAppealText>,
            Without<ScoreText>,
            Without<ComboText>,
            Without<AttackModeText>,
            Without<LevelProgressText>,
            Without<CurrentLevelText>,
        ),
    >,
    mode_query: Query<
        &mut Text,
        (
            With<AttackModeText>,
            Without<ScoreText>,
            Without<ComboText>,
            Without<CurbAppealText>,
            Without<LevelProgressText>,
            Without<CurrentLevelText>,
        ),
    >,
    progress_query: Query<
        &mut Text,
        (
            With<LevelProgressText>,
            Without<ScoreText>,
            Without<ComboText>,
            Without<CurbAppealText>,
            Without<AttackModeText>,
            Without<CurrentLevelText>,
        ),
    >,
    level_query: Query<
        &mut Text,
        (
            With<CurrentLevelText>,
            Without<ScoreText>,
            Without<ComboText>,
            Without<CurbAppealText>,
            Without<AttackModeText>,
            Without<LevelProgressText>,
        ),
    >,
    dandelion_query: Query<&crate::enemies::Dandelion>,
) {
    update_score_display(&game_data, score_query);
    update_combo_display(&game_data, combo_query);
    update_combo_timer_display(&game_data, combo_timer_bar_query);
    update_curb_appeal_display(dandelion_query, curb_appeal_query);
    update_attack_mode_display(&game_data, &level_data, mode_query);
    update_level_progress_display(&game_data, &level_data, progress_query);
    update_current_level_display(&level_data, level_query);
}

/// Update mobile button text to match current mode
fn update_button_text(
    game_data: Res<GameData>,
    level_data: Res<crate::levels::LevelData>,
    attack_mode_button_query: Query<&Children, With<AttackModeButton>>,
    music_button_query: Query<&Children, With<MusicButton>>,
    mut text_query: Query<&mut Text>,
) {
    let mode_text = if game_data.slash_mode {
        let total_stars = level_data.get_total_stars();
        if total_stars >= 15 {
            "2x Slash+"
        } else if total_stars >= 9 {
            "2x Slash"
        } else {
            "Slash"
        }
    } else {
        "Click"
    };

    for children in attack_mode_button_query.iter() {
        for child in children.iter() {
            if let Ok(mut text) = text_query.get_mut(child) {
                **text = mode_text.to_string();
            }
        }
    }

    let music_text = if game_data.music_enabled { "Music ON" } else { "Music OFF" };

    for children in music_button_query.iter() {
        for child in children.iter() {
            if let Ok(mut text) = text_query.get_mut(child) {
                **text = music_text.to_string();
            }
        }
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

/// Update slash effects
fn update_slash_effects(mut commands: Commands, mut slash_query: Query<(Entity, &mut SlashEffect, &mut Sprite)>, time: Res<Time>) {
    for (entity, mut slash_effect, mut sprite) in slash_query.iter_mut() {
        slash_effect.timer.tick(time.delta());

        // Fade out the slash effect over time
        let progress = slash_effect.timer.elapsed_secs() / slash_effect.timer.duration().as_secs_f32();
        sprite.color.set_alpha(1.0 - progress);

        if slash_effect.timer.finished() {
            if let Ok(mut ec) = commands.get_entity(entity) {
                ec.despawn();
            }
        }
    }
}

/// Update delayed slash effects
fn update_delayed_slash_effects(
    mut commands: Commands,
    mut delayed_query: Query<(Entity, &mut DelayedSlashEffect)>,
    mut dandelion_query: Query<(Entity, &mut crate::enemies::Dandelion, &Transform)>,
    time: Res<Time>,
    game_assets: Res<crate::GameAssets>,
    mut game_data: ResMut<GameData>,
) {
    for (entity, mut delayed_effect) in delayed_query.iter_mut() {
        delayed_effect.delay_timer.tick(time.delta());

        if delayed_effect.delay_timer.just_finished() {
            // Spawn the actual slash effect
            spawn_slash_effect(&mut commands, delayed_effect.slash_start, delayed_effect.slash_end);

            // Process delayed slash damage and only play sound if enemies are hit
            let _hit_count = crate::enemies::process_delayed_slash_damage(
                &mut commands,
                &mut game_data,
                &game_assets,
                &mut dandelion_query,
                delayed_effect.slash_start,
                delayed_effect.slash_end,
            );

            // Remove the delayed effect entity
            if let Ok(mut ec) = commands.get_entity(entity) {
                ec.despawn();
            }
        }
    }
}

/// Spawn a visual slash effect
pub fn spawn_slash_effect(commands: &mut Commands, start_pos: Vec2, end_pos: Vec2) {
    let direction = end_pos - start_pos;
    let length = direction.length();
    let angle = direction.y.atan2(direction.x);
    let center = (start_pos + end_pos) / 2.0;

    commands.spawn((
        Sprite {
            color: Color::srgba(1.0, 1.0, 0.0, 0.8), // Bright yellow slash
            ..default()
        },
        Transform::from_translation(Vec3::new(center.x, center.y, 20.0))
            .with_rotation(Quat::from_rotation_z(angle))
            .with_scale(Vec3::new(length, 4.0, 1.0)), // 4 pixel wide line
        SlashEffect {
            timer: Timer::from_seconds(0.2, TimerMode::Once), // 200ms duration
        },
        GameEntity, // Add GameEntity component for proper cleanup
    ));
}

/// Spawn a delayed slash effect for double slash
pub fn spawn_delayed_slash_effect(commands: &mut Commands, start_pos: Vec2, end_pos: Vec2, delay: f32) {
    commands.spawn((
        DelayedSlashEffect {
            delay_timer: Timer::from_seconds(delay, TimerMode::Once),
            slash_start: start_pos,
            slash_end: end_pos,
        },
        GameEntity,
    ));
}

/// Update dynamic font sizes based on window dimensions
fn update_dynamic_font_sizes(windows: Query<&Window>, mut text_query: Query<(&mut TextFont, &DynamicFontSize)>) {
    for (mut text_font, dynamic_size) in &mut text_query {
        text_font.font_size = calculate_font_size(dynamic_size.base_size, &windows);
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

    // Remove level session resource to ensure fresh start on restart
    commands.remove_resource::<crate::levels::LevelSession>();

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

fn play_level1_music(asset_server: Res<AssetServer>, mut commands: Commands, game_data: Res<GameData>) {
    let music: Handle<AudioSource> = asset_server.load("audio/level1.wav");
    commands.spawn((
        AudioPlayer(music),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Loop,
            paused: !game_data.music_enabled,
            ..default()
        },
        Level1Music,
        crate::SoundEntity,
    ));
}

/// Handle level start events when level is selected from pause menu
fn handle_level_start_events(
    mut commands: Commands,
    mut level_start_events: EventReader<LevelStartEvent>,
    mut game_data: ResMut<GameData>,
    mut level_data: ResMut<LevelData>,
    enemy_entities: Query<Entity, With<crate::enemies::EnemyEntity>>,
    powerup_entities: Query<Entity, With<crate::powerups::PowerupEntity>>,
    rabbit_entities: Query<Entity, With<crate::powerups::Rabbit>>,
    fire_entities: Query<Entity, With<crate::powerups::FireIgnition>>,
    mut level_complete_overlay_query: Query<&mut Visibility, With<LevelCompleteOverlay>>,
) {
    for event in level_start_events.read() {
        // Hide level complete overlay if visible
        for mut visibility in &mut level_complete_overlay_query {
            *visibility = Visibility::Hidden;
        }

        // Clear all enemies and powerups from the screen
        for entity in &enemy_entities {
            if let Ok(mut ec) = commands.get_entity(entity) {
                ec.despawn();
            }
        }
        for entity in &powerup_entities {
            if let Ok(mut ec) = commands.get_entity(entity) {
                ec.despawn();
            }
        }

        // Clear all rabbits from the screen
        for entity in &rabbit_entities {
            if let Ok(mut ec) = commands.get_entity(entity) {
                ec.despawn();
            }
        }

        // Clear all fire entities from the screen
        for entity in &fire_entities {
            if let Ok(mut ec) = commands.get_entity(entity) {
                ec.despawn();
            }
        }

        // Reset game data for the selected level
        game_data.score = 0;
        game_data.combo = 0;
        game_data.combo_timer.reset();
        game_data.dandelion_count = 0;

        // Set the current level to the selected level
        level_data.set_current_level(event.level_id);

        info!("Game state reset for level {}", event.level_id);
    }
}

/// Handle level completion events and transitions
fn handle_level_completion_events(
    mut commands: Commands,
    mut level_complete_events: EventReader<LevelCompleteEvent>,
    mut level_complete_overlay_query: Query<&mut Visibility, With<LevelCompleteOverlay>>,
    mut level_complete_text_query: Query<&mut Text, With<LevelCompleteText>>,
    mut level_complete_stars_query: Query<(Entity, Option<&Children>), With<LevelCompleteStars>>,
    game_assets: Res<crate::GameAssets>,
) {
    for event in level_complete_events.read() {
        // Show level complete overlay
        for mut visibility in &mut level_complete_overlay_query {
            *visibility = Visibility::Visible;
        }

        // Update level complete text with completion info
        for mut text in &mut level_complete_text_query {
            text.0 = format!(
                "Level {} Complete!\n\nScore: {}\nTime: {:.1}s",
                event.level_id,
                event.final_score,
                event.completion_time.as_secs_f32()
            );
        }

        // Update stars display - only show earned stars
        for (stars_entity, children) in &mut level_complete_stars_query {
            let current_star_count = children.map(|c| c.len()).unwrap_or(0);

            if current_star_count != event.stars_earned as usize {
                // Clear existing stars if count is different
                if let Some(children) = children {
                    for child in children.iter() {
                        commands.entity(child).despawn();
                    }
                }

                // Add correct number of stars using image assets
                commands.entity(stars_entity).with_children(|parent| {
                    for _i in 0..event.stars_earned {
                        parent.spawn((
                            Node {
                                width: Val::Px(30.0),
                                height: Val::Px(30.0),
                                margin: UiRect::all(Val::Px(5.0)),
                                ..default()
                            },
                            ImageNode::new(game_assets.star_complete.clone()),
                        ));
                    }
                });
            }
        }

        info!("Level completion overlay shown for level {}", event.level_id);
    }
}

/// Handle interactions with the level completion overlay
fn handle_level_completion_interactions(
    mut commands: Commands,
    mut interaction_query: Query<(&Interaction, &mut BackgroundColor, Option<&LevelCompleteContinueButton>), (Changed<Interaction>, With<Button>)>,
    mut game_data: ResMut<GameData>,
    mut level_complete_overlay_query: Query<&mut Visibility, With<LevelCompleteOverlay>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut level_data: ResMut<LevelData>,
    mut level_start_events: EventWriter<LevelStartEvent>,
    enemy_entities: Query<Entity, With<crate::enemies::EnemyEntity>>,
    powerup_entities: Query<Entity, With<crate::powerups::PowerupEntity>>,
    rabbit_entities: Query<Entity, With<crate::powerups::Rabbit>>,
    fire_entities: Query<Entity, With<crate::powerups::FireIgnition>>,
) {
    for (interaction, mut color, continue_button) in &mut interaction_query {
        if continue_button.is_some() {
            match *interaction {
                Interaction::Pressed => {
                    // Hide level complete overlay
                    for mut visibility in &mut level_complete_overlay_query {
                        *visibility = Visibility::Hidden;
                    }

                    // Clear all enemies and powerups from the screen for next level
                    for entity in &enemy_entities {
                        if let Ok(mut ec) = commands.get_entity(entity) {
                            ec.despawn();
                        }
                    }
                    for entity in &powerup_entities {
                        if let Ok(mut ec) = commands.get_entity(entity) {
                            ec.despawn();
                        }
                    }

                    // Clear all rabbits from the screen
                    for entity in &rabbit_entities {
                        if let Ok(mut ec) = commands.get_entity(entity) {
                            ec.despawn();
                        }
                    }

                    // Clear all fire entities from the screen
                    for entity in &fire_entities {
                        if let Ok(mut ec) = commands.get_entity(entity) {
                            ec.despawn();
                        }
                    }

                    // Reset game data for the next level
                    game_data.score = 0;
                    game_data.combo = 0;
                    game_data.combo_timer.reset();
                    game_data.dandelion_count = 0;

                    // Check if there's a next level
                    let current_level_id = level_data.current_level;
                    let next_level_id = current_level_id + 1;

                    if level_data.get_level(next_level_id).is_some() {
                        // Go to next level
                        level_data.set_current_level(next_level_id);
                        // Emit level start event for the new level
                        level_start_events.write(LevelStartEvent { level_id: next_level_id });
                        // Stay in playing state to continue with next level
                        info!("Advancing to level {}", next_level_id);
                    } else {
                        // No more levels, return to main menu
                        next_state.set(GameState::Menu);
                        info!("All levels completed, returning to main menu");
                    }
                }
                Interaction::Hovered => {
                    *color = BackgroundColor(Color::srgb(0.3, 0.8, 0.3));
                }
                Interaction::None => {
                    *color = BackgroundColor(Color::srgb(0.2, 0.7, 0.2));
                }
            }
        }
    }
}

/// Setup the level completion overlay UI
fn setup_level_complete_overlay(mut commands: Commands, _asset_server: Res<AssetServer>) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            Visibility::Hidden,
            LevelCompleteOverlay,
            GameEntity,
        ))
        .with_children(|parent| {
            // Background panel - responsive sizing for mobile landscape
            parent
                .spawn((
                    Node {
                        width: Val::Vw(85.0),
                        max_width: Val::Px(600.0),
                        height: Val::Vh(70.0),
                        max_height: Val::Px(450.0),
                        min_height: Val::Vh(50.0),
                        padding: UiRect::all(Val::VMin(3.0)),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.95)),
                    BorderRadius::all(Val::VMin(2.0)),
                ))
                .with_children(|parent| {
                    // Level complete text with responsive font scaling
                    parent.spawn((
                        Text::new("Level Complete!"),
                        TextFont { font_size: 36.0, ..default() }, // Smaller for mobile
                        TextColor(Color::WHITE),
                        LevelCompleteText,
                        DynamicFontSize { base_size: 36.0 },
                        Node {
                            margin: UiRect::bottom(Val::Vh(2.0)),
                            ..default()
                        },
                    ));

                    // Stars display with responsive sizing
                    parent.spawn((
                        Node {
                            width: Val::Vw(25.0),
                            max_width: Val::Px(200.0),
                            height: Val::Vh(8.0),
                            max_height: Val::Px(50.0),
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            margin: UiRect::bottom(Val::Vh(3.0)),
                            ..default()
                        },
                        LevelCompleteStars,
                    ));

                    // Continue button with responsive sizing
                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Vw(25.0),
                                max_width: Val::Px(200.0),
                                min_width: Val::Px(150.0),
                                height: Val::Vh(8.0),
                                max_height: Val::Px(60.0),
                                min_height: Val::Px(45.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                margin: UiRect::top(Val::Vh(2.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.2, 0.7, 0.2)),
                            BorderRadius::all(Val::VMin(1.5)),
                            LevelCompleteContinueButton,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("Continue"),
                                TextFont { font_size: 20.0, ..default() }, // Responsive font size
                                TextColor(Color::WHITE),
                                DynamicFontSize { base_size: 20.0 },
                            ));
                        });
                });
        });
}
