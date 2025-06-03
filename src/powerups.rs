use bevy::prelude::*;
use rand::Rng;
use std::collections::HashMap;

use crate::GameAssets;
use crate::GameState;
use crate::enemies::{Dandelion, DandelionAreaTracker};
use crate::pause_menu::PauseState;
use crate::playing::GameData;

// Constants for powerup behavior
const POWERUP_SPAWN_INTERVAL: f32 = 10.0;
const POWERUP_CLICK_RADIUS: f32 = 30.0;
const RABBIT_LIFETIME: f32 = 3.0;
const RABBIT_SPEED: f32 = 120.0;
const RABBIT_EAT_DISTANCE: f32 = 25.0;
const RABBIT_SCALE: f32 = 0.2; // Scale 175px sprite to 35px
const FLAMETHROWER_SCALE: f32 = 0.2; // Scale 175px sprite to 35px
const FIRE_RADIUS: f32 = 100.0;
const FIRE_LIFETIME: f32 = 3.0;
const SPAWN_MARGIN: f32 = 50.0;
const TOP_UI_HEIGHT_RATIO: f32 = 0.12;
const BOTTOM_UI_HEIGHT_RATIO: f32 = 0.08;

/// Component to track rabbit sound duration
#[derive(Component)]
struct RabbitSoundTimer {
    timer: Timer,
}

/// Plugin for handling powerup spawning and behavior
pub struct PowerupsPlugin;

impl Plugin for PowerupsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), setup_powerup_resources)
            .insert_resource(FireManager::new())
            .add_systems(
                Update,
                (
                    spawn_powerups,
                    handle_powerup_clicks,
                    update_powerup_effects,
                    handle_debug_keys,
                    update_rabbits,
                    update_rabbit_sprites,
                    update_fire_system,
                    cleanup_expired_entities,
                    update_rabbit_sound_timers,
                )
                    .run_if(in_state(GameState::Playing))
                    .run_if(in_state(PauseState::Playing)),
            )
            .add_systems(OnExit(GameState::Playing), cleanup_powerups);
    }
}

/// Timer resource for spawning powerups
#[derive(Resource)]
struct PowerupSpawnTimer {
    timer: Timer,
}

impl Default for PowerupSpawnTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(POWERUP_SPAWN_INTERVAL, TimerMode::Repeating),
        }
    }
}

/// Types of powerups available
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PowerupType {
    Bunny,
    Flamethrower,
}

impl PowerupType {
    /// Get all available powerup types
    pub fn all() -> Vec<Self> {
        vec![PowerupType::Bunny, PowerupType::Flamethrower]
    }

    /// Get a random powerup type
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        let powerups = Self::all();
        powerups[rng.gen_range(0..powerups.len())]
    }
}

/// Component for powerup entities
#[derive(Component)]
pub struct Powerup {
    pub powerup_type: PowerupType,
}

/// Component for powerup spawn effect (blue expanding circle)
#[derive(Component)]
struct PowerupEffect {
    timer: Timer,
    initial_scale: f32,
}

/// Marker component for powerup entities
#[derive(Component)]
pub struct PowerupEntity;

/// Resource to track dandelion targeting to prevent rabbits from swarming the same target
#[derive(Resource, Default)]
struct RabbitTargeting {
    /// Maps dandelion entity to the rabbit entity targeting it
    targets: HashMap<Entity, Entity>,
}

impl RabbitTargeting {
    /// Reserve a dandelion for a specific rabbit
    fn claim_target(&mut self, rabbit: Entity, dandelion: Entity) {
        self.targets.insert(dandelion, rabbit);
    }

    /// Check if a dandelion is already being targeted
    fn is_targeted(&self, dandelion: Entity) -> bool {
        self.targets.contains_key(&dandelion)
    }

    /// Remove a target claim (when rabbit dies or changes target)
    fn release_target(&mut self, dandelion: Entity) {
        self.targets.remove(&dandelion);
    }

    /// Get the rabbit targeting a specific dandelion
    fn get_targeting_rabbit(&self, dandelion: Entity) -> Option<Entity> {
        self.targets.get(&dandelion).copied()
    }

