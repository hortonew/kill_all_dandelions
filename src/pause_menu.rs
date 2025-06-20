use bevy::{
    input::{
        mouse::{MouseScrollUnit, MouseWheel},
        touch::{TouchInput, TouchPhase},
    },
    picking::hover::HoverMap,
    prelude::*,
    ui::ScrollPosition,
};

use crate::GameState;
use crate::levels::{LevelData, LevelStartEvent};

/// Plugin for handling the pause menu
pub struct PauseMenuPlugin;

impl Plugin for PauseMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<PauseState>()
            .init_state::<PauseMenuState>()
            .init_resource::<TouchScrollState>()
            .add_systems(OnEnter(PauseState::Paused), (setup_pause_menu_on_pause, pause_sounds))
            .add_systems(OnExit(PauseState::Paused), (cleanup_pause_menu, resume_sounds))
            .add_systems(
                Update,
                (handle_pause_input, pause_menu_interactions).run_if(in_state(PauseState::Paused).and(in_state(PauseMenuState::PauseMenu))),
            )
            .add_systems(
                Update,
                (handle_powerup_help_input, powerup_help_interactions).run_if(in_state(PauseState::Paused).and(in_state(PauseMenuState::PowerupHelp))),
            )
            .add_systems(
                Update,
                (handle_level_selection_input, level_selection_interactions, update_star_displays)
                    .run_if(in_state(PauseState::Paused).and(in_state(PauseMenuState::LevelSelection))),
            )
            .add_systems(Update, switch_pause_menu_content.run_if(in_state(PauseState::Paused)))
            .add_systems(Update, update_dynamic_font_sizes)
            .add_systems(Update, update_scroll_position)
            .add_systems(
                Update,
                handle_touch_scroll.run_if(in_state(PauseState::Paused).and(in_state(PauseMenuState::LevelSelection))),
            );
    }
}

/// Local pause state within the playing state
#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PauseState {
    Playing,
    Paused,
}

/// Sub-state for different pause menu screens
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PauseMenuState {
    PauseMenu,
    PowerupHelp,
    LevelSelection,
}

impl Default for PauseState {
    fn default() -> Self {
        Self::Playing
    }
}

impl Default for PauseMenuState {
    fn default() -> Self {
        Self::PauseMenu
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
    PowerupHelp,
    LevelSelection,
}

/// Powerup help menu button types
#[derive(Component)]
enum PowerupHelpButton {
    Back,
}

/// Level selection menu button types
#[derive(Component)]
enum LevelSelectionButton {
    Back,
    LevelButton(u32),
}

/// Component for star display in level selection
#[derive(Component)]
struct StarDisplay {
    level_id: u32,
    star_index: u32, // 0, 1, or 2 for the three stars
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

/// Marker component for dynamic font scaling
#[derive(Component)]
struct DynamicFontSize {
    base_size: f32,
}

/// Handle input while paused
fn handle_pause_input(keyboard_input: Res<ButtonInput<KeyCode>>, mut next_pause_state: ResMut<NextState<PauseState>>) {
    if keyboard_input.just_pressed(KeyCode::KeyQ) {
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
                        width: Val::Vw(60.0),
                        max_width: Val::Px(400.0),
                        height: Val::Vh(60.0),
                        max_height: Val::Px(350.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        padding: UiRect::all(Val::Vh(3.0)),
                        row_gap: Val::Vh(3.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                    BorderRadius::all(Val::Px(10.0)),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Game Paused"),
                        TextFont { font_size: 28.0, ..default() },
                        TextColor(Color::WHITE),
                        DynamicFontSize { base_size: 28.0 },
                    ));

                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Vw(40.0),
                                max_width: Val::Px(250.0),
                                height: Val::Vh(7.0),
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
                                TextFont { font_size: 18.0, ..default() },
                                TextColor(Color::WHITE),
                                DynamicFontSize { base_size: 18.0 },
                            ));
                        });

                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Vw(40.0),
                                max_width: Val::Px(250.0),
                                height: Val::Vh(7.0),
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
                                TextFont { font_size: 18.0, ..default() },
                                TextColor(Color::WHITE),
                                DynamicFontSize { base_size: 18.0 },
                            ));
                        });

                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Vw(40.0),
                                max_width: Val::Px(250.0),
                                height: Val::Vh(7.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.3, 0.5, 0.7)),
                            BorderRadius::all(Val::Px(5.0)),
                            PauseMenuButton::PowerupHelp,
                            PauseMenuEntity,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("Powerup Help"),
                                TextFont { font_size: 18.0, ..default() },
                                TextColor(Color::WHITE),
                                DynamicFontSize { base_size: 18.0 },
                            ));
                        });

                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Vw(40.0),
                                max_width: Val::Px(250.0),
                                height: Val::Vh(7.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.4, 0.6, 0.2)),
                            BorderRadius::all(Val::Px(5.0)),
                            PauseMenuButton::LevelSelection,
                            PauseMenuEntity,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("Level Selection"),
                                TextFont { font_size: 18.0, ..default() },
                                TextColor(Color::WHITE),
                                DynamicFontSize { base_size: 18.0 },
                            ));
                        });
                });
        });
}

