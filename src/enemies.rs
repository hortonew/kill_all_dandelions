use bevy::prelude::*;
use rand::Rng;
use std::collections::HashSet;

use crate::GameState;
use crate::pause_menu::PauseState;
use crate::playing::GameData;

/// Plugin for handling enemy spawning and behavior
pub struct EnemiesPlugin;

impl Plugin for EnemiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), (setup_enemy_timer, setup_area_tracker))
            .add_systems(
                Update,
                (
                    spawn_dandelions,
                    handle_dandelion_clicks,
                    update_seed_orbs,
                    check_dandelion_merging,
                    debug_dandelion_count,
                )
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

/// Resource to track total dandelion visual area for performance
#[derive(Resource, Default)]
pub struct DandelionAreaTracker {
    pub total_area: f32,
}

/// Component marking dandelion enemies
#[derive(Component, Clone)]
pub struct Dandelion {
    pub health: u32,
    pub size: DandelionSize,
}

/// Dandelion size variants
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DandelionSize {
    Tiny,
    Small,
    Medium,
    Large,
    Huge,
}

impl DandelionSize {
    /// Get the asset filename for this size
    pub fn asset_path(&self) -> &'static str {
        match self {
            DandelionSize::Tiny => "dandelion_tiny.png",
            DandelionSize::Small => "dandelion_small.png",
            DandelionSize::Medium => "dandelion_medium.png",
            DandelionSize::Large => "dandelion_large.png",
            DandelionSize::Huge => "dandelion_huge.png",
        }
    }

    /// Get spawn count when this dandelion dies
    pub fn spawn_count(&self) -> u32 {
        match self {
            DandelionSize::Tiny => 2,
            DandelionSize::Small => 3,
            DandelionSize::Medium => 4,
            DandelionSize::Large => 5,
            DandelionSize::Huge => 6,
        }
    }

    /// Get scale factor for rendering
    pub fn scale(&self) -> f32 {
        match self {
            DandelionSize::Tiny => 1.0,
            DandelionSize::Small => 1.3,
            DandelionSize::Medium => 1.6,
            DandelionSize::Large => 2.0,
            DandelionSize::Huge => 2.5,
        }
    }

    /// Get collision radius
    pub fn collision_radius(&self) -> f32 {
        match self {
            DandelionSize::Tiny => 17.5,
            DandelionSize::Small => 23.0,
            DandelionSize::Medium => 28.0,
            DandelionSize::Large => 35.0,
            DandelionSize::Huge => 44.0,
        }
    }

    /// Get merge radius (when dandelions should combine)
    pub fn merge_radius(&self) -> f32 {
        self.collision_radius() * 1.2
    }

    /// Get next size up for merging
    pub fn next_size(&self) -> Option<Self> {
        match self {
            DandelionSize::Tiny => Some(DandelionSize::Small),
            DandelionSize::Small => Some(DandelionSize::Medium),
            DandelionSize::Medium => Some(DandelionSize::Large),
            DandelionSize::Large => Some(DandelionSize::Huge),
            DandelionSize::Huge => None,
        }
    }

    /// Get visual area coverage for curb appeal calculation
    pub fn visual_area(&self) -> f32 {
        let radius = self.collision_radius();
        let base_area = std::f32::consts::PI * radius * radius;

        // Apply size-based multiplier for curb appeal impact
        let size_multiplier = match self {
            DandelionSize::Tiny => 1.0,
            DandelionSize::Small => 1.2,
            DandelionSize::Medium => 1.5,
            DandelionSize::Large => 2.0,
            DandelionSize::Huge => 3.0,
        };

        base_area * size_multiplier
    }
}

/// Component for seed orbs that spawn new dandelions
#[derive(Component)]
struct SeedOrb {
    target_position: Vec2,
    spawn_timer: Timer,
}

/// Marker component for enemy entities
#[derive(Component)]
struct EnemyEntity;

/// Setup the enemy spawn timer
fn setup_enemy_timer(mut commands: Commands) {
    commands.insert_resource(DandelionSpawnTimer::default());
}

/// Setup the dandelion area tracker
fn setup_area_tracker(mut commands: Commands) {
    commands.insert_resource(DandelionAreaTracker::default());
}