    /// Clear all targets for a specific rabbit (when rabbit dies)
    fn clear_rabbit_targets(&mut self, rabbit: Entity) {
        self.targets.retain(|_, &mut targeting_rabbit| targeting_rabbit != rabbit);
    }

    /// Clear all targets (used during cleanup to prevent memory leaks)
    fn clear(&mut self) {
        self.targets.clear();
    }
}

/// Component for rabbit entities
#[derive(Component)]
pub struct Rabbit {
    target: Option<Entity>,
    dandelions_eaten: u32,
    lifetime: Timer,
    speed: f32,
    facing_right: bool, // Track movement direction for sprite flipping
}

impl Default for Rabbit {
    fn default() -> Self {
        Self {
            target: None,
            dandelions_eaten: 0,
            lifetime: Timer::from_seconds(RABBIT_LIFETIME, TimerMode::Once),
            speed: RABBIT_SPEED,
            facing_right: false, // Default faces left (original sprite direction)
        }
    }
}

/// Component for fire ignition entities
#[derive(Component)]
pub struct FireIgnition {
    radius: f32,
    damage_timer: Timer,
    lifetime: Timer,
    generation: u32, // Track fire generation to limit chain reactions
}

impl Default for FireIgnition {
    fn default() -> Self {
        Self {
            radius: FIRE_RADIUS,
            damage_timer: Timer::from_seconds(0.2, TimerMode::Repeating),
            lifetime: Timer::from_seconds(FIRE_LIFETIME, TimerMode::Once),
            generation: 0,
        }
    }
}

/// Resource to efficiently track active fires and batch damage calculations
#[derive(Resource, Default)]
struct FireManager {
    /// Spatial grid for efficient collision detection
    active_fires: Vec<FireData>,
    /// Queue of pending fire spawns to batch process
    pending_fires: Vec<PendingFire>,
    /// Timer for batched processing
    batch_timer: Timer,
}

#[derive(Clone)]
struct FireData {
    position: Vec2,
    radius: f32,
    generation: u32,
}

struct PendingFire {
    position: Vec2,
    generation: u32,
}

impl FireManager {
    const MAX_GENERATION: u32 = 5; // Limit chain reaction depth
    const BATCH_INTERVAL: f32 = 0.05; // Process fires every 50ms for faster spreading

    fn new() -> Self {
        Self {
            active_fires: Vec::new(),
            pending_fires: Vec::new(),
            batch_timer: Timer::from_seconds(Self::BATCH_INTERVAL, TimerMode::Repeating),
        }
    }
}

/// Component for fire preview radius indicator
#[derive(Component)]
struct FirePreview;

/// Setup powerup resources including timer and targeting
fn setup_powerup_resources(mut commands: Commands) {
    commands.insert_resource(PowerupSpawnTimer::default());
    commands.insert_resource(RabbitTargeting::default());
    commands.insert_resource(FireManager::new());
}

/// Spawn powerups at random positions
fn spawn_powerups(mut commands: Commands, mut spawn_timer: ResMut<PowerupSpawnTimer>, time: Res<Time>, windows: Query<&Window>, assets: Res<GameAssets>) {
    spawn_timer.timer.tick(time.delta());

    if spawn_timer.timer.just_finished() {
        if let Ok(window) = windows.single() {
            let position = calculate_random_spawn_position(window);
            let powerup_type = PowerupType::random();
            spawn_powerup_with_effect(&mut commands, &assets, position, powerup_type);
            debug!("Spawned {:?} powerup at ({:.1}, {:.1})", powerup_type, position.x, position.y);
        }
    }
}

/// Calculate a random spawn position within safe boundaries
fn calculate_random_spawn_position(window: &Window) -> Vec2 {
    let mut rng = rand::thread_rng();

    let top_ui_height = window.height() * TOP_UI_HEIGHT_RATIO;
    let bottom_ui_height = window.height() * BOTTOM_UI_HEIGHT_RATIO;

    let min_x = -window.width() / 2.0 + SPAWN_MARGIN;
    let max_x = window.width() / 2.0 - SPAWN_MARGIN;
    let min_y = -window.height() / 2.0 + bottom_ui_height + SPAWN_MARGIN;
    let max_y = window.height() / 2.0 - top_ui_height - SPAWN_MARGIN;

    Vec2::new(rng.gen_range(min_x..max_x), rng.gen_range(min_y..max_y))
}