/// Handle pause menu button interactions
fn pause_menu_interactions(
    mut interaction_query: Query<(&Interaction, &mut BackgroundColor, &PauseMenuButton), (Changed<Interaction>, With<Button>)>,
    mut next_pause_state: ResMut<NextState<PauseState>>,
    mut next_pause_menu_state: ResMut<NextState<PauseMenuState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut level_data: ResMut<LevelData>,
) {
    for (interaction, mut color, button_type) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => match button_type {
                PauseMenuButton::Resume => {
                    next_pause_state.set(PauseState::Playing);
                }
                PauseMenuButton::Restart => {
                    // Reset all level progress before returning to menu
                    level_data.reset_all_progress();
                    next_pause_state.set(PauseState::Playing);
                    next_game_state.set(GameState::Menu);
                }
                PauseMenuButton::PowerupHelp => {
                    next_pause_menu_state.set(PauseMenuState::PowerupHelp);
                }
                PauseMenuButton::LevelSelection => {
                    next_pause_menu_state.set(PauseMenuState::LevelSelection);
                }
            },
            Interaction::Hovered => match button_type {
                PauseMenuButton::Resume => *color = BackgroundColor(Color::srgb(0.5, 0.5, 0.5)),
                PauseMenuButton::Restart => *color = BackgroundColor(Color::srgb(0.8, 0.4, 0.4)),
                PauseMenuButton::PowerupHelp => *color = BackgroundColor(Color::srgb(0.4, 0.6, 0.8)),
                PauseMenuButton::LevelSelection => *color = BackgroundColor(Color::srgb(0.6, 0.8, 0.4)),
            },
            Interaction::None => match button_type {
                PauseMenuButton::Resume => *color = BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                PauseMenuButton::Restart => *color = BackgroundColor(Color::srgb(0.6, 0.3, 0.3)),
                PauseMenuButton::PowerupHelp => *color = BackgroundColor(Color::srgb(0.3, 0.5, 0.7)),
                PauseMenuButton::LevelSelection => *color = BackgroundColor(Color::srgb(0.4, 0.6, 0.2)),
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

/// Handle input while in powerup help screen
fn handle_powerup_help_input(keyboard_input: Res<ButtonInput<KeyCode>>, mut next_pause_menu_state: ResMut<NextState<PauseMenuState>>) {
    if keyboard_input.just_pressed(KeyCode::KeyQ) {
        next_pause_menu_state.set(PauseMenuState::PauseMenu);
    }
}

/// Setup powerup help menu UI
fn setup_powerup_help_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect::all(Val::VMin(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            PauseMenuEntity,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Vw(85.0),
                        max_width: Val::Px(700.0),
                        min_width: Val::Px(300.0),
                        height: Val::Vh(80.0),
                        max_height: Val::Px(600.0),
                        min_height: Val::Vh(60.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::FlexStart,
                        padding: UiRect::all(Val::VMin(2.0)),
                        row_gap: Val::VMin(2.0),
                        overflow: Overflow::scroll_y(),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                    BorderRadius::all(Val::VMin(1.5)),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Powerup Help"), 
                        TextFont { font_size: 24.0, ..default() },
                        TextColor(Color::WHITE),
                        DynamicFontSize { base_size: 24.0 },
                    ));

                    // Powerup table container
                    parent
                        .spawn((Node {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Vh(1.5),
                            ..default()
                        },))                        .with_children(|parent| {
                            // Bunny powerup row
                            parent
                                .spawn((
                                    Node {
                                        width: Val::Percent(100.0),
                                        min_height: Val::Vh(10.0),
                                        flex_direction: FlexDirection::Row,
                                        align_items: AlignItems::Center,
                                        column_gap: Val::Vw(3.0),
                                        padding: UiRect::all(Val::Vh(1.5)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                                    BorderRadius::all(Val::Px(5.0)),
                                ))
                                .with_children(|parent| {
                                    // Powerup icon
                                    parent.spawn((
                                        ImageNode::new(asset_server.load("bunny.png")),
                                        Node {
                                            width: Val::Vw(8.0),
                                            height: Val::Vw(8.0),
                                            max_width: Val::Px(60.0),
                                            max_height: Val::Px(60.0),
                                            ..default()
                                        },
                                    ));

                                    // Description text
                                    parent
                                        .spawn((
                                            Node {
                                                flex_direction: FlexDirection::Column,
                                                flex_grow: 1.0,
                                                row_gap: Val::Vh(0.5),
                                                ..default()
                                            },
                                        ))
                                        .with_children(|parent| {
                                            parent.spawn((
                                                Text::new("Bunny"),
                                                TextFont {
                                                    font_size: 16.0,
                                                    ..default()
                                                },
                                                TextColor(Color::srgb(0.9, 0.9, 0.5)),
                                                DynamicFontSize { base_size: 16.0 },
                                            ));

                                            parent.spawn((
                                                Text::new("Spawns 3 rabbits that seek and destroy dandelions. Each rabbit has 3 seconds to eat a dandelion, and eating a least 2 spawns a new rabbit."),
                                                TextFont {
                                                    font_size: 14.0,
                                                    ..default()
                                                },
                                                TextColor(Color::srgb(0.8, 0.8, 0.8)),
                                                DynamicFontSize { base_size: 14.0 },
                                            ));
                                        });
                                });
                            // Flamethrower powerup row
                            parent
                                .spawn((
                                    Node {
                                        width: Val::Percent(100.0),
                                        min_height: Val::Vh(10.0),
                                        flex_direction: FlexDirection::Row,
                                        align_items: AlignItems::Center,
                                        column_gap: Val::Vw(3.0),
                                        padding: UiRect::all(Val::Vh(1.5)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                                    BorderRadius::all(Val::Px(5.0)),
                                ))
                                .with_children(|parent| {
                                    // Powerup icon
                                    parent.spawn((
                                        ImageNode::new(asset_server.load("flamethrower.png")),
                                        Node {
                                            width: Val::Vw(8.0),
                                            height: Val::Vw(8.0),
                                            max_width: Val::Px(60.0),
                                            max_height: Val::Px(60.0),
                                            ..default()
                                        },
                                    ));

                                    // Description text
                                    parent
                                        .spawn((
                                            Node {
                                                flex_direction: FlexDirection::Column,
                                                flex_grow: 1.0,
                                                row_gap: Val::Vh(0.5),
                                                ..default()
                                            },
                                        ))
                                        .with_children(|parent| {
                                            parent.spawn((
                                                Text::new("Flamethrower"),
                                                TextFont {
                                                    font_size: 18.0,
                                                    ..default()
                                                },
                                                TextColor(Color::srgb(0.9, 0.9, 0.5)),
                                                DynamicFontSize { base_size: 18.0 },
                                            ));

                                            parent.spawn((
                                                Text::new("Creates a fire ignition that continuously damages all dandelions within its radius for 2 seconds. Effective against groups of dandelions."),
                                                TextFont {
                                                    font_size: 14.0,
                                                    ..default()
                                                },
                                                TextColor(Color::srgb(0.8, 0.8, 0.8)),
                                                DynamicFontSize { base_size: 14.0 },
                                            ));
                                        });
                                });
                        });

                    // Back button
                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Vw(30.0),
                                max_width: Val::Px(200.0),
                                height: Val::Vh(7.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                margin: UiRect::top(Val::Vh(2.5)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                            BorderRadius::all(Val::Px(5.0)),
                            PowerupHelpButton::Back,
                            PauseMenuEntity,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("Back"), 
                                TextFont { font_size: 20.0, ..default() },
                                TextColor(Color::WHITE),
                                DynamicFontSize { base_size: 20.0 },
                            ));
                        });
                });
        });
}

