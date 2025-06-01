use bevy::prelude::*;

mod enemies;
mod menu;
mod pause_menu;
mod playing;
mod powerups;
use enemies::EnemiesPlugin;
use menu::MenuPlugin;
use pause_menu::PauseMenuPlugin;
use playing::PlayingPlugin;
use powerups::PowerupsPlugin;

/// Game states for managing different screens
#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
enum GameState {
    Menu,
    Playing,
}

impl Default for GameState {
    fn default() -> Self {
        Self::Menu
    }
}

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Kill All Dandelions".into(),
                resizable: true,
                fit_canvas_to_parent: true,
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }).set(bevy::log::LogPlugin {
            level: bevy::log::Level::INFO,
            filter: "wgpu=warn,naga=warn".to_string(),
            ..default()
        }))
        .init_state::<GameState>()
        .add_plugins((MenuPlugin, PauseMenuPlugin, PlayingPlugin, EnemiesPlugin, PowerupsPlugin))
        .run()
}
