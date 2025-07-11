use bevy::prelude::*;
use rand::Rng;

use crate::GameState;
use crate::levels::{LevelData, LevelStartEvent};

/// Plugin for handling the main menu screen
pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<MenuState>()
            .add_systems(OnEnter(GameState::Menu), (setup_menu_camera, setup_menu_ui, reset_menu_state))
            .add_systems(OnEnter(MenuState::Credits), setup_credits_menu)
            .add_systems(OnExit(MenuState::Credits), cleanup_credits_menu)
            .add_systems(Update, handle_menu_input.run_if(in_state(GameState::Menu)))
            .add_systems(Update, update_dynamic_font_sizes.run_if(in_state(GameState::Menu)))
            .add_systems(OnExit(GameState::Menu), cleanup_menu);
    }
}

/// Menu state for different screens
#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum MenuState {
    Main,
    Credits,
}

impl Default for MenuState {
    fn default() -> Self {
        Self::Main
    }
}

/// Marker component for menu entities
#[derive(Component)]
struct MenuEntity;

/// Menu button types
#[derive(Component)]
enum MenuButton {
    Play,
    Credits,
}

/// Marker component for dynamic font scaling
#[derive(Component)]
struct DynamicFontSize {
    base_size: f32,
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

/// Setup the menu camera
fn setup_menu_camera(mut commands: Commands) {
    commands.spawn((Camera2d, MenuEntity));
}

/// Setup the main menu UI
fn setup_menu_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Main menu container
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Vw(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
            MenuEntity,
        ))
        .with_children(|parent| {
            // Title with icon
            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    margin: UiRect::all(Val::Vh(2.0)),
                    column_gap: Val::Vw(2.0),
                    ..default()
                },))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Kill All Dandelions"),
                        TextFont { font_size: 36.0, ..default() },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                        DynamicFontSize { base_size: 36.0 },
                    ));
                });
            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    margin: UiRect::all(Val::Vh(2.0)),
                    column_gap: Val::Vw(2.0),
                    ..default()
                },))
                .with_children(|parent| {
                    parent.spawn((
                        ImageNode::new(asset_server.load("dandelion_small.png")),
                        Node {
                            width: Val::Px(75.0),
                            height: Val::Px(75.0),
                            ..default()
                        },
                    ));
                });
            // Subtitle
            parent.spawn((
                Text::new(get_random_subtitle()),
                TextFont { font_size: 16.0, ..default() },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                DynamicFontSize { base_size: 16.0 },
                Node {
                    margin: UiRect::all(Val::Vh(1.0)),
                    ..default()
                },
            ));

            // Play button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Vw(35.0),
                        height: Val::Vh(8.0),
                        margin: UiRect::all(Val::Vh(2.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.3, 0.7, 0.3)),
                    MenuButton::Play,
                    MenuEntity,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Start Game"),
                        TextFont { font_size: 22.0, ..default() },
                        TextColor(Color::WHITE),
                        DynamicFontSize { base_size: 22.0 },
                    ));
                });

            // Credits button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Vw(35.0),
                        height: Val::Vh(8.0),
                        margin: UiRect::all(Val::Vh(1.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.4, 0.4, 0.6)),
                    MenuButton::Credits,
                    MenuEntity,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Credits"),
                        TextFont { font_size: 22.0, ..default() },
                        TextColor(Color::WHITE),
                        DynamicFontSize { base_size: 22.0 },
                    ));
                });
        });
}

#[derive(Component)]
struct CreditsMenuEntity;

