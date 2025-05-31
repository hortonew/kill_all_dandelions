use bevy::prelude::*;

mod menu;
mod playing;

use menu::MenuPlugin;
use playing::PlayingPlugin;

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
                // For web builds, let the canvas fill the browser window
                fit_canvas_to_parent: true,
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        .init_state::<GameState>()
        .add_plugins((MenuPlugin, PlayingPlugin))
        .run()
}