/// Setup level selection menu UI
fn setup_level_selection_menu(mut commands: Commands, level_data: Res<LevelData>, game_assets: Res<crate::GameAssets>) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect::all(Val::VMin(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            PauseMenuEntity,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Vw(90.0),
                        max_width: Val::Px(900.0),
                        min_width: Val::Px(300.0),
                        height: Val::Vh(85.0),
                        max_height: Val::Px(700.0),
                        min_height: Val::Vh(60.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::FlexStart,
                        padding: UiRect::all(Val::VMin(2.5)),
                        row_gap: Val::VMin(2.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                    BorderRadius::all(Val::VMin(1.5)),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Level Selection"),
                        TextFont { font_size: 28.0, ..default() },
                        TextColor(Color::WHITE),
                        DynamicFontSize { base_size: 28.0 },
                    ));

                    // Level grid container - scrollable with responsive spacing
                    parent
                        .spawn((
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Percent(65.0), // Reduced to ensure scrolling is needed
                                max_height: Val::Px(320.0), // Reduced to 320px to force scrolling with 4+ rows
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(20.0), // Increased gap for better spacing
                                overflow: Overflow::scroll_y(),
                                padding: UiRect::all(Val::Px(15.0)), // Increased padding
                                ..default()
                            },
                            ScrollPosition::default(), // Add ScrollPosition component for proper scrolling
                        ))
                        .with_children(|parent| {
                            let total_levels = level_data.levels.len();
                            let levels_per_row = 3; // Better for mobile landscape
                            let total_rows = total_levels.div_ceil(levels_per_row);

                            // Create level cards in rows of 3 for better mobile compatibility
                            for row in 0..total_rows {
                                parent
                                    .spawn((Node {
                                        width: Val::Percent(100.0),
                                        height: Val::Px(110.0),     // Increased height for better touch targets and scrolling
                                        min_height: Val::Px(100.0), // Increased minimum content height
                                        flex_direction: FlexDirection::Row,
                                        justify_content: JustifyContent::SpaceEvenly,
                                        column_gap: Val::Px(10.0), // Fixed gap
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },))
                                    .with_children(|parent| {
                                        for col in 0..levels_per_row {
                                            let level_id = row * levels_per_row + col + 1;
                                            if level_id <= total_levels {
                                                let level = level_data.get_level(level_id as u32);
                                                let is_unlocked = level_data.is_level_unlocked(level_id as u32);

                                                // Create level card with responsive sizing
                                                parent
                                                    .spawn((
                                                        Button,
                                                        Node {
                                                            width: Val::Percent(30.0),
                                                            height: Val::Px(85.0), // Increased height for better touch targets
                                                            min_height: Val::Px(75.0),
                                                            flex_direction: FlexDirection::Column,
                                                            align_items: AlignItems::Center,
                                                            justify_content: JustifyContent::Center,
                                                            padding: UiRect::all(Val::Px(8.0)),
                                                            ..default()
                                                        },
                                                        BackgroundColor(if is_unlocked {
                                                            Color::srgb(0.4, 0.4, 0.6) // Unlocked
                                                        } else {
                                                            Color::srgb(0.3, 0.3, 0.3) // Locked
                                                        }),
                                                        BorderRadius::all(Val::Px(8.0)),
                                                        LevelSelectionButton::LevelButton(level_id as u32),
                                                        PauseMenuEntity,
                                                    ))
                                                    .with_children(|parent| {
                                                        // Level number/name with responsive font
                                                        if let Some(level) = level {
                                                            parent.spawn((
                                                                Text::new(format!("{}", level_id)),
                                                                TextFont { font_size: 16.0, ..default() },
                                                                TextColor(if is_unlocked { Color::WHITE } else { Color::srgb(0.6, 0.6, 0.6) }),
                                                                DynamicFontSize { base_size: 16.0 },
                                                            ));

                                                            // Level name (smart truncation)
                                                            let display_name = if level.name.len() > 35 {
                                                                format!("{}...", &level.name[..32])
                                                            } else {
                                                                level.name.clone()
                                                            };

                                                            parent.spawn((
                                                                Text::new(display_name),
                                                                TextFont { font_size: 9.0, ..default() },
                                                                TextColor(if is_unlocked {
                                                                    Color::srgb(0.9, 0.9, 0.9)
                                                                } else {
                                                                    Color::srgb(0.5, 0.5, 0.5)
                                                                }),
                                                                DynamicFontSize { base_size: 9.0 },
                                                                Node {
                                                                    margin: UiRect::top(Val::Px(3.0)),
                                                                    ..default()
                                                                },
                                                            ));
                                                        } else {
                                                            parent.spawn((
                                                                Text::new(format!("{}", level_id)),
                                                                TextFont { font_size: 16.0, ..default() },
                                                                TextColor(Color::srgb(0.6, 0.6, 0.6)),
                                                                DynamicFontSize { base_size: 16.0 },
                                                            ));
                                                        }

                                                        // Stars display with responsive spacing
                                                        if is_unlocked {
                                                            parent
                                                                .spawn((Node {
                                                                    flex_direction: FlexDirection::Row,
                                                                    column_gap: Val::Px(4.0),
                                                                    margin: UiRect::top(Val::Px(5.0)),
                                                                    justify_content: JustifyContent::Center,
                                                                    ..default()
                                                                },))
                                                                .with_children(|parent| {
                                                                    for star_index in 0..3 {
                                                                        parent.spawn((
                                                                            Node {
                                                                                width: Val::Px(25.0),
                                                                                height: Val::Px(25.0),
                                                                                ..default()
                                                                            },
                                                                            ImageNode::new(game_assets.star_incomplete.clone()),
                                                                            StarDisplay {
                                                                                level_id: level_id as u32,
                                                                                star_index,
                                                                            },
                                                                        ));
                                                                    }
                                                                });
                                                        }
                                                    });
                                            } else {
                                                // Empty placeholder to maintain grid structure
                                                parent.spawn((Node {
                                                    width: Val::Percent(30.0),
                                                    height: Val::Px(85.0), // Match level card height
                                                    ..default()
                                                },));
                                            }
                                        }
                                    });
                            }
                        });

                    // Back button with responsive sizing
                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Vw(30.0),
                                max_width: Val::Px(200.0),
                                min_width: Val::Px(120.0),
                                height: Val::Vh(7.0),
                                max_height: Val::Px(50.0),
                                min_height: Val::Px(35.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                margin: UiRect::top(Val::Px(20.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                            BorderRadius::all(Val::Px(8.0)),
                            LevelSelectionButton::Back,
                            PauseMenuEntity,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("Back"),
                                TextFont { font_size: 18.0, ..default() },
                                TextColor(Color::WHITE),
                                DynamicFontSize { base_size: 18.0 },
                            ));
                        });
                });
        });
}

