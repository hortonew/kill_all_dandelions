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

/// Marker component for sound entities that need cleanup
#[derive(Component)]
pub struct SoundEntity;

fn main() -> AppExit {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Kill All Dandelions".into(),
                        resizable: true,
                        fit_canvas_to_parent: true,
                        prevent_default_event_handling: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(bevy::log::LogPlugin {
                    level: bevy::log::Level::INFO,
                    filter: "wgpu=warn,naga=warn".to_string(),
                    ..default()
                }),
        )
        .init_state::<GameState>()
        .add_systems(Startup, preload_assets)
        .add_systems(OnExit(GameState::Playing), cleanup_sounds)
        .add_plugins((MenuPlugin, PauseMenuPlugin, PlayingPlugin, EnemiesPlugin, PowerupsPlugin))
        .run()
}

/// Resource holding all preloaded asset handles
#[derive(Resource, Clone)]
pub struct GameAssets {
    pub bunny: Handle<Image>,
    pub flamethrower: Handle<Image>,
    pub dandelion_tiny: Handle<Image>,
    pub dandelion_small: Handle<Image>,
    pub dandelion_medium: Handle<Image>,
    pub dandelion_large: Handle<Image>,
    pub dandelion_huge: Handle<Image>,
    pub dandelion: Handle<Image>,
    pub seed: Handle<Image>,
    pub slash_sound: Handle<AudioSource>,
    pub rabbit_sound: Handle<AudioSource>,
}

fn preload_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let assets = GameAssets {
        bunny: asset_server.load("bunny.png"),
        flamethrower: asset_server.load("flamethrower.png"),
        dandelion_tiny: asset_server.load("dandelion_tiny.png"),
        dandelion_small: asset_server.load("dandelion_small.png"),
        dandelion_medium: asset_server.load("dandelion_medium.png"),
        dandelion_large: asset_server.load("dandelion_large.png"),
        dandelion_huge: asset_server.load("dandelion_huge.png"),
        dandelion: asset_server.load("dandelion.png"),
        seed: asset_server.load("seed.png"),
        slash_sound: asset_server.load("audio/slash.wav"),
        rabbit_sound: asset_server.load("audio/rabbit.wav"),
    };
    commands.insert_resource(assets);
}

/// Cleanup sound entities when exiting playing state
fn cleanup_sounds(mut commands: Commands, sound_entities: Query<Entity, With<SoundEntity>>) {
    for entity in &sound_entities {
        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.despawn();
        }
    }
    debug!("Sound entities cleaned up");
}
