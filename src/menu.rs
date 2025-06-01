use bevy::prelude::*;

use crate::GameState;

/// Plugin for handling the main menu screen
pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Menu), (setup_menu_camera, setup_menu_ui))
            .add_systems(Update, handle_menu_input.run_if(in_state(GameState::Menu)))
            .add_systems(OnExit(GameState::Menu), cleanup_menu);
    }
}

/// Marker component for menu entities
#[derive(Component)]
struct MenuEntity;

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
                    margin: UiRect::all(Val::Px(20.0)),
                    column_gap: Val::Px(15.0),
                    ..default()
                },))
                .with_children(|parent| {
                    parent.spawn((
                        ImageNode::new(asset_server.load("dandelion_tiny.png")),
                        Node {
                            width: Val::Px(48.0),
                            height: Val::Px(48.0),
                            ..default()
                        },
                    ));
                    parent.spawn((
                        Text::new("Kill All Dandelions"),
                        TextFont { font_size: 48.0, ..default() },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));
                });

            // Subtitle
            parent.spawn((
                Text::new("Tap fast, combo hard, maintain the perfect lawn!"),
                TextFont { font_size: 18.0, ..default() },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                Node {
                    margin: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
            ));

            // Play button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(60.0),
                        margin: UiRect::all(Val::Px(20.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.3, 0.7, 0.3)),
                    MenuEntity,
                ))
                .with_children(|parent| {
                    parent.spawn((Text::new("Start Game"), TextFont { font_size: 24.0, ..default() }, TextColor(Color::WHITE)));
                });
        });
}

/// Handle menu input and button interactions
fn handle_menu_input(
    mut interaction_query: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<Button>)>,
    mut next_state: ResMut<NextState<GameState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    // Handle button interactions
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                next_state.set(GameState::Playing);
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::srgb(0.4, 0.8, 0.4));
            }
            Interaction::None => {
                *color = BackgroundColor(Color::srgb(0.3, 0.7, 0.3));
            }
        }
    }

    // Handle keyboard input
    if keyboard_input.just_pressed(KeyCode::Space) || keyboard_input.just_pressed(KeyCode::Enter) {
        next_state.set(GameState::Playing);
    }
}

/// Cleanup menu entities when exiting menu state
fn cleanup_menu(mut commands: Commands, menu_entities: Query<Entity, With<MenuEntity>>) {
    for entity in &menu_entities {
        commands.entity(entity).despawn();
    }
}