/// Setup credit screen
fn setup_credits_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
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
            CreditsMenuEntity,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Vw(80.0),
                        max_width: Val::Px(600.0),
                        height: Val::Vh(70.0),
                        max_height: Val::Px(500.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::FlexStart,
                        padding: UiRect::all(Val::Vh(2.5)),
                        row_gap: Val::Vh(2.5),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                    BorderRadius::all(Val::Px(10.0)),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Credits"),
                        TextFont { font_size: 28.0, ..default() },
                        TextColor(Color::WHITE),
                        DynamicFontSize { base_size: 28.0 },
                    ));

                    // Powerup table container
                    parent
                        .spawn((Node {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Vh(2.0),
                            ..default()
                        },))
                        .with_children(|parent| {
                            // Erik
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
                                    parent.spawn((
                                        ImageNode::new(asset_server.load("erik.png")),
                                        Node {
                                            width: Val::Vw(10.0),
                                            height: Val::Vw(10.0),
                                            max_width: Val::Px(60.0),
                                            max_height: Val::Px(60.0),
                                            ..default()
                                        },
                                    ));

                                    // Description text
                                    parent
                                        .spawn((Node {
                                            flex_direction: FlexDirection::Column,
                                            flex_grow: 1.0,
                                            row_gap: Val::Vh(0.5),
                                            ..default()
                                        },))
                                        .with_children(|parent| {
                                            parent.spawn((
                                                Text::new("Erik"),
                                                TextFont { font_size: 18.0, ..default() },
                                                TextColor(Color::srgb(0.9, 0.9, 0.5)),
                                                DynamicFontSize { base_size: 18.0 },
                                            ));

                                            parent.spawn((
                                                Text::new("Game developer, Sound designer (blog.erikhorton.com)"),
                                                TextFont { font_size: 14.0, ..default() },
                                                TextColor(Color::srgb(0.8, 0.8, 0.8)),
                                                DynamicFontSize { base_size: 14.0 },
                                            ));
                                        });
                                });
                            // Emi
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
                                    parent.spawn((
                                        ImageNode::new(asset_server.load("emi.png")),
                                        Node {
                                            width: Val::Vw(10.0),
                                            height: Val::Vw(10.0),
                                            max_width: Val::Px(60.0),
                                            max_height: Val::Px(60.0),
                                            ..default()
                                        },
                                    ));

                                    // Description text
                                    parent
                                        .spawn((Node {
                                            flex_direction: FlexDirection::Column,
                                            flex_grow: 1.0,
                                            row_gap: Val::Vh(0.5),
                                            ..default()
                                        },))
                                        .with_children(|parent| {
                                            parent.spawn((
                                                Text::new("Emi"),
                                                TextFont { font_size: 18.0, ..default() },
                                                TextColor(Color::srgb(0.9, 0.9, 0.5)),
                                                DynamicFontSize { base_size: 18.0 },
                                            ));

                                            parent.spawn((
                                                Text::new("Artist (www.emisketchbook.com)"),
                                                TextFont { font_size: 14.0, ..default() },
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
                            CreditsBackButton,
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

#[derive(Component)]
struct CreditsBackButton;

/// Reset menu state to main when entering menu
fn reset_menu_state(mut next_menu_state: ResMut<NextState<MenuState>>) {
    next_menu_state.set(MenuState::Main);
}

/// Handle menu input and button interactions
fn handle_menu_input(
    mut main_button_query: Query<(&Interaction, &mut BackgroundColor, &MenuButton), (Changed<Interaction>, With<Button>, Without<CreditsBackButton>)>,
    mut credits_button_query: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<CreditsBackButton>)>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut next_menu_state: ResMut<NextState<MenuState>>,
    current_menu_state: Res<State<MenuState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut level_data: ResMut<LevelData>,
    mut level_start_events: EventWriter<LevelStartEvent>,
) {
    match current_menu_state.get() {
        MenuState::Main => {
            // Handle main menu buttons only when in main menu state
            for (interaction, mut color, button_type) in &mut main_button_query {
                match *interaction {
                    Interaction::Pressed => match button_type {
                        MenuButton::Play => {
                            // Set the current level to level 1 and emit start event
                            level_data.set_current_level(1);
                            level_start_events.write(LevelStartEvent { level_id: 1 });
                            next_game_state.set(GameState::Playing);
                        }
                        MenuButton::Credits => next_menu_state.set(MenuState::Credits),
                    },
                    Interaction::Hovered => {
                        *color = match button_type {
                            MenuButton::Play => BackgroundColor(Color::srgb(0.4, 0.8, 0.4)),
                            MenuButton::Credits => BackgroundColor(Color::srgb(0.5, 0.5, 0.7)),
                        };
                    }
                    Interaction::None => {
                        *color = match button_type {
                            MenuButton::Play => BackgroundColor(Color::srgb(0.3, 0.7, 0.3)),
                            MenuButton::Credits => BackgroundColor(Color::srgb(0.4, 0.4, 0.6)),
                        };
                    }
                }
            }

            // Handle keyboard input only in main menu
            if keyboard_input.just_pressed(KeyCode::Space) || keyboard_input.just_pressed(KeyCode::Enter) {
                // Set the current level to level 1 and emit start event
                level_data.set_current_level(1);
                level_start_events.write(LevelStartEvent { level_id: 1 });
                next_game_state.set(GameState::Playing);
            }
        }
        MenuState::Credits => {
            // Handle credits back button only when in credits state
            for (interaction, mut color) in &mut credits_button_query {
                match *interaction {
                    Interaction::Pressed => {
                        next_menu_state.set(MenuState::Main);
                    }
                    Interaction::Hovered => {
                        *color = BackgroundColor(Color::srgb(0.4, 0.4, 0.4));
                    }
                    Interaction::None => {
                        *color = BackgroundColor(Color::srgb(0.3, 0.3, 0.3));
                    }
                }
            }
        }
    }
}

/// Cleanup menu entities when exiting menu state
fn cleanup_menu(mut commands: Commands, menu_entities: Query<Entity, With<MenuEntity>>) {
    for entity in &menu_entities {
        commands.entity(entity).despawn();
    }
}

/// Cleanup credits menu entities
fn cleanup_credits_menu(mut commands: Commands, credits_entities: Query<Entity, With<CreditsMenuEntity>>) {
    for entity in &credits_entities {
        commands.entity(entity).despawn();
    }
}

/// Get a random subtitle for the main menu
fn get_random_subtitle() -> &'static str {
    const SUBTITLES: &[&str] = &[
        "The \"Appease the HOA\" simulator!",
        "Dandelion demolition derby!",
        "Weeding out the competition!",
        "Flower power? More like flower DESTROY!",
    ];

    let mut rng = rand::thread_rng();
    SUBTITLES[rng.gen_range(0..SUBTITLES.len())]
}

/// Update dynamic font sizes based on window dimensions
fn update_dynamic_font_sizes(windows: Query<&Window>, mut text_query: Query<(&mut TextFont, &DynamicFontSize)>) {
    for (mut text_font, dynamic_size) in &mut text_query {
        text_font.font_size = calculate_font_size(dynamic_size.base_size, &windows);
    }
}