/// Spawn a powerup with its visual effect
fn spawn_powerup_with_effect(commands: &mut Commands, assets: &GameAssets, position: Vec2, powerup_type: PowerupType) {
    // Spawn the powerup
    let image_handle = match powerup_type {
        PowerupType::Bunny => assets.bunny.clone(),
        PowerupType::Flamethrower => assets.flamethrower.clone(),
    };
    commands.spawn((
        Sprite {
            image: image_handle,
            ..default()
        },
        Transform::from_translation(Vec3::new(position.x, position.y, 15.0)).with_scale(Vec3::splat(if powerup_type == PowerupType::Bunny {
            RABBIT_SCALE
        } else {
            FLAMETHROWER_SCALE
        })),
        Powerup { powerup_type },
        PowerupEntity,
    ));
}

/// Handle clicks and touches on powerups to trigger them immediately
fn handle_powerup_clicks(
    mut commands: Commands,
    mouse_input: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    powerup_query: Query<(Entity, &Powerup, &Transform)>,
    assets: Res<GameAssets>,
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

    for (entity, powerup, transform) in powerup_query.iter() {
        let powerup_pos = transform.translation.truncate();
        let distance = world_pos.distance(powerup_pos);
        if distance <= POWERUP_CLICK_RADIUS {
            use_powerup(powerup.powerup_type, powerup_pos, &mut commands, &assets);
            if let Ok(mut ec) = commands.get_entity(entity) {
                ec.despawn();
            }
            debug!("Triggered {:?} powerup at ({:.1}, {:.1})", powerup.powerup_type, powerup_pos.x, powerup_pos.y);
            break;
        }
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

/// Execute powerup effect at the specified location
fn use_powerup(powerup_type: PowerupType, position: Vec2, commands: &mut Commands, assets: &GameAssets) {
    match powerup_type {
        PowerupType::Bunny => {
            spawn_rabbits(commands, assets, position);
            debug!("Bunny powerup activated at ({:.1}, {:.1})", position.x, position.y);
        }
        PowerupType::Flamethrower => {
            spawn_fire_ignition(commands, assets, position);
            play_flamethrower_sound(commands, assets);
            debug!("Flamethrower powerup activated at ({:.1}, {:.1})", position.x, position.y);
        }
    }
}

/// Update powerup spawn effects (blue expanding circles)
fn update_powerup_effects(mut commands: Commands, mut effect_query: Query<(Entity, &mut PowerupEffect, &mut Transform, &mut Sprite)>, time: Res<Time>) {
    for (entity, mut effect, mut transform, mut sprite) in effect_query.iter_mut() {
        effect.timer.tick(time.delta());

        let progress = effect.timer.elapsed_secs() / effect.timer.duration().as_secs_f32();

        // Expand the effect
        let scale = effect.initial_scale + progress * 2.0;
        transform.scale = Vec3::splat(scale);

        // Fade out the effect
        let alpha = 0.8 * (1.0 - progress);
        sprite.color.set_alpha(alpha);

        if effect.timer.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}

/// Handle debug keys for testing - F for fire, B for bunny
fn handle_debug_keys(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    assets: Res<GameAssets>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
) {
    let spawn_position = if let Some(world_pos) = get_world_click_position(&windows, &camera_query) {
        // Use cursor position if available
        world_pos
    } else {
        // Default to center of screen
        Vec2::ZERO
    };

    if keyboard_input.just_pressed(KeyCode::KeyF) {
        use_powerup(PowerupType::Flamethrower, spawn_position, &mut commands, &assets);
        debug!("Debug: Spawned fire at ({:.1}, {:.1})", spawn_position.x, spawn_position.y);
    }

    if keyboard_input.just_pressed(KeyCode::KeyB) {
        use_powerup(PowerupType::Bunny, spawn_position, &mut commands, &assets);
        debug!("Debug: Spawned bunny at ({:.1}, {:.1})", spawn_position.x, spawn_position.y);
    }

    if keyboard_input.just_pressed(KeyCode::KeyD) {
        crate::enemies::spawn_dandelion_ring(&mut commands, &assets, spawn_position);
        debug!("Debug: Spawned dandelion ring at ({:.1}, {:.1})", spawn_position.x, spawn_position.y);
    }
}

/// Spawn 3 rabbits at the specified location
fn spawn_rabbits(commands: &mut Commands, assets: &GameAssets, position: Vec2) {
    for i in 0..3 {
        let angle = (i as f32) * (2.0 * std::f32::consts::PI / 3.0); // 120 degrees apart
        let offset = Vec2::new(angle.cos(), angle.sin()) * 20.0;
        let spawn_pos = position + offset;

        commands.spawn((
            Sprite {
                image: assets.bunny.clone(),
                ..default()
            },
            Transform::from_translation(Vec3::new(spawn_pos.x, spawn_pos.y, 12.0)).with_scale(Vec3::splat(RABBIT_SCALE)),
            Rabbit::default(),
            PowerupEntity,
        ));
    }
}

/// Spawn fire ignition at the specified location
fn spawn_fire_ignition(commands: &mut Commands, assets: &GameAssets, position: Vec2) {
    spawn_fire_ignition_with_generation(commands, assets, position, 0);
}

/// Spawn fire ignition with specific generation for chain reactions
fn spawn_fire_ignition_with_generation(commands: &mut Commands, assets: &GameAssets, position: Vec2, generation: u32) {
    commands.spawn((
        Sprite {
            image: assets.flamethrower.clone(),
            color: Color::srgba(1.0, 0.4, 0.0, 0.9),
            ..default()
        },
        Transform::from_translation(Vec3::new(position.x, position.y, 12.0)).with_scale(Vec3::splat(FLAMETHROWER_SCALE)),
        FireIgnition {
            generation,
            ..Default::default()
        },
        PowerupEntity,
    ));
}

/// Component for tracking sprite animation state (future expansion)
#[derive(Component)]
struct RabbitSprite {
    // Future: animation state, frame timer, etc.
    // For now just used as a marker for rabbit sprites
}

/// Update rabbit behavior - target and move towards dandelions with team coordination
fn update_rabbits(
    mut commands: Commands,
    mut rabbit_query: Query<(Entity, &mut Transform, &mut Rabbit)>,
    dandelion_query: Query<(Entity, &Transform, &Dandelion), (With<Dandelion>, Without<Rabbit>)>,
    time: Res<Time>,
    assets: Res<GameAssets>,
    mut game_data: ResMut<GameData>,
    mut area_tracker: ResMut<DandelionAreaTracker>,
    mut rabbit_targeting: ResMut<RabbitTargeting>,
) {
    // Clean up any invalid targets from the targeting resource
    let valid_dandelions: std::collections::HashSet<Entity> = dandelion_query.iter().map(|(e, _, _)| e).collect();
    rabbit_targeting.targets.retain(|&dandelion, _| valid_dandelions.contains(&dandelion));

    for (rabbit_entity, mut rabbit_transform, mut rabbit) in rabbit_query.iter_mut() {
        rabbit.lifetime.tick(time.delta());

        // Find optimal dandelion target if no current target or target is invalid
        if rabbit.target.is_none() || dandelion_query.get(rabbit.target.unwrap()).is_err() {
            if let Some(old_target) = rabbit.target {
                rabbit_targeting.release_target(old_target);
            }

            let new_target = find_best_dandelion_target(rabbit_entity, rabbit_transform.translation.truncate(), &dandelion_query, &rabbit_targeting);

            if let Some(target_entity) = new_target {
                rabbit_targeting.claim_target(rabbit_entity, target_entity);
            }

            rabbit.target = new_target;
        }

        // Move towards target and handle dandelion consumption
        if let Some(target_entity) = rabbit.target {
            if let Ok((_, target_transform, target_dandelion)) = dandelion_query.get(target_entity) {
                let rabbit_pos = rabbit_transform.translation.truncate();
                let target_pos = target_transform.translation.truncate();
                let direction = (target_pos - rabbit_pos).normalize_or_zero();

                // Update facing direction based on movement
                if direction.x > 0.1 {
                    rabbit.facing_right = true;
                } else if direction.x < -0.1 {
                    rabbit.facing_right = false;
                }

                let movement = direction * rabbit.speed * time.delta_secs();
                rabbit_transform.translation += movement.extend(0.0);

                let distance = rabbit_pos.distance(target_pos);
                if distance <= RABBIT_EAT_DISTANCE {
                    handle_rabbit_eating_dandelion(
                        &mut commands,
                        &assets,
                        rabbit_entity,
                        target_entity,
                        target_dandelion,
                        &mut rabbit,
                        rabbit_transform.translation.truncate(),
                        &mut rabbit_targeting,
                        &mut game_data,
                        &mut area_tracker,
                    );
                }
            }
        }
    }
}

/// Handle a rabbit eating a dandelion and potential rabbit reproduction
fn handle_rabbit_eating_dandelion(
    commands: &mut Commands,
    assets: &GameAssets,
    rabbit_entity: Entity,
    target_entity: Entity,
    target_dandelion: &Dandelion,
    rabbit: &mut Rabbit,
    rabbit_pos: Vec2,
    rabbit_targeting: &mut RabbitTargeting,
    game_data: &mut GameData,
    area_tracker: &mut DandelionAreaTracker,
) {
    // Play rabbit eating sound
    play_rabbit_sound(commands, assets);

    // Release the target claim and remove dandelion
    rabbit_targeting.release_target(target_entity);
    if let Ok(mut ec) = commands.get_entity(target_entity) {
        ec.despawn();
    }

    // Update game tracking
    area_tracker.total_area -= target_dandelion.size.visual_area();
    game_data.add_dandelion_kill();
    game_data.dandelion_count = game_data.dandelion_count.saturating_sub(1);

    rabbit.dandelions_eaten += 1;
    rabbit.target = None;

    let size_name = get_dandelion_size_name(target_dandelion.size);

    debug!("Rabbit ate a {} dandelion! Total eaten: {}", size_name, rabbit.dandelions_eaten);

    // Rabbit reproduction after eating 2 dandelions
    if rabbit.dandelions_eaten >= 2 {
        spawn_rabbits(commands, assets, rabbit_pos);
        rabbit_targeting.clear_rabbit_targets(rabbit_entity);
        if let Ok(mut ec) = commands.get_entity(rabbit_entity) {
            ec.despawn();
        }
        debug!("Rabbit spawned new rabbits after eating 2 dandelions!");
    }
}

/// Get a human-readable name for dandelion size
fn get_dandelion_size_name(size: crate::enemies::DandelionSize) -> &'static str {
    match size {
        crate::enemies::DandelionSize::Tiny => "tiny",
        crate::enemies::DandelionSize::Small => "small",
        crate::enemies::DandelionSize::Medium => "medium",
        crate::enemies::DandelionSize::Large => "large",
        crate::enemies::DandelionSize::Huge => "huge",
    }
}

/// Get size bonus multiplier for targeting priority
fn get_dandelion_size_bonus(size: crate::enemies::DandelionSize) -> f32 {
    match size {
        crate::enemies::DandelionSize::Tiny => 1.0,
        crate::enemies::DandelionSize::Small => 1.2,
        crate::enemies::DandelionSize::Medium => 1.5,
        crate::enemies::DandelionSize::Large => 2.0,
        crate::enemies::DandelionSize::Huge => 3.0,
    }
}

/// Find the best dandelion target for a rabbit using team coordination
fn find_best_dandelion_target(
    rabbit_entity: Entity,
    rabbit_pos: Vec2,
    dandelion_query: &Query<(Entity, &Transform, &Dandelion), (With<Dandelion>, Without<Rabbit>)>,
    rabbit_targeting: &RabbitTargeting,
) -> Option<Entity> {
    let mut best_target = None;
    let mut best_score = f32::NEG_INFINITY;

    // First pass: try to find untargeted dandelions (preferred)
    for (dandelion_entity, dandelion_transform, dandelion) in dandelion_query.iter() {
        // Skip if already being targeted by another rabbit
        if rabbit_targeting.is_targeted(dandelion_entity) && rabbit_targeting.get_targeting_rabbit(dandelion_entity) != Some(rabbit_entity) {
            continue;
        }

        let dandelion_pos = dandelion_transform.translation.truncate();
        let distance = rabbit_pos.distance(dandelion_pos);

        // Calculate score based on distance and dandelion size
        let size_bonus = get_dandelion_size_bonus(dandelion.size);
        let score = (1000.0 / (distance + 1.0)) * size_bonus;

        if score > best_score {
            best_score = score;
            best_target = Some(dandelion_entity);
        }
    }

    // If no untargeted dandelion found, fallback to random nearby dandelion
    if best_target.is_none() {
        best_target = find_fallback_dandelion_target(rabbit_pos, dandelion_query);
    }

    best_target
}

/// Find a fallback dandelion target when no optimal target is available  
fn find_fallback_dandelion_target(
    rabbit_pos: Vec2,
    dandelion_query: &Query<(Entity, &Transform, &Dandelion), (With<Dandelion>, Without<Rabbit>)>,
) -> Option<Entity> {
    let close_dandelions: Vec<Entity> = dandelion_query
        .iter()
        .filter_map(|(entity, transform, _)| {
            let distance = rabbit_pos.distance(transform.translation.truncate());
            if distance <= 200.0 { Some(entity) } else { None }
        })
        .collect();

    if !close_dandelions.is_empty() {
        let mut rng = rand::thread_rng();
        let random_index = rng.gen_range(0..close_dandelions.len());
        Some(close_dandelions[random_index])
    } else {
        None
    }
}

/// New optimized fire system with batched processing
fn update_fire_system(
    mut commands: Commands,
    mut fire_query: Query<(Entity, &mut Transform, &mut FireIgnition, &mut Sprite), With<FireIgnition>>,
    dandelion_query: Query<(Entity, &Transform, &Dandelion), (With<Dandelion>, Without<FireIgnition>)>,
    mut fire_manager: ResMut<FireManager>,
    time: Res<Time>,
    assets: Res<GameAssets>,
    mut game_data: ResMut<GameData>,
    mut area_tracker: ResMut<DandelionAreaTracker>,
) {
    // Update fire manager timer (kept for potential future optimizations)
    fire_manager.batch_timer.tick(time.delta());

    // Clear and rebuild active fires list to prevent stale data accumulation
    fire_manager.active_fires.clear();

    for (fire_entity, mut fire_transform, mut fire, mut sprite) in fire_query.iter_mut() {
        fire.damage_timer.tick(time.delta());
        fire.lifetime.tick(time.delta());

        // Store active fire data for batch processing
        fire_manager.active_fires.push(FireData {
            position: fire_transform.translation.truncate(),
            radius: fire.radius,
            generation: fire.generation,
        });

        // Fire visual effects
        let lifetime_progress = fire.lifetime.elapsed_secs() / fire.lifetime.duration().as_secs_f32();
        let pulse = (time.elapsed_secs() * 12.0).sin() * 0.15 + 1.0;
        fire_transform.scale = Vec3::splat(FLAMETHROWER_SCALE * pulse);
        let alpha = (1.0 - lifetime_progress) * 0.95;
        sprite.color = Color::srgba(1.0, 0.4, 0.0, alpha);

        // Remove expired fires
        if fire.lifetime.just_finished() {
            commands.entity(fire_entity).despawn();
        }
    }

    // Process fire damage every frame for immediate spreading
    let mut dandelions_to_destroy = Vec::new();

    // Single pass through all dandelions, check against all fires
    for (dandelion_entity, dandelion_transform, dandelion) in dandelion_query.iter() {
        let dandelion_pos = dandelion_transform.translation.truncate();

        // Check if this dandelion is hit by any fire
        for fire_data in &fire_manager.active_fires {
            let distance = fire_data.position.distance(dandelion_pos);
            if distance <= fire_data.radius {
                dandelions_to_destroy.push((dandelion_entity, dandelion_pos, dandelion.size, fire_data.generation));
                break; // One hit is enough
            }
        }
    }

    // Process destroyed dandelions and queue chain fires
    for (dandelion_entity, dandelion_pos, dandelion_size, generation) in dandelions_to_destroy {
        // Remove the dandelion
        commands.entity(dandelion_entity).despawn();

        // Update tracking
        area_tracker.total_area -= dandelion_size.visual_area();
        game_data.add_dandelion_kill();
        game_data.dandelion_count = game_data.dandelion_count.saturating_sub(1);

        // Queue chain fire if generation limit not exceeded
        if generation < FireManager::MAX_GENERATION {
            fire_manager.pending_fires.push(PendingFire {
                position: dandelion_pos,
                generation: generation + 1,
            });
        }
    }

    // Spawn pending chain fires immediately for instant spreading
    for pending_fire in fire_manager.pending_fires.drain(..) {
        spawn_fire_ignition_with_generation(&mut commands, &assets, pending_fire.position, pending_fire.generation);
    }
}

/// Clean up expired entities (rabbits and fires)
fn cleanup_expired_entities(
    mut commands: Commands,
    rabbit_query: Query<(Entity, &Rabbit)>,
    fire_query: Query<(Entity, &FireIgnition)>,
    mut rabbit_targeting: ResMut<RabbitTargeting>,
) {
    // Clean up expired rabbits
    for (entity, rabbit) in rabbit_query.iter() {
        if rabbit.lifetime.just_finished() {
            rabbit_targeting.clear_rabbit_targets(entity);
            if let Ok(mut ec) = commands.get_entity(entity) {
                ec.despawn();
            }
            debug!("Rabbit expired after 3 seconds");
        }
    }

    // Clean up expired fires
    for (entity, fire) in fire_query.iter() {
        if fire.lifetime.just_finished() {
            if let Ok(mut ec) = commands.get_entity(entity) {
                ec.despawn();
            }
        }
    }
}

/// Play rabbit sound effect for limited duration
fn play_rabbit_sound(commands: &mut Commands, game_assets: &GameAssets) {
    commands.spawn((
        AudioPlayer(game_assets.rabbit_sound.clone()),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Once,
            ..default()
        },
        RabbitSoundTimer {
            timer: Timer::from_seconds(0.4, TimerMode::Once),
        },
        crate::SoundEntity,
    ));
}

/// Play flamethrower sound effect for limited duration
fn play_flamethrower_sound(commands: &mut Commands, game_assets: &GameAssets) {
    commands.spawn((
        AudioPlayer(game_assets.flamethrower_sound.clone()),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Once,
            ..default()
        },
        RabbitSoundTimer {
            timer: Timer::from_seconds(0.6, TimerMode::Once),
        },
        crate::SoundEntity,
    ));
}