/// Spawn dandelions at random positions
fn spawn_dandelions(
    mut commands: Commands,
    mut spawn_timer: ResMut<DandelionSpawnTimer>,
    time: Res<Time>,
    windows: Query<&Window>,
    asset_server: Res<AssetServer>,
    mut game_data: ResMut<GameData>,
    mut area_tracker: ResMut<DandelionAreaTracker>,
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

            let size = DandelionSize::Tiny;
            commands.spawn((
                Sprite {
                    image: asset_server.load(size.asset_path()),
                    color: Color::WHITE,
                    ..default()
                },
                Transform::from_translation(Vec3::new(x, y, 10.0)).with_scale(Vec3::splat(size.scale())),
                Dandelion { health: 1, size },
                EnemyEntity,
            ));

            game_data.dandelion_count += 1;
            area_tracker.total_area += size.visual_area();
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
    mut area_tracker: ResMut<DandelionAreaTracker>,
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
                        let collision_radius = dandelion.size.collision_radius();

                        // Simple circular collision detection using the dandelion's size
                        let distance = world_pos.distance(dandelion_pos);
                        if distance <= collision_radius {
                            dandelion.health = dandelion.health.saturating_sub(1);

                            if dandelion.health == 0 {
                                let spawn_count = dandelion.size.spawn_count();
                                area_tracker.total_area -= dandelion.size.visual_area();
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

/// Debug system to count dandelions (runs less frequently)
fn debug_dandelion_count(dandelions: Query<&Dandelion>, time: Res<Time>) {
    // Only log every 2 seconds to reduce spam
    if (time.elapsed_secs() as u32) % 2 == 0 && time.delta_secs() < 0.1 {
        let count = dandelions.iter().count();
        if count > 0 {
            info!("Current dandelion count: {}", count);
        }
    }
}

/// Cleanup enemy entities when exiting playing state
fn cleanup_enemies(mut commands: Commands, enemy_entities: Query<Entity, With<EnemyEntity>>) {
    commands.remove_resource::<DandelionSpawnTimer>();
    commands.remove_resource::<DandelionAreaTracker>();

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
    mut area_tracker: ResMut<DandelionAreaTracker>,
) {
    for (entity, mut transform, mut orb) in orb_query.iter_mut() {
        orb.spawn_timer.tick(time.delta());

        // Move orb toward target position
        let current_pos = transform.translation.truncate();
        let direction = (orb.target_position - current_pos).normalize_or_zero();
        let move_speed = 150.0;
        let new_pos = current_pos + direction * move_speed * time.delta_secs();
        transform.translation = Vec3::new(new_pos.x, new_pos.y, 15.0);

        // Reduce verbose logging
        if orb.spawn_timer.just_finished() {
            info!(
                "Seed orb moving: ({:.1}, {:.1}) -> ({:.1}, {:.1})",
                current_pos.x, current_pos.y, new_pos.x, new_pos.y
            );
        }

        // Spawn new dandelion when timer finishes
        if orb.spawn_timer.finished() {
            let size = DandelionSize::Tiny;
            commands.spawn((
                Sprite {
                    image: asset_server.load(size.asset_path()),
                    color: Color::WHITE,
                    ..default()
                },
                Transform::from_translation(Vec3::new(orb.target_position.x, orb.target_position.y, 10.0)).with_scale(Vec3::splat(size.scale())),
                Dandelion { health: 1, size },
                EnemyEntity,
            ));

            // Remove the seed orb and update dandelion count
            commands.entity(entity).despawn();
            game_data.dandelion_count += 1;
            area_tracker.total_area += size.visual_area();
            info!("Seed orb spawned new dandelion at ({:.1}, {:.1})", orb.target_position.x, orb.target_position.y);
        }
    }
}

/// Check for dandelions that should merge together
fn check_dandelion_merging(
    mut commands: Commands,
    dandelion_query: Query<(Entity, &Dandelion, &Transform)>,
    asset_server: Res<AssetServer>,
    mut game_data: ResMut<GameData>,
    mut area_tracker: ResMut<DandelionAreaTracker>,
) {
    let mut to_merge: Vec<(Entity, Entity, Vec2, DandelionSize, DandelionSize)> = Vec::new();
    let mut entities_to_remove: HashSet<Entity> = HashSet::new();

    // Collect all dandelions for comparison
    let dandelions: Vec<(Entity, &Dandelion, &Transform)> = dandelion_query.iter().collect();

    // Check all pairs for merging opportunities
    for i in 0..dandelions.len() {
        for j in (i + 1)..dandelions.len() {
            let (entity1, dandelion1, transform1) = dandelions[i];
            let (entity2, dandelion2, transform2) = dandelions[j];

            // Skip if either entity is already marked for removal
            if entities_to_remove.contains(&entity1) || entities_to_remove.contains(&entity2) {
                continue;
            }

            // Only merge dandelions of the same size
            if dandelion1.size != dandelion2.size {
                continue;
            }

            let pos1 = transform1.translation.truncate();
            let pos2 = transform2.translation.truncate();
            let distance = pos1.distance(pos2);
            let merge_radius = dandelion1.size.merge_radius();

            if distance <= merge_radius {
                // Check if we can create a larger dandelion
                if let Some(new_size) = dandelion1.size.next_size() {
                    // Calculate merge position (midpoint)
                    let merge_pos = (pos1 + pos2) / 2.0;

                    to_merge.push((entity1, entity2, merge_pos, new_size, dandelion1.size));
                    entities_to_remove.insert(entity1);
                    entities_to_remove.insert(entity2);

                    info!(
                        "Merging two {:?} dandelions at ({:.1}, {:.1}) and ({:.1}, {:.1}) into {:?} at ({:.1}, {:.1})",
                        dandelion1.size, pos1.x, pos1.y, pos2.x, pos2.y, new_size, merge_pos.x, merge_pos.y
                    );
                } else {
                    info!("Two {:?} dandelions are close but cannot merge further (already at max size)", dandelion1.size);
                }
            }
        }
    }

    // Execute all merges
    for (entity1, entity2, merge_pos, new_size, old_size) in to_merge {
        // Update area tracker: remove two old dandelions, add one new one
        area_tracker.total_area -= old_size.visual_area() * 2.0;
        area_tracker.total_area += new_size.visual_area();

        // Remove the two original dandelions
        commands.entity(entity1).despawn();
        commands.entity(entity2).despawn();

        // Create new merged dandelion
        commands.spawn((
            Sprite {
                image: asset_server.load(new_size.asset_path()),
                color: Color::WHITE,
                ..default()
            },
            Transform::from_translation(Vec3::new(merge_pos.x, merge_pos.y, 10.0)).with_scale(Vec3::splat(new_size.scale())),
            Dandelion { health: 1, size: new_size },
            EnemyEntity,
        ));

        // Update count (2 removed, 1 added = net -1)
        game_data.dandelion_count = game_data.dandelion_count.saturating_sub(1);
    }
}
