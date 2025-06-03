use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use rand::Rng;
use std::collections::HashSet;

use crate::GameAssets;
use crate::GameState;
use crate::pause_menu::PauseState;
use crate::playing::GameData;

/// Plugin for handling enemy spawning and behavior
pub struct EnemiesPlugin;

impl Plugin for EnemiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), (setup_enemy_timer, setup_area_tracker, setup_variety_spawner))
            .add_systems(
                Update,
                (
                    spawn_dandelions,
                    spawn_variety_dandelions,
                    handle_dandelion_clicks,
                    update_seed_orbs,
                    check_dandelion_merging,
                    update_merge_effects,
                    update_moving_dandelions,
                    check_moving_dandelion_collisions,
                    update_upgrade_cooldowns,
                    manage_health_bars,
                    update_health_bar_positions,
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

/// Timer resource for spawning variety of dandelions
#[derive(Resource)]
struct VarietySpawnTimer {
    timer: Timer,
    enabled: bool,
    difficulty_threshold: u32,
}

impl Default for VarietySpawnTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(10.0, TimerMode::Repeating),
            enabled: false,
            difficulty_threshold: 500, // Enable when score reaches 500
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

    /// Get scale factor for rendering with relative sizing
    pub fn scale(&self) -> f32 {
        // Scale 175x175 images with relative sizing (tiny = 35x35 base)
        match self {
            DandelionSize::Tiny => 0.2,    // 35x35
            DandelionSize::Small => 0.26,  // 45.5x45.5
            DandelionSize::Medium => 0.32, // 56x56
            DandelionSize::Large => 0.4,   // 70x70
            DandelionSize::Huge => 0.5,    // 87.5x87.5
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

    /// Get base health before level scaling
    pub fn base_health(&self) -> u32 {
        match self {
            DandelionSize::Tiny => 1,
            DandelionSize::Small => 2,
            DandelionSize::Medium => 3,
            DandelionSize::Large => 4,
            DandelionSize::Huge => 5,
        }
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
pub struct EnemyEntity;

/// Component for merge effect
#[derive(Component)]
struct MergeEffect {
    timer: Timer,
    initial_scale: f32,
}

/// Component for moving dandelions (huge size only)
#[derive(Component)]
struct MovingDandelion {
    velocity: Vec2,
    speed: f32,
    direction_change_timer: Timer,
}

impl Default for MovingDandelion {
    fn default() -> Self {
        let mut rng = rand::thread_rng();
        let angle = rng.gen_range(0.0..std::f32::consts::TAU);
        let speed = 50.0;
        Self {
            velocity: Vec2::new(angle.cos(), angle.sin()) * speed,
            speed,
            direction_change_timer: Timer::from_seconds(2.0, TimerMode::Repeating),
        }
    }
}

/// Component to prevent rapid successive upgrades
#[derive(Component)]
struct UpgradeCooldown {
    timer: Timer,
}

impl Default for UpgradeCooldown {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(1.0, TimerMode::Once),
        }
    }
}

/// Component for health bars attached to damaged dandelions
#[derive(Component)]
struct HealthBar {
    dandelion_entity: Entity,
    max_health: u32,
}

/// Marker component for health bar background
#[derive(Component)]
struct HealthBarBackground;

/// Marker component for health bar fill
#[derive(Component)]
struct HealthBarFill;

/// Setup the enemy spawn timer
fn setup_enemy_timer(mut commands: Commands) {
    commands.insert_resource(DandelionSpawnTimer::default());
}

/// Setup the dandelion area tracker
fn setup_area_tracker(mut commands: Commands) {
    commands.insert_resource(DandelionAreaTracker::default());
}

/// Setup the variety spawner
fn setup_variety_spawner(mut commands: Commands) {
    commands.insert_resource(VarietySpawnTimer::default());
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
    level_data: Option<Res<crate::levels::LevelData>>,
) {
    // Apply level-based spawn rate scaling
    let spawn_rate_multiplier = if let Some(level_data) = &level_data {
        if let Some(current_level) = level_data.levels.get((level_data.current_level - 1) as usize) {
            current_level.enemy_scaling.spawn_rate_multiplier
        } else {
            1.0
        }
    } else {
        1.0
    };

    // Scale the timer based on spawn rate multiplier (higher multiplier = faster spawning)
    let adjusted_delta = time.delta().mul_f32(spawn_rate_multiplier);
    spawn_timer.timer.tick(adjusted_delta);

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

            // Apply level-based health scaling
            let base_health = 1;
            let health = if let Some(level_data) = &level_data {
                if let Some(current_level) = level_data.levels.get((level_data.current_level - 1) as usize) {
                    (base_health as f32 * current_level.enemy_scaling.health_multiplier).ceil() as u32
                } else {
                    base_health
                }
            } else {
                base_health
            };

            let size = DandelionSize::Tiny;
            commands.spawn((
                Sprite {
                    image: asset_server.load(size.asset_path()),
                    color: Color::WHITE,
                    ..default()
                },
                Transform::from_translation(Vec3::new(x, y, 10.0)).with_scale(Vec3::splat(size.scale())),
                Dandelion { health, size },
                EnemyEntity,
            ));

            game_data.dandelion_count += 1;
            area_tracker.total_area += size.visual_area();
        }
    }
}

/// System parameter struct to group related resources
#[derive(SystemParam)]
struct DandelionGameState<'w, 's> {
    commands: Commands<'w, 's>,
    game_data: ResMut<'w, GameData>,
    asset_server: Res<'w, AssetServer>,
    area_tracker: ResMut<'w, DandelionAreaTracker>,
    game_assets: Res<'w, crate::GameAssets>,
}

/// Handle clicks and touches on dandelions
fn handle_dandelion_clicks(
    game_state: DandelionGameState,
    dandelion_query: Query<(Entity, &mut Dandelion, &Transform)>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
) {
    // Check for mouse click
    let mouse_clicked = mouse_input.just_pressed(MouseButton::Left);

    // Check for touch input (any finger touching)
    let touch_started = touches.any_just_pressed();

    if !mouse_clicked && !touch_started {
        return;
    }

    // Get position from mouse or touch
    let world_pos = if mouse_clicked {
        get_world_click_position(&windows, &camera_query)
    } else if touch_started {
        get_world_touch_position(&windows, &camera_query, &touches)
    } else {
        None
    };

    let world_pos = match world_pos {
        Some(pos) => pos,
        None => return,
    };

    debug!("Click/touch at world position: ({:.1}, {:.1})", world_pos.x, world_pos.y);

    // Check if using slash mode or regular click mode
    if game_state.game_data.slash_mode {
        process_slash_attack(game_state, dandelion_query, world_pos);
    } else {
        process_dandelion_hit(game_state, dandelion_query, world_pos);
    }
}

/// Convert screen click to world coordinates
fn get_world_click_position(windows: &Query<&Window>, camera_query: &Query<(&Camera, &GlobalTransform)>) -> Option<Vec2> {
    let window = windows.single().ok()?;
    let (camera, camera_transform) = camera_query.single().ok()?;
    let cursor_pos = window.cursor_position()?;
    camera.viewport_to_world_2d(camera_transform, cursor_pos).ok()
}

/// Convert touch position to world coordinates
fn get_world_touch_position(windows: &Query<&Window>, camera_query: &Query<(&Camera, &GlobalTransform)>, touches: &Touches) -> Option<Vec2> {
    let _window = windows.single().ok()?;
    let (camera, camera_transform) = camera_query.single().ok()?;

    // Get the first touch that just started
    let touch = touches.iter_just_pressed().next()?;
    let touch_pos = touch.position();

    camera.viewport_to_world_2d(camera_transform, touch_pos).ok()
}

/// Check if click hit a dandelion and process the hit
fn process_dandelion_hit(mut game_state: DandelionGameState, mut dandelion_query: Query<(Entity, &mut Dandelion, &Transform)>, click_pos: Vec2) {
    for (entity, mut dandelion, transform) in dandelion_query.iter_mut() {
        let dandelion_pos = transform.translation.truncate();
        let collision_radius = dandelion.size.collision_radius();
        let distance = click_pos.distance(dandelion_pos);

        if distance <= collision_radius {
            damage_dandelion(&mut game_state, entity, &mut dandelion, dandelion_pos);
            break; // Only hit one dandelion per click
        }
    }
}

/// Process slash attack hitting all dandelions along a diagonal line
fn process_slash_attack(mut game_state: DandelionGameState, mut dandelion_query: Query<(Entity, &mut Dandelion, &Transform)>, click_pos: Vec2) {
    let slash_offset = game_state.game_data.slash_offset;

    // Create diagonal slash line from top-right to bottom-left of click position
    let start_pos = click_pos + Vec2::new(slash_offset, slash_offset);
    let end_pos = click_pos - Vec2::new(slash_offset, slash_offset);

    // Spawn visual slash effect
    crate::playing::spawn_slash_effect(&mut game_state.commands, start_pos, end_pos);

    let mut hit_count = 0;

    for (entity, mut dandelion, transform) in dandelion_query.iter_mut() {
        let dandelion_pos = transform.translation.truncate();
        let collision_radius = dandelion.size.collision_radius();

        // Calculate distance from dandelion to slash line
        let distance_to_line = distance_point_to_line_segment(dandelion_pos, start_pos, end_pos);

        if distance_to_line <= collision_radius {
            damage_dandelion(&mut game_state, entity, &mut dandelion, dandelion_pos);
            hit_count += 1;
        }
    }

    if hit_count > 0 {
        debug!(
            "Slash attack hit {} dandelions along line from ({:.1}, {:.1}) to ({:.1}, {:.1})",
            hit_count, start_pos.x, start_pos.y, end_pos.x, end_pos.y
        );
    }
}

/// Calculate distance from a point to a line segment
fn distance_point_to_line_segment(point: Vec2, line_start: Vec2, line_end: Vec2) -> f32 {
    let line_vec = line_end - line_start;
    let line_length_squared = line_vec.length_squared();

    if line_length_squared == 0.0 {
        // Line segment is a point
        return point.distance(line_start);
    }

    // Project point onto line segment
    let t = ((point - line_start).dot(line_vec) / line_length_squared).clamp(0.0, 1.0);
    let projection = line_start + t * line_vec;

    point.distance(projection)
}

/// Apply damage to a dandelion and handle destruction
fn damage_dandelion(game_state: &mut DandelionGameState, entity: Entity, dandelion: &mut Dandelion, position: Vec2) {
    dandelion.health = dandelion.health.saturating_sub(1);

    // Play slash sound effect when dandelion is hit
    play_slash_sound(&mut game_state.commands, &game_state.game_assets);

    if dandelion.health == 0 {
        destroy_dandelion(game_state, entity, dandelion, position);
    }
}

/// Destroy a dandelion and spawn seeds
fn destroy_dandelion(game_state: &mut DandelionGameState, entity: Entity, dandelion: &Dandelion, position: Vec2) {
    let spawn_count = dandelion.size.spawn_count();

    game_state.area_tracker.total_area -= dandelion.size.visual_area();
    spawn_seed_orbs(&mut game_state.commands, &game_state.asset_server, position, spawn_count);
    game_state.commands.entity(entity).despawn();
    game_state.game_data.add_dandelion_kill();
    game_state.game_data.dandelion_count = game_state.game_data.dandelion_count.saturating_sub(1);

    debug!(
        "Dandelion destroyed at ({:.1}, {:.1})! Score: {}, Combo: {}x, Spawning {} seeds",
        position.x, position.y, game_state.game_data.score, game_state.game_data.combo, spawn_count
    );
}

/// Play slash sound effect
fn play_slash_sound(commands: &mut Commands, game_assets: &crate::GameAssets) {
    commands.spawn((AudioPlayer(game_assets.slash_sound.clone()), crate::SoundEntity));
}

/// Debug system to count dandelions (runs less frequently)
fn debug_dandelion_count(dandelions: Query<&Dandelion>, time: Res<Time>) {
    // Only log every 2 seconds to reduce spam
    if (time.elapsed_secs() as u32) % 2 == 0 && time.delta_secs() < 0.1 {
        let count = dandelions.iter().count();
        if count > 0 {
            debug!("Current dandelion count: {}", count);
        }
    }
}

/// Cleanup enemy entities when exiting playing state
fn cleanup_enemies(mut commands: Commands, enemy_entities: Query<Entity, With<EnemyEntity>>) {
    commands.remove_resource::<DandelionSpawnTimer>();
    commands.remove_resource::<DandelionAreaTracker>();
    commands.remove_resource::<VarietySpawnTimer>();

    for entity in &enemy_entities {
        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.despawn();
        }
    }

    debug!("Enemies cleaned up");
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
        debug!(
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
    level_data: Option<Res<crate::levels::LevelData>>,
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
            debug!(
                "Seed orb moving: ({:.1}, {:.1}) -> ({:.1}, {:.1})",
                current_pos.x, current_pos.y, new_pos.x, new_pos.y
            );
        }

        // Spawn new dandelion when timer finishes
        if orb.spawn_timer.finished() {
            if let Ok(mut ec) = commands.get_entity(entity) {
                ec.despawn();
            }

            // Apply level-based health scaling
            let base_health = 1;
            let health = if let Some(level_data) = &level_data {
                if let Some(current_level) = level_data.levels.get((level_data.current_level - 1) as usize) {
                    (base_health as f32 * current_level.enemy_scaling.health_multiplier).ceil() as u32
                } else {
                    base_health
                }
            } else {
                base_health
            };

            let size = DandelionSize::Tiny;
            commands.spawn((
                Sprite {
                    image: asset_server.load(size.asset_path()),
                    color: Color::WHITE,
                    ..default()
                },
                Transform::from_translation(Vec3::new(orb.target_position.x, orb.target_position.y, 10.0)).with_scale(Vec3::splat(size.scale())),
                Dandelion { health, size },
                EnemyEntity,
            ));

            // Remove the seed orb and update dandelion count
            game_data.dandelion_count += 1;
            area_tracker.total_area += size.visual_area();
            debug!(
                "Seed orb spawned new dandelion at ({:.1}, {:.1}) with health {}",
                orb.target_position.x, orb.target_position.y, health
            );
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
    level_data: Option<Res<crate::levels::LevelData>>,
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

                    debug!(
                        "Merging two {:?} dandelions at ({:.1}, {:.1}) and ({:.1}, {:.1}) into {:?} at ({:.1}, {:.1})",
                        dandelion1.size, pos1.x, pos1.y, pos2.x, pos2.y, new_size, merge_pos.x, merge_pos.y
                    );
                } else {
                    debug!("Two {:?} dandelions are close but cannot merge further (already at max size)", dandelion1.size);
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
        if let Ok(mut ec) = commands.get_entity(entity1) {
            ec.despawn();
        }
        if let Ok(mut ec) = commands.get_entity(entity2) {
            ec.despawn();
        }

        // Spawn merge effect
        spawn_merge_effect(&mut commands, merge_pos, new_size);

        // Create new merged dandelion
        // Apply level-based health scaling to merged dandelions
        let base_health = match new_size {
            DandelionSize::Tiny => 1,
            DandelionSize::Small => 2,
            DandelionSize::Medium => 3,
            DandelionSize::Large => 4,
            DandelionSize::Huge => 5,
        };

        let health = if let Some(level_data) = &level_data {
            if let Some(current_level) = level_data.levels.get((level_data.current_level - 1) as usize) {
                (base_health as f32 * current_level.enemy_scaling.health_multiplier).ceil() as u32
            } else {
                base_health
            }
        } else {
            base_health
        };

        let mut entity_commands = commands.spawn((
            Sprite {
                image: asset_server.load(new_size.asset_path()),
                color: Color::WHITE,
                ..default()
            },
            Transform::from_translation(Vec3::new(merge_pos.x, merge_pos.y, 10.0)).with_scale(Vec3::splat(new_size.scale())),
            Dandelion { health, size: new_size },
            EnemyEntity,
        ));

        // Add moving component if huge size
        if new_size == DandelionSize::Huge {
            entity_commands.insert(MovingDandelion::default());
        }

        // Update count (2 removed, 1 added = net -1)
        game_data.dandelion_count = game_data.dandelion_count.saturating_sub(1);
    }
}