/// Update dynamic font sizes based on window dimensions
fn update_dynamic_font_sizes(windows: Query<&Window>, mut text_query: Query<(&mut TextFont, &DynamicFontSize)>) {
    for (mut text_font, dynamic_size) in &mut text_query {
        text_font.font_size = calculate_font_size(dynamic_size.base_size, &windows);
    }
}

/// Update star displays based on level progress
fn update_star_displays(mut star_query: Query<(&mut ImageNode, &StarDisplay)>, level_data: Res<LevelData>, game_assets: Res<crate::GameAssets>) {
    for (mut image_node, star_display) in &mut star_query {
        if let Some(progress) = level_data.level_progress.get((star_display.level_id - 1) as usize) {
            let filled_stars = progress.best_stars;

            // Update star image based on index and progress
            let star_image = if star_display.star_index < filled_stars {
                game_assets.star_complete.clone()
            } else {
                game_assets.star_incomplete.clone()
            };

            image_node.image = star_image;
        }
    }
}

/// Handle input while in level selection screen
fn handle_level_selection_input(keyboard_input: Res<ButtonInput<KeyCode>>, mut next_pause_menu_state: ResMut<NextState<PauseMenuState>>) {
    if keyboard_input.just_pressed(KeyCode::KeyQ) {
        next_pause_menu_state.set(PauseMenuState::PauseMenu);
    }
}