/// Update rabbit sound timers and despawn audio entities when timer expires
fn update_rabbit_sound_timers(mut commands: Commands, time: Res<Time>, mut sound_query: Query<(Entity, &mut RabbitSoundTimer)>) {
    for (entity, mut sound_timer) in sound_query.iter_mut() {
        sound_timer.timer.tick(time.delta());
        if sound_timer.timer.finished() {
            if let Ok(mut ec) = commands.get_entity(entity) {
                ec.despawn();
            }
        }
    }
}

/// Cleanup powerup entities when exiting playing state
fn cleanup_powerups(mut commands: Commands, powerup_entities: Query<Entity, With<PowerupEntity>>, rabbit_targeting: Option<ResMut<RabbitTargeting>>) {
    // Clear rabbit targeting HashMap before removing resource
    if let Some(mut targeting) = rabbit_targeting {
        targeting.clear();
    }

    commands.remove_resource::<PowerupSpawnTimer>();
    commands.remove_resource::<RabbitTargeting>();
    commands.remove_resource::<FireManager>();

    for entity in &powerup_entities {
        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.despawn();
        }
    }

    debug!("Powerups cleaned up");
}

/// Update rabbit sprite orientation based on movement direction
fn update_rabbit_sprites(mut rabbit_query: Query<(&Rabbit, &mut Sprite), With<Rabbit>>) {
    for (rabbit, mut sprite) in rabbit_query.iter_mut() {
        // Flip sprite horizontally based on facing direction
        // Original sprite faces left, so flip_x = true when facing right
        sprite.flip_x = rabbit.facing_right;
    }
}
