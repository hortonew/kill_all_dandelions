use bevy::prelude::*;

use crate::GameState;

/// Plugin for handling the pause menu
pub struct PauseMenuPlugin;

impl Plugin for PauseMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<PauseState>()
            .init_state::<PauseMenuState>()
            .add_systems(OnEnter(PauseState::Paused), setup_pause_menu_on_pause)
            .add_systems(OnExit(PauseState::Paused), cleanup_pause_menu)
            .add_systems(
                Update,
                (handle_pause_input, pause_menu_interactions).run_if(in_state(PauseState::Paused).and(in_state(PauseMenuState::PauseMenu))),
            )
            .add_systems(
                Update,
                (handle_powerup_help_input, powerup_help_interactions).run_if(in_state(PauseState::Paused).and(in_state(PauseMenuState::PowerupHelp))),
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
}

/// Powerup help menu button types
#[derive(Component)]
enum PowerupHelpButton {
    Back,
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
                    parent.spawn((Text::new("Game Paused"), TextFont { font_size: 32.0, ..default() }, TextColor(Color::WHITE)));

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
                            BackgroundColor(Color::srgb(0.3, 0.5, 0.7)),
                            BorderRadius::all(Val::Px(5.0)),
                            PauseMenuButton::PowerupHelp,
                            PauseMenuEntity,
                        ))
                        .with_children(|parent| {
                            parent.spawn((Text::new("Powerup Help"), TextFont { font_size: 20.0, ..default() }, TextColor(Color::WHITE)));
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
            },
            Interaction::Hovered => match button_type {
                PauseMenuButton::Resume => *color = BackgroundColor(Color::srgb(0.5, 0.5, 0.5)),
                PauseMenuButton::Restart => *color = BackgroundColor(Color::srgb(0.8, 0.4, 0.4)),
                PauseMenuButton::PowerupHelp => *color = BackgroundColor(Color::srgb(0.4, 0.6, 0.8)),
            },
            Interaction::None => match button_type {
                PauseMenuButton::Resume => *color = BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                PauseMenuButton::Restart => *color = BackgroundColor(Color::srgb(0.6, 0.3, 0.3)),
                PauseMenuButton::PowerupHelp => *color = BackgroundColor(Color::srgb(0.3, 0.5, 0.7)),
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
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            PauseMenuEntity,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(600.0),
                        height: Val::Px(500.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::FlexStart,
                        padding: UiRect::all(Val::Px(20.0)),
                        row_gap: Val::Px(20.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                    BorderRadius::all(Val::Px(10.0)),
                ))
                .with_children(|parent| {
                    parent.spawn((Text::new("Powerup Help"), TextFont { font_size: 28.0, ..default() }, TextColor(Color::WHITE)));

                    // Powerup table container
                    parent
                        .spawn((Node {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(15.0),
                            ..default()
                        },))                        .with_children(|parent| {
                            // Bunny powerup row
                            parent
                                .spawn((
                                    Node {
                                        width: Val::Percent(100.0),
                                        height: Val::Px(80.0),
                                        flex_direction: FlexDirection::Row,
                                        align_items: AlignItems::Center,
                                        column_gap: Val::Px(20.0),
                                        padding: UiRect::all(Val::Px(10.0)),
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
                                            width: Val::Px(60.0),
                                            height: Val::Px(60.0),
                                            ..default()
                                        },
                                    ));

                                    // Description text
                                    parent
                                        .spawn((
                                            Node {
                                                flex_direction: FlexDirection::Column,
                                                flex_grow: 1.0,
                                                row_gap: Val::Px(5.0),
                                                ..default()
                                            },
                                        ))
                                        .with_children(|parent| {
                                            parent.spawn((
                                                Text::new("Bunny"),
                                                TextFont {
                                                    font_size: 18.0,
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
                                        height: Val::Px(80.0),
                                        flex_direction: FlexDirection::Row,
                                        align_items: AlignItems::Center,
                                        column_gap: Val::Px(20.0),
                                        padding: UiRect::all(Val::Px(10.0)),
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
                                            width: Val::Px(60.0),
                                            height: Val::Px(60.0),
                                            ..default()
                                        },
                                    ));

                                    // Description text
                                    parent
                                        .spawn((
                                            Node {
                                                flex_direction: FlexDirection::Column,
                                                flex_grow: 1.0,
                                                row_gap: Val::Px(5.0),
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
                                width: Val::Px(200.0),
                                height: Val::Px(50.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                margin: UiRect::top(Val::Px(20.0)),
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

/// Setup pause menu when entering paused state
fn setup_pause_menu_on_pause(pause_menu_state: Res<State<PauseMenuState>>, commands: Commands, asset_server: Res<AssetServer>) {
    match pause_menu_state.get() {
        PauseMenuState::PauseMenu => setup_pause_menu(commands),
        PauseMenuState::PowerupHelp => setup_powerup_help_menu(commands, asset_server),
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