/// Handle level selection button interactions
fn level_selection_interactions(
    mut interaction_query: Query<(&Interaction, &mut BackgroundColor, &LevelSelectionButton), (Changed<Interaction>, With<Button>)>,
    mut next_pause_menu_state: ResMut<NextState<PauseMenuState>>,
    mut next_pause_state: ResMut<NextState<PauseState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut level_start_events: EventWriter<LevelStartEvent>,
    level_data: Res<LevelData>,
) {
    for (interaction, mut color, button_type) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => match button_type {
                LevelSelectionButton::Back => {
                    next_pause_menu_state.set(PauseMenuState::PauseMenu);
                }
                LevelSelectionButton::LevelButton(level_id) => {
                    if level_data.is_level_unlocked(*level_id) {
                        // Start the selected level
                        level_start_events.write(LevelStartEvent { level_id: *level_id });

                        // Resume game and go to playing state
                        next_pause_state.set(PauseState::Playing);
                        next_game_state.set(GameState::Playing);

                        info!("Starting level {}", level_id);
                    }
                }
            },
            Interaction::Hovered => match button_type {
                LevelSelectionButton::Back => *color = BackgroundColor(Color::srgb(0.5, 0.5, 0.5)),
                LevelSelectionButton::LevelButton(level_id) => {
                    if level_data.is_level_unlocked(*level_id) {
                        *color = BackgroundColor(Color::srgb(0.5, 0.5, 0.7));
                    } else {
                        *color = BackgroundColor(Color::srgb(0.4, 0.3, 0.3));
                    }
                }
            },
            Interaction::None => match button_type {
                LevelSelectionButton::Back => *color = BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                LevelSelectionButton::LevelButton(level_id) => {
                    if level_data.is_level_unlocked(*level_id) {
                        *color = BackgroundColor(Color::srgb(0.4, 0.4, 0.6));
                    } else {
                        *color = BackgroundColor(Color::srgb(0.3, 0.3, 0.3));
                    }
                }
            },
        }
    }
}

