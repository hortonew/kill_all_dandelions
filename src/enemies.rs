use bevy::prelude::*;
use rand::Rng;

use crate::GameState;
use crate::pause_menu::PauseState;
use crate::playing::GameData;

/// Plugin for handling enemy spawning and behavior
pub struct EnemiesPlugin;

impl Plugin for EnemiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), setup_enemy_timer)
            .add_systems(
                Update,
                (spawn_dandelions, handle_dandelion_clicks, update_seed_orbs, debug_dandelion_count)
                    .run_if(in_state(GameState::Playing))
                    .run_if(in_state(PauseState::Playing)),
            )
            .add_systems(OnExit(GameState::Playing), cleanup_enemies);
    }
}

/// Timer resource for spawning dandelions
#[derive(Resource)]
struct DandelionSpawnTimer {
    timer: Timer,
}

impl Default for DandelionSpawnTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(2.0, TimerMode::Repeating),
        }
    }
}

/// Component marking dandelion enemies
#[derive(Component)]
pub struct Dandelion {
    pub health: u32,
    pub spawn_count: u32,
}

/// Component for seed orbs that spawn new dandelions
#[derive(Component)]
struct SeedOrb {
    target_position: Vec2,
    spawn_timer: Timer,
    spawn_count: u32,
}

/// Marker component for enemy entities
#[derive(Component)]
struct EnemyEntity;

/// Setup the enemy spawn timer
fn setup_enemy_timer(mut commands: Commands) {
    commands.insert_resource(DandelionSpawnTimer::default());
}

/// Spawn dandelions at random positions
fn spawn_dandelions(
    mut commands: Commands,
    mut spawn_timer: ResMut<DandelionSpawnTimer>,
    time: Res<Time>,
    windows: Query<&Window>,
    asset_server: Res<AssetServer>,
    mut game_data: ResMut<GameData>,
) {
    spawn_timer.timer.tick(time.delta());

    if spawn_timer.timer.just_finished() {
        if let Ok(window) = windows.single() {
            let mut rng = rand::thread_rng();

            // Calculate safe spawn area to avoid UI panels at top and bottom
            let margin = 30.0; // Margin from edges
            let top_ui_height = window.height() * 0.12; // 12vh for top panel
            let bottom_ui_height = window.height() * 0.08; // 8vh for bottom panel

            // Calculate available grass area
            let min_x = -window.width() / 2.0 + margin;
            let max_x = window.width() / 2.0 - margin;
            let min_y = -window.height() / 2.0 + bottom_ui_height + margin;
            let max_y = window.height() / 2.0 - top_ui_height - margin;

            let x = rng.gen_range(min_x..max_x);
            let y = rng.gen_range(min_y..max_y);

            // Spawn dandelion sprite at proper size
            info!(
                "Spawning dandelion at ({:.1}, {:.1}) in grass area [{}x{} to {}x{}]",
                x, y, min_x as i32, min_y as i32, max_x as i32, max_y as i32
            );
            commands.spawn((
                Sprite {
                    image: asset_server.load("dandelion.png"),
                    color: Color::WHITE,
                    ..default()
                },
                Transform::from_translation(Vec3::new(x, y, 10.0)).with_scale(Vec3::splat(1.0)),
                Dandelion { health: 1, spawn_count: 2 },
                EnemyEntity,
            ));

            // Increment dandelion count
            game_data.dandelion_count += 1;
        }
    }
}

/// Handle clicks on dandelions
fn handle_dandelion_clicks(
    mut commands: Commands,
    mut dandelion_query: Query<(Entity, &mut Dandelion, &Transform)>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut game_data: ResMut<GameData>,
    asset_server: Res<AssetServer>,
) {
    if mouse_input.just_pressed(MouseButton::Left) {
        if let (Ok(window), Ok((camera, camera_transform))) = (windows.single(), camera_query.single()) {
            if let Some(cursor_pos) = window.cursor_position() {
                // Convert screen coordinates to world coordinates
                if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
                    info!("Click at world position: ({:.1}, {:.1})", world_pos.x, world_pos.y);

                    // Check if click hit any dandelion
                    for (entity, mut dandelion, transform) in dandelion_query.iter_mut() {
                        let dandelion_pos = transform.translation.truncate();
                        // Assume dandelion.png is about 35x35 pixels
                        let sprite_size = 35.0;
                        let half_size = sprite_size / 2.0;

                        // Simple AABB collision detection
                        if world_pos.x >= dandelion_pos.x - half_size
                            && world_pos.x <= dandelion_pos.x + half_size
                            && world_pos.y >= dandelion_pos.y - half_size
                            && world_pos.y <= dandelion_pos.y + half_size
                        {
                            dandelion.health = dandelion.health.saturating_sub(1);

                            if dandelion.health == 0 {
                                let spawn_count = dandelion.spawn_count;
                                spawn_seed_orbs(&mut commands, &asset_server, dandelion_pos, spawn_count);
                                commands.entity(entity).despawn();
                                game_data.add_dandelion_kill();
                                game_data.dandelion_count = game_data.dandelion_count.saturating_sub(1);
                                info!(
                                    "Dandelion destroyed at ({:.1}, {:.1})! Score: {}, Combo: {}x, Spawning {} seeds",
                                    dandelion_pos.x, dandelion_pos.y, game_data.score, game_data.combo, spawn_count
                                );
                            }

                            break; // Only hit one dandelion per click
                        }
                    }
                }
            }
        }
    }
}

