use bevy::prelude::*;

use crate::GameState;
use crate::levels::{LevelData, LevelStartEvent};

/// Plugin for handling the pause menu
pub struct PauseMenuPlugin;

impl Plugin for PauseMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<PauseState>()
            .init_state::<PauseMenuState>()
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
            .add_systems(Update, switch_pause_menu_content.run_if(in_state(PauseState::Paused)));
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
                    parent.spawn((Text::new("Game Paused"), TextFont { font_size: 28.0, ..default() }, TextColor(Color::WHITE)));

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
                            parent.spawn((Text::new("Resume Game"), TextFont { font_size: 18.0, ..default() }, TextColor(Color::WHITE)));
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
                            parent.spawn((Text::new("Restart Game"), TextFont { font_size: 18.0, ..default() }, TextColor(Color::WHITE)));
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
                            parent.spawn((Text::new("Powerup Help"), TextFont { font_size: 18.0, ..default() }, TextColor(Color::WHITE)));
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
                            parent.spawn((Text::new("Level Selection"), TextFont { font_size: 18.0, ..default() }, TextColor(Color::WHITE)));
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
                padding: UiRect::all(Val::Vw(2.0)),
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
                        height: Val::Vh(80.0),
                        max_height: Val::Px(600.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::FlexStart,
                        padding: UiRect::all(Val::Vh(2.0)),
                        row_gap: Val::Vh(2.0),
                        overflow: Overflow::scroll_y(),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                    BorderRadius::all(Val::Px(10.0)),
                ))
                .with_children(|parent| {
                    parent.spawn((Text::new("Powerup Help"), TextFont { font_size: 24.0, ..default() }, TextColor(Color::WHITE)));

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
                                            ));

                                            parent.spawn((
                                                Text::new("Spawns 3 rabbits that seek and destroy dandelions. Each rabbit has 3 seconds to eat a dandelion, and eating a least 2 spawns a new rabbit."),
                                                TextFont {
                                                    font_size: 14.0,
                                                    ..default()
                                                },
                                                TextColor(Color::srgb(0.8, 0.8, 0.8)),
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
                                            ));

                                            parent.spawn((
                                                Text::new("Creates a fire ignition that continuously damages all dandelions within its radius for 2 seconds. Effective against groups of dandelions."),
                                                TextFont {
                                                    font_size: 14.0,
                                                    ..default()
                                                },
                                                TextColor(Color::srgb(0.8, 0.8, 0.8)),
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
                            parent.spawn((Text::new("Back"), TextFont { font_size: 20.0, ..default() }, TextColor(Color::WHITE)));
                        });
                });
        });
}

/// Setup level selection menu UI
fn setup_level_selection_menu(mut commands: Commands, _asset_server: Res<AssetServer>) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect::all(Val::Vw(2.0)),
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
                        max_width: Val::Px(800.0),
                        height: Val::Vh(85.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::FlexStart,
                        padding: UiRect::all(Val::Vh(2.0)),
                        row_gap: Val::Vh(2.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                    BorderRadius::all(Val::Px(10.0)),
                ))
                .with_children(|parent| {
                    parent.spawn((Text::new("Level Selection"), TextFont { font_size: 28.0, ..default() }, TextColor(Color::WHITE)));

                    // Level grid container - scrollable
                    parent
                        .spawn((Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(70.0),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Vh(2.0),
                            overflow: Overflow::scroll_y(),
                            padding: UiRect::all(Val::Px(10.0)),
                            ..default()
                        },))
                        .with_children(|parent| {
                            // Create level cards in rows of 2
                            for row in 0..5 {
                                parent
                                    .spawn((Node {
                                        width: Val::Percent(100.0),
                                        flex_direction: FlexDirection::Row,
                                        justify_content: JustifyContent::SpaceEvenly,
                                        column_gap: Val::Vw(3.0),
                                        ..default()
                                    },))
                                    .with_children(|parent| {
                                        for col in 0..2 {
                                            let level_id = row * 2 + col + 1;
                                            if level_id <= 10 {
                                                // Create level card inline
                                                parent
                                                    .spawn((
                                                        Button,
                                                        Node {
                                                            width: Val::Percent(45.0),
                                                            // height: Val::Vh(15.0),
                                                            flex_direction: FlexDirection::Column,
                                                            align_items: AlignItems::Center,
                                                            justify_content: JustifyContent::Center,
                                                            padding: UiRect::all(Val::Vh(1.0)),
                                                            ..default()
                                                        },
                                                        BackgroundColor(if level_id == 1 {
                                                            Color::srgb(0.4, 0.4, 0.6) // Level 1 is always unlocked
                                                        } else {
                                                            Color::srgb(0.3, 0.3, 0.3) // Default locked color
                                                        }),
                                                        BorderRadius::all(Val::Px(8.0)),
                                                        LevelSelectionButton::LevelButton(level_id),
                                                        PauseMenuEntity,
                                                    ))
                                                    .with_children(|parent| {
                                                        // Level number
                                                        parent.spawn((
                                                            Text::new(format!("Level {}", level_id)),
                                                            TextFont { font_size: 20.0, ..default() },
                                                            TextColor(Color::WHITE),
                                                        ));

                                                        // Stars display
                                                        parent
                                                            .spawn((Node {
                                                                flex_direction: FlexDirection::Row,
                                                                column_gap: Val::Px(5.0),
                                                                ..default()
                                                            },))
                                                            .with_children(|parent| {
                                                                for star_index in 0..3 {
                                                                    parent.spawn((
                                                                        Text::new("*"), // Simple asterisk - will be updated based on progress
                                                                        TextFont { font_size: 16.0, ..default() },
                                                                        TextColor(Color::srgb(0.8, 0.8, 0.2)),
                                                                        StarDisplay { level_id, star_index },
                                                                    ));
                                                                }
                                                            });
                                                    });
                                            }
                                        }
                                    });
                            }
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
                            LevelSelectionButton::Back,
                            PauseMenuEntity,
                        ))
                        .with_children(|parent| {
                            parent.spawn((Text::new("Back"), TextFont { font_size: 20.0, ..default() }, TextColor(Color::WHITE)));
                        });
                });
        });
}

/// Update star displays based on level progress
fn update_star_displays(mut star_query: Query<(&mut Text, &mut TextColor, &StarDisplay)>, level_data: Res<LevelData>) {
    for (mut text, mut text_color, star_display) in &mut star_query {
        if let Some(progress) = level_data.level_progress.get((star_display.level_id - 1) as usize) {
            let filled_stars = progress.best_stars;

            // Update star text based on index and progress
            let (star_text, star_color) = if star_display.star_index < filled_stars {
                ("*", Color::srgb(1.0, 0.8, 0.0)) // Gold asterisk for filled
            } else {
                ("o", Color::srgb(0.4, 0.4, 0.4)) // Gray circle for empty
            };

            text.0 = star_text.to_string();
            *text_color = TextColor(star_color);
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
fn setup_pause_menu_on_pause(pause_menu_state: Res<State<PauseMenuState>>, commands: Commands, asset_server: Res<AssetServer>) {
    match pause_menu_state.get() {
        PauseMenuState::PauseMenu => setup_pause_menu(commands),
        PauseMenuState::PowerupHelp => setup_powerup_help_menu(commands, asset_server),
        PauseMenuState::LevelSelection => setup_level_selection_menu(commands, asset_server),
    }
}

/// Switch pause menu content based on pause menu state changes
fn switch_pause_menu_content(
    pause_menu_state: Res<State<PauseMenuState>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
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
                PauseMenuState::LevelSelection => setup_level_selection_menu(commands, asset_server),
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