/// Setup pause menu when entering paused state
fn setup_pause_menu_on_pause(
    pause_menu_state: Res<State<PauseMenuState>>,
    commands: Commands,
    asset_server: Res<AssetServer>,
    level_data: Res<LevelData>,
    game_assets: Res<crate::GameAssets>,
) {
    match pause_menu_state.get() {
        PauseMenuState::PauseMenu => setup_pause_menu(commands),
        PauseMenuState::PowerupHelp => setup_powerup_help_menu(commands, asset_server),
        PauseMenuState::LevelSelection => setup_level_selection_menu(commands, level_data, game_assets),
    }
}

/// Switch pause menu content based on pause menu state changes
fn switch_pause_menu_content(
    pause_menu_state: Res<State<PauseMenuState>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    level_data: Res<LevelData>,
    game_assets: Res<crate::GameAssets>,
    pause_entities: Query<Entity, With<PauseMenuEntity>>,
    mut local_previous_state: Local<Option<PauseMenuState>>,
) {
    let current_state = *pause_menu_state.get();

    if let Some(previous_state) = *local_previous_state {
        if previous_state != current_state {
            // Clean up previous menu
            for entity in &pause_entities {
                commands.entity(entity).despawn();
            }

            // Setup new menu
            match current_state {
                PauseMenuState::PauseMenu => setup_pause_menu(commands),
                PauseMenuState::PowerupHelp => setup_powerup_help_menu(commands, asset_server),
                PauseMenuState::LevelSelection => setup_level_selection_menu(commands, level_data, game_assets),
            }
        }
    }

    *local_previous_state = Some(current_state);
}

/// Handle powerup help menu button interactions
fn powerup_help_interactions(
    mut interaction_query: Query<(&Interaction, &mut BackgroundColor, &PowerupHelpButton), (Changed<Interaction>, With<Button>)>,
    mut next_pause_menu_state: ResMut<NextState<PauseMenuState>>,
) {
    for (interaction, mut color, button_type) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => match button_type {
                PowerupHelpButton::Back => {
                    next_pause_menu_state.set(PauseMenuState::PauseMenu);
                }
            },
            Interaction::Hovered => match button_type {
                PowerupHelpButton::Back => *color = BackgroundColor(Color::srgb(0.5, 0.5, 0.5)),
            },
            Interaction::None => match button_type {
                PowerupHelpButton::Back => *color = BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
            },
        }
    }
}