/// Debug system to count dandelions
fn debug_dandelion_count(dandelions: Query<&Dandelion>) {
    let count = dandelions.iter().count();
    if count > 0 {
        info!("Current dandelion count: {}", count);
    }
}

/// Cleanup enemy entities when exiting playing state
fn cleanup_enemies(mut commands: Commands, enemy_entities: Query<Entity, With<EnemyEntity>>) {
    commands.remove_resource::<DandelionSpawnTimer>();

    for entity in &enemy_entities {
        commands.entity(entity).despawn();
    }

    info!("Enemies cleaned up");
}

/// Spawn seed orbs that will create new dandelions after a delay
fn spawn_seed_orbs(commands: &mut Commands, asset_server: &Res<AssetServer>, origin: Vec2, count: u32) {
    let mut rng = rand::thread_rng();

    for _ in 0..count {
        // Generate random direction and distance for seed travel
        let angle = rng.gen_range(0.0..std::f32::consts::TAU);
        let distance = rng.gen_range(50.0..150.0);
        let target_x = origin.x + angle.cos() * distance;
        let target_y = origin.y + angle.sin() * distance;

        commands.spawn((
            Sprite {
                image: asset_server.load("seed.png"),
                color: Color::WHITE,
                ..default()
            },
            Transform::from_translation(Vec3::new(origin.x, origin.y, 15.0)).with_scale(Vec3::splat(1.0)),
            SeedOrb {
                target_position: Vec2::new(target_x, target_y),
                spawn_timer: Timer::from_seconds(0.4, TimerMode::Once),
                spawn_count: 1,
            },
            EnemyEntity,
        ));
        info!(
            "Spawned seed orb at ({:.1}, {:.1}) targeting ({:.1}, {:.1})",
            origin.x, origin.y, target_x, target_y
        );
    }
}

/// Update seed orb movement and spawning
fn update_seed_orbs(
    mut commands: Commands,
    mut orb_query: Query<(Entity, &mut Transform, &mut SeedOrb)>,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut game_data: ResMut<GameData>,
) {
    for (entity, mut transform, mut orb) in orb_query.iter_mut() {
        orb.spawn_timer.tick(time.delta());

        // Move orb toward target position
        let current_pos = transform.translation.truncate();
        let direction = (orb.target_position - current_pos).normalize_or_zero();
        let move_speed = 150.0;
        let new_pos = current_pos + direction * move_speed * time.delta_secs();
        transform.translation = Vec3::new(new_pos.x, new_pos.y, 15.0);

        // Add some visual feedback
        if orb.spawn_timer.elapsed_secs() > 0.1 {
            info!(
                "Seed orb moving: ({:.1}, {:.1}) -> ({:.1}, {:.1})",
                current_pos.x, current_pos.y, new_pos.x, new_pos.y
            );
        }

        // Spawn new dandelion when timer finishes
        if orb.spawn_timer.finished() {
            commands.spawn((
                Sprite {
                    image: asset_server.load("dandelion.png"),
                    color: Color::WHITE,
                    ..default()
                },
                Transform::from_translation(Vec3::new(orb.target_position.x, orb.target_position.y, 10.0)).with_scale(Vec3::splat(1.0)),
                Dandelion { health: 1, spawn_count: 2 },
                EnemyEntity,
            ));

            // Remove the seed orb and update dandelion count
            commands.entity(entity).despawn();
            game_data.dandelion_count += 1;
            info!("Seed orb spawned new dandelion at ({:.1}, {:.1})", orb.target_position.x, orb.target_position.y);
        }
    }
}