/// Spawn merge effect at the given position
fn spawn_merge_effect(commands: &mut Commands, position: Vec2, size: DandelionSize) {
    let effect_scale = size.scale() * 1.5;
    commands.spawn((
        Sprite {
            color: Color::srgba(1.0, 1.0, 0.0, 0.8), // Bright yellow with transparency
            custom_size: Some(Vec2::splat(80.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(position.x, position.y, 20.0)).with_scale(Vec3::splat(effect_scale)),
        MergeEffect {
            timer: Timer::from_seconds(0.5, TimerMode::Once),
            initial_scale: effect_scale,
        },
        EnemyEntity,
    ));
}

/// Update merge effects
fn update_merge_effects(mut commands: Commands, mut effect_query: Query<(Entity, &mut Transform, &mut MergeEffect, &mut Sprite)>, time: Res<Time>) {
    for (entity, mut transform, mut effect, mut sprite) in effect_query.iter_mut() {
        effect.timer.tick(time.delta());

        let progress = effect.timer.elapsed_secs() / effect.timer.duration().as_secs_f32();

        // Scale up and fade out
        let scale = effect.initial_scale * (1.0 + progress * 2.0);
        transform.scale = Vec3::splat(scale);

        let alpha = 1.0 - progress;
        sprite.color.set_alpha(alpha);

        if effect.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

/// Update moving dandelions
fn update_moving_dandelions(mut moving_query: Query<(&mut Transform, &mut MovingDandelion)>, time: Res<Time>, windows: Query<&Window>) {
    if let Ok(window) = windows.single() {
        let margin = 50.0;
        let top_ui_height = window.height() * 0.12;
        let bottom_ui_height = window.height() * 0.08;

        let bounds = Rect::new(
            -window.width() / 2.0 + margin,
            -window.height() / 2.0 + bottom_ui_height + margin,
            window.width() / 2.0 - margin,
            window.height() / 2.0 - top_ui_height - margin,
        );

        for (mut transform, mut moving) in moving_query.iter_mut() {
            moving.direction_change_timer.tick(time.delta());

            // Change direction randomly
            if moving.direction_change_timer.just_finished() {
                let mut rng = rand::thread_rng();
                let angle = rng.gen_range(0.0..std::f32::consts::TAU);
                moving.velocity = Vec2::new(angle.cos(), angle.sin()) * moving.speed;
            }

            let delta = moving.velocity * time.delta_secs();
            let new_pos = transform.translation.truncate() + delta;

            // Bounce off boundaries
            let mut velocity = moving.velocity;
            if new_pos.x < bounds.min.x || new_pos.x > bounds.max.x {
                velocity.x = -velocity.x;
            }
            if new_pos.y < bounds.min.y || new_pos.y > bounds.max.y {
                velocity.y = -velocity.y;
            }
            moving.velocity = velocity;

            // Update position with boundary clamping
            let clamped_pos = new_pos.clamp(bounds.min, bounds.max);
            transform.translation = Vec3::new(clamped_pos.x, clamped_pos.y, transform.translation.z);
        }
    }
}

/// Check collisions between moving huge dandelions and stationary ones
fn check_moving_dandelion_collisions(
    mut commands: Commands,
    moving_query: Query<(Entity, &Transform, &Dandelion), With<MovingDandelion>>,
    mut stationary_query: Query<(Entity, &Transform, &mut Dandelion), (Without<MovingDandelion>, With<Dandelion>, Without<UpgradeCooldown>)>,
    asset_server: Res<AssetServer>,
    mut area_tracker: ResMut<DandelionAreaTracker>,
) {
    let mut upgrades_this_frame = 0;
    const MAX_UPGRADES_PER_FRAME: usize = 10; // Limit to prevent performance issues

    'outer: for (_moving_entity, moving_transform, moving_dandelion) in moving_query.iter() {
        if moving_dandelion.size != DandelionSize::Huge {
            continue;
        }

        let moving_pos = moving_transform.translation.truncate();
        let moving_radius = moving_dandelion.size.collision_radius();

        for (stationary_entity, stationary_transform, mut stationary_dandelion) in stationary_query.iter_mut() {
            if upgrades_this_frame >= MAX_UPGRADES_PER_FRAME {
                break 'outer;
            }

            let stationary_pos = stationary_transform.translation.truncate();
            let stationary_radius = stationary_dandelion.size.collision_radius();

            let distance = moving_pos.distance(stationary_pos);
            let collision_distance = moving_radius + stationary_radius;

            if distance <= collision_distance {
                // Upgrade the stationary dandelion if possible
                if let Some(new_size) = stationary_dandelion.size.next_size() {
                    let old_size = stationary_dandelion.size;

                    // Update area tracker
                    area_tracker.total_area -= stationary_dandelion.size.visual_area();
                    area_tracker.total_area += new_size.visual_area();

                    // Update the dandelion
                    stationary_dandelion.size = new_size;

                    // Add upgrade cooldown to prevent immediate re-upgrading
                    if let Ok(mut entity_commands) = commands.get_entity(stationary_entity) {
                        entity_commands.try_insert(UpgradeCooldown::default());
                    }

                    // Update the sprite and transform
                    if let Ok(mut entity_commands) = commands.get_entity(stationary_entity) {
                        entity_commands.try_insert((
                            Sprite {
                                image: asset_server.load(new_size.asset_path()),
                                color: Color::WHITE,
                                ..default()
                            },
                            Transform::from_translation(Vec3::new(stationary_pos.x, stationary_pos.y, 10.0)).with_scale(Vec3::splat(new_size.scale())),
                        ));
                    }

                    // If it became huge, make it moving too
                    if new_size == DandelionSize::Huge {
                        if let Ok(mut entity_commands) = commands.get_entity(stationary_entity) {
                            entity_commands.try_insert(MovingDandelion::default());
                        }
                    }

                    upgrades_this_frame += 1;
                    debug!("Moving huge dandelion upgraded a {:?} to {:?}", old_size, new_size);
                }
            }
        }
    }
}

/// Update upgrade cooldowns and remove them when expired
fn update_upgrade_cooldowns(mut commands: Commands, mut cooldown_query: Query<(Entity, &mut UpgradeCooldown)>, time: Res<Time>) {
    for (entity, mut cooldown) in cooldown_query.iter_mut() {
        cooldown.timer.tick(time.delta());

        if cooldown.timer.finished() {
            if let Ok(mut entity_commands) = commands.get_entity(entity) {
                entity_commands.remove::<UpgradeCooldown>();
            }
        }
    }
}

/// Spawn variety of dandelions when difficulty threshold is met
fn spawn_variety_dandelions(
    mut commands: Commands,
    mut variety_timer: ResMut<VarietySpawnTimer>,
    time: Res<Time>,
    windows: Query<&Window>,
    asset_server: Res<AssetServer>,
    mut game_data: ResMut<GameData>,
    mut area_tracker: ResMut<DandelionAreaTracker>,
    level_data: Option<Res<crate::levels::LevelData>>,
) {
    // Use level-based difficulty threshold instead of fixed threshold
    let difficulty_threshold = if let Some(level_data) = &level_data {
        if let Some(current_level) = level_data.levels.get((level_data.current_level - 1) as usize) {
            current_level.enemy_scaling.difficulty_threshold
        } else {
            variety_timer.difficulty_threshold
        }
    } else {
        variety_timer.difficulty_threshold
    };

    // Check if we should enable variety spawning
    if !variety_timer.enabled && game_data.score >= difficulty_threshold {
        variety_timer.enabled = true;
        debug!("Variety spawning enabled at score {}", game_data.score);
    }

    if !variety_timer.enabled {
        return;
    }

    // Apply level-based spawn rate scaling
    let spawn_rate_multiplier = if let Some(level_data) = &level_data {
        if let Some(current_level) = level_data.levels.get((level_data.current_level - 1) as usize) {
            current_level.enemy_scaling.spawn_rate_multiplier
        } else {
            1.0
        }
    } else {
        1.0
    };

    // Scale the timer based on spawn rate multiplier
    let adjusted_delta = time.delta().mul_f32(spawn_rate_multiplier);
    variety_timer.timer.tick(adjusted_delta);

    if variety_timer.timer.just_finished() {
        if let Ok(window) = windows.single() {
            let mut rng = rand::thread_rng();

            // Calculate safe spawn area
            let margin = 30.0;
            let top_ui_height = window.height() * 0.12;
            let bottom_ui_height = window.height() * 0.08;

            let min_x = -window.width() / 2.0 + margin;
            let max_x = window.width() / 2.0 - margin;
            let min_y = -window.height() / 2.0 + bottom_ui_height + margin;
            let max_y = window.height() / 2.0 - top_ui_height - margin;

            // Spawn one of each size
            let sizes = [
                DandelionSize::Tiny,
                DandelionSize::Small,
                DandelionSize::Medium,
                DandelionSize::Large,
                DandelionSize::Huge,
            ];

            // Apply level-based health scaling
            let health_multiplier = if let Some(level_data) = &level_data {
                if let Some(current_level) = level_data.levels.get((level_data.current_level - 1) as usize) {
                    current_level.enemy_scaling.health_multiplier
                } else {
                    1.0
                }
            } else {
                1.0
            };

            for size in sizes {
                let x = rng.gen_range(min_x..max_x);
                let y = rng.gen_range(min_y..max_y);

                // Calculate scaled health based on size and level
                let base_health = match size {
                    DandelionSize::Tiny => 1,
                    DandelionSize::Small => 2,
                    DandelionSize::Medium => 3,
                    DandelionSize::Large => 4,
                    DandelionSize::Huge => 5,
                };
                let health = (base_health as f32 * health_multiplier).ceil() as u32;

                let mut entity_commands = commands.spawn((
                    Sprite {
                        image: asset_server.load(size.asset_path()),
                        color: Color::WHITE,
                        ..default()
                    },
                    Transform::from_translation(Vec3::new(x, y, 10.0)).with_scale(Vec3::splat(size.scale())),
                    Dandelion { health, size },
                    EnemyEntity,
                ));

                // Add moving component if huge size
                if size == DandelionSize::Huge {
                    entity_commands.insert(MovingDandelion::default());
                }

                game_data.dandelion_count += 1;
                area_tracker.total_area += size.visual_area();
            }

            debug!(
                "Spawned variety pack of dandelions (difficulty mode) with {}x health scaling",
                health_multiplier
            );
        }
    }
}

/// Spawn a ring of dandelions for testing fire spread
pub fn spawn_dandelion_ring(commands: &mut Commands, assets: &GameAssets, position: Vec2) {
    let radius = 100.0;
    let dandelion_count = 12;
    let size = DandelionSize::Tiny;
    let image_handle = assets.dandelion_tiny.clone();
    for i in 0..dandelion_count {
        let angle = (i as f32) * (std::f32::consts::PI * 2.0 / dandelion_count as f32);
        let offset = Vec2::new(angle.cos(), angle.sin()) * radius;
        let spawn_pos = position + offset;

        commands.spawn((
            Sprite {
                image: image_handle.clone(),
                color: Color::WHITE,
                ..default()
            },
            Transform::from_translation(Vec3::new(spawn_pos.x, spawn_pos.y, 10.0)).with_scale(Vec3::splat(size.scale())),
            Dandelion { health: 1, size },
            EnemyEntity,
        ));
    }
}

/// Calculate the maximum health for a dandelion based on its size and current level scaling
fn calculate_max_health(size: DandelionSize, level_data: Option<&crate::levels::LevelData>) -> u32 {
    let base_health = size.base_health();

    if let Some(level_data) = level_data {
        if let Some(current_level) = level_data.levels.get((level_data.current_level - 1) as usize) {
            (base_health as f32 * current_level.enemy_scaling.health_multiplier).ceil() as u32
        } else {
            base_health
        }
    } else {
        base_health
    }
}

/// Get health bar color based on health percentage
fn get_health_bar_color(health_percentage: f32) -> Color {
    if health_percentage >= 0.75 {
        Color::srgb(0.0, 0.8, 0.0) // Green
    } else if health_percentage >= 0.25 {
        Color::srgb(1.0, 0.6, 0.0) // Orange
    } else {
        Color::srgb(0.9, 0.1, 0.1) // Red
    }
}

/// Spawn a health bar for a damaged dandelion
fn spawn_health_bar(commands: &mut Commands, dandelion_entity: Entity, dandelion_transform: &Transform, dandelion: &Dandelion, max_health: u32) {
    let health_percentage = dandelion.health as f32 / max_health as f32;
    let bar_color = get_health_bar_color(health_percentage);

    // Health bar dimensions
    let bar_width = 20.0;
    let bar_height = 3.0;
    let bar_offset_y = dandelion.size.collision_radius() + 8.0;

    // Position above the dandelion
    let bar_position = dandelion_transform.translation.truncate() + Vec2::new(0.0, bar_offset_y);

    // Create the health bar entity with children
    commands
        .spawn((
            HealthBar { dandelion_entity, max_health },
            Transform::from_translation(Vec3::new(bar_position.x, bar_position.y, 15.0)),
            EnemyEntity,
        ))
        .with_children(|parent| {
            // Background (dark gray)
            parent.spawn((
                Sprite {
                    color: Color::srgb(0.2, 0.2, 0.2),
                    custom_size: Some(Vec2::new(bar_width, bar_height)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
                HealthBarBackground,
                EnemyEntity,
            ));

            // Foreground (colored health fill)
            let fill_width = bar_width * health_percentage;
            parent.spawn((
                Sprite {
                    color: bar_color,
                    custom_size: Some(Vec2::new(fill_width, bar_height)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(-(bar_width - fill_width) / 2.0, 0.0, 1.0)),
                HealthBarFill,
                EnemyEntity,
            ));
        });
}

/// System to manage health bars for damaged dandelions
fn manage_health_bars(
    mut commands: Commands,
    dandelion_query: Query<(Entity, &Transform, &Dandelion), With<Dandelion>>,
    health_bar_query: Query<(Entity, &HealthBar), With<HealthBar>>,
    level_data: Option<Res<crate::levels::LevelData>>,
) {
    // Create a map of existing health bars
    let mut existing_health_bars: std::collections::HashMap<Entity, Entity> = std::collections::HashMap::new();
    for (health_bar_entity, health_bar) in health_bar_query.iter() {
        existing_health_bars.insert(health_bar.dandelion_entity, health_bar_entity);
    }

    for (dandelion_entity, dandelion_transform, dandelion) in dandelion_query.iter() {
        let max_health = calculate_max_health(dandelion.size, level_data.as_deref());
        let health_percentage = dandelion.health as f32 / max_health as f32;

        // Check if dandelion is damaged (less than 100% health)
        if health_percentage < 1.0 {
            // If no health bar exists, create one
            if !existing_health_bars.contains_key(&dandelion_entity) {
                println!("Creating health bar for damaged dandelion: {:.1}% health", health_percentage * 100.0);
                spawn_health_bar(&mut commands, dandelion_entity, dandelion_transform, dandelion, max_health);
            }
        } else {
            // If dandelion is at full health, remove health bar
            if let Some(health_bar_entity) = existing_health_bars.get(&dandelion_entity) {
                if let Ok(mut entity_commands) = commands.get_entity(*health_bar_entity) {
                    entity_commands.despawn();
                }
            }
        }
    }

    // Remove health bars for dandelions that no longer exist
    for (health_bar_entity, health_bar) in health_bar_query.iter() {
        if dandelion_query.get(health_bar.dandelion_entity).is_err() {
            // Dandelion no longer exists, remove health bar
            if let Ok(mut entity_commands) = commands.get_entity(health_bar_entity) {
                entity_commands.despawn();
            }
        }
    }
}

/// System to update health bar positions to follow dandelions
fn update_health_bar_positions(
    dandelion_query: Query<(Entity, &Transform, &Dandelion), (With<Dandelion>, Without<HealthBar>)>,
    mut health_bar_query: Query<(Entity, &HealthBar, &mut Transform, &Children), With<HealthBar>>,
    mut fill_query: Query<(&mut Transform, &mut Sprite), (With<HealthBarFill>, Without<HealthBar>, Without<Dandelion>)>,
) {
    for (_health_bar_entity, health_bar, mut health_bar_transform, children) in health_bar_query.iter_mut() {
        if let Ok((_, dandelion_transform, dandelion)) = dandelion_query.get(health_bar.dandelion_entity) {
            let health_percentage = dandelion.health as f32 / health_bar.max_health as f32;
            let bar_color = get_health_bar_color(health_percentage);
            let bar_offset_y = dandelion.size.collision_radius() + 8.0;
            let bar_position = dandelion_transform.translation.truncate() + Vec2::new(0.0, bar_offset_y);

            // Update health bar parent position
            health_bar_transform.translation = Vec3::new(bar_position.x, bar_position.y, 15.0);

            let bar_width = 20.0;
            let bar_height = 3.0;
            let fill_width = bar_width * health_percentage;

            // Update fill child position and size
            for child in children.iter() {
                if let Ok((mut fill_transform, mut fill_sprite)) = fill_query.get_mut(child) {
                    fill_transform.translation.x = -(bar_width - fill_width) / 2.0;
                    fill_sprite.custom_size = Some(Vec2::new(fill_width, bar_height));
                    fill_sprite.color = bar_color;
                }
            }
        }
    }
}
