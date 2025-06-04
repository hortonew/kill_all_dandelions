use bevy::prelude::*;

mod enemies;
mod levels;
mod menu;
mod pause_menu;
mod playing;
mod powerups;
use enemies::EnemiesPlugin;
use levels::LevelsPlugin;
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

// Resource for entity diagnostic logging timer
// #[derive(Resource)]
// struct EntityDiagnosticTimer {
//     timer: Timer,
// }

// impl Default for EntityDiagnosticTimer {
//     fn default() -> Self {
//         Self {
//             timer: Timer::from_seconds(1.0, TimerMode::Repeating),
//         }
//     }
// }

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
        //.init_resource::<EntityDiagnosticTimer>()
        .add_systems(Startup, preload_assets)
        .add_systems(OnExit(GameState::Playing), cleanup_sounds)
        // .add_systems(Update, log_entity_counts.run_if(in_state(GameState::Playing)))
        .add_plugins((MenuPlugin, PauseMenuPlugin, PlayingPlugin, EnemiesPlugin, PowerupsPlugin, LevelsPlugin))
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
    pub flamethrower_sound: Handle<AudioSource>,
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
        flamethrower_sound: asset_server.load("audio/slash.wav"),
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

// Log entity counts every second for performance monitoring
// fn log_entity_counts(
//     mut timer_res: ResMut<EntityDiagnosticTimer>,
//     time: Res<Time>,
//     dandelions: Query<Entity, With<enemies::Dandelion>>,
//     rabbits: Query<Entity, With<powerups::Rabbit>>,
//     fire_ignitions: Query<Entity, With<powerups::FireIgnition>>,
//     sound_entities: Query<Entity, With<SoundEntity>>,
//     rabbit_sound_timers: Query<Entity, With<powerups::RabbitSoundTimer>>,
//     enemy_entities: Query<Entity, With<enemies::EnemyEntity>>,
//     powerup_entities: Query<Entity, With<powerups::PowerupEntity>>,
//     health_bars: Query<Entity, With<enemies::HealthBar>>,
//     slash_effects: Query<Entity, With<playing::SlashEffect>>,
//     seed_orbs: Query<Entity, With<enemies::SeedOrb>>,
//     merge_effects: Query<Entity, With<enemies::MergeEffect>>,
//     moving_dandelions: Query<Entity, With<enemies::MovingDandelion>>,
//     powerup_effects: Query<Entity, With<powerups::PowerupEffect>>,
// ) {
//     timer_res.timer.tick(time.delta());

//     if timer_res.timer.just_finished() {
//         let dandelion_count = dandelions.iter().count();
//         let rabbit_count = rabbits.iter().count();
//         let fire_count = fire_ignitions.iter().count();
//         let sound_count = sound_entities.iter().count();
//         let rabbit_sound_count = rabbit_sound_timers.iter().count();
//         let enemy_count = enemy_entities.iter().count();
//         let powerup_count = powerup_entities.iter().count();
//         let health_bar_count = health_bars.iter().count();
//         let slash_effect_count = slash_effects.iter().count();
//         let seed_orb_count = seed_orbs.iter().count();
//         let merge_effect_count = merge_effects.iter().count();
//         let moving_dandelion_count = moving_dandelions.iter().count();
//         let powerup_effect_count = powerup_effects.iter().count();

//         debug!(
//             "Entity counts - Dandelions: {}, Rabbits: {}, Fires: {}, Sounds: {}, RabbitSounds: {}, Enemies: {}, Powerups: {}, HealthBars: {}, SlashEffects: {}, SeedOrbs: {}, MergeEffects: {}, MovingDandelions: {}, PowerupEffects: {}",
//             dandelion_count,
//             rabbit_count,
//             fire_count,
//             sound_count,
//             rabbit_sound_count,
//             enemy_count,
//             powerup_count,
//             health_bar_count,
//             slash_effect_count,
//             seed_orb_count,
//             merge_effect_count,
//             moving_dandelion_count,
//             powerup_effect_count
//         );
//     }
// }