/// Pause all active sound entities when game is paused
fn pause_sounds(sound_query: Query<&AudioSink, With<crate::SoundEntity>>) {
    for sink in &sound_query {
        sink.pause();
    }
    debug!("All sounds paused");
}

/// Resume all active sound entities when game is resumed
fn resume_sounds(sound_query: Query<&AudioSink, With<crate::SoundEntity>>) {
    for sink in &sound_query {
        sink.play();
    }
    debug!("All sounds resumed");
}

/// Updates the scroll position of scrollable nodes in response to mouse input
fn update_scroll_position(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    hover_map: Res<HoverMap>,
    mut scrolled_node_query: Query<&mut ScrollPosition>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    pause_menu_state: Res<State<PauseMenuState>>,
) {
    // Only process scroll events when in level selection
    if *pause_menu_state.get() != PauseMenuState::LevelSelection {
        return;
    }

    for mouse_wheel_event in mouse_wheel_events.read() {
        // Adjust scroll sensitivity for better mobile experience
        let (mut dx, mut dy) = match mouse_wheel_event.unit {
            MouseScrollUnit::Line => (mouse_wheel_event.x * 30.0, mouse_wheel_event.y * 30.0),
            MouseScrollUnit::Pixel => (mouse_wheel_event.x * 1.5, mouse_wheel_event.y * 1.5),
        };

        if keyboard_input.pressed(KeyCode::ControlLeft) || keyboard_input.pressed(KeyCode::ControlRight) {
            std::mem::swap(&mut dx, &mut dy);
        }

        // Apply scroll to all scroll containers when any UI element is hovered in level selection
        let mut scroll_applied = false;
        for (_pointer, pointer_map) in hover_map.iter() {
            if !pointer_map.is_empty() {
                // If anything is being hovered, apply scroll to all scroll containers
                for mut scroll_position in &mut scrolled_node_query {
                    scroll_position.offset_x -= dx;
                    scroll_position.offset_y -= dy;
                    scroll_applied = true;
                }
                break;
            }
        }

        // If nothing is being hovered but we're in level selection, still allow scrolling
        if !scroll_applied {
            for mut scroll_position in &mut scrolled_node_query {
                scroll_position.offset_x -= dx;
                scroll_position.offset_y -= dy;
            }
        }
    }
}

/// Resource to track touch scroll state
#[derive(Resource, Default)]
struct TouchScrollState {
    last_touch_position: Option<Vec2>,
    is_scrolling: bool,
}

/// Handle touch scrolling input
fn handle_touch_scroll(
    mut touch_input: EventReader<TouchInput>,
    mut scroll_query: Query<&mut ScrollPosition>,
    mut touch_scroll_state: ResMut<TouchScrollState>,
    pause_menu_state: Res<State<PauseMenuState>>,
) {
    // Only process touch input in level selection
    if *pause_menu_state.get() != PauseMenuState::LevelSelection {
        return;
    }

    for event in touch_input.read() {
        match event.phase {
            TouchPhase::Started => {
                // Initialize tracking on touch start
                touch_scroll_state.last_touch_position = Some(event.position);
                touch_scroll_state.is_scrolling = false; // Don't scroll until we get movement
            }
            TouchPhase::Moved => {
                let position = event.position;

                // Update scroll position based on touch movement
                if let Some(last_position) = touch_scroll_state.last_touch_position {
                    let delta = position - last_position;

                    // Only scroll if there's meaningful movement (helps prevent accidental scrolls)
                    if delta.length() > 2.0 {
                        for mut scroll_position in &mut scroll_query {
                            // Use vertical delta for vertical scrolling with enhanced sensitivity for mobile
                            scroll_position.offset_y -= delta.y * 1.5;
                        }
                        touch_scroll_state.is_scrolling = true;
                    }
                }

                // Update last touch position
                touch_scroll_state.last_touch_position = Some(position);
            }
            TouchPhase::Ended | TouchPhase::Canceled => {
                // Reset scrolling state on touch end
                touch_scroll_state.is_scrolling = false;
                touch_scroll_state.last_touch_position = None;
            }
        }
    }
}
