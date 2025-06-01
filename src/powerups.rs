use bevy::prelude::*;
use rand::Rng;
use std::collections::HashMap;

use crate::GameState;
use crate::enemies::{Dandelion, DandelionAreaTracker};
use crate::pause_menu::PauseState;
use crate::playing::GameData;

/// Plugin for handling powerup spawning and behavior
pub struct PowerupsPlugin;

impl Plugin for PowerupsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), setup_powerup_timer)
            .add_systems(
                Update,
                (
                    spawn_powerups,
                    handle_powerup_clicks,
                    update_powerup_effects,
                    handle_powerup_usage,
                    update_rabbits,
                    update_fire_ignition,
                    cleanup_expired_rabbits,
                    cleanup_expired_fire,
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
            timer: Timer::from_seconds(15.0, TimerMode::Repeating),
        }
    }
}

/// Resource to track the currently selected powerup
#[derive(Resource, Default)]
pub struct SelectedPowerup {
    pub powerup_type: Option<PowerupType>,
}

/// Types of powerups available
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PowerupType {
    Bunny,
    Flamethrower,
}

impl PowerupType {
    /// Get the asset path for this powerup
    pub fn asset_path(&self) -> &'static str {
        match self {
            PowerupType::Bunny => "bunny.png",
            PowerupType::Flamethrower => "flamethrower.png",
        }
    }

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
struct PowerupEntity;

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
}

/// Component for rabbit entities
#[derive(Component)]
struct Rabbit {
    target: Option<Entity>,
    dandelions_eaten: u32,
    lifetime: Timer,
    speed: f32,
}

impl Default for Rabbit {
    fn default() -> Self {
        Self {
            target: None,
            dandelions_eaten: 0,
            lifetime: Timer::from_seconds(3.0, TimerMode::Once),
            speed: 120.0,
        }
    }
}

/// Component for fire ignition entities
#[derive(Component)]
struct FireIgnition {
    radius: f32,
    damage_timer: Timer,
    lifetime: Timer,
}

impl Default for FireIgnition {
    fn default() -> Self {
        Self {
            radius: 100.0,
            damage_timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            lifetime: Timer::from_seconds(2.0, TimerMode::Once),
        }
    }
}

/// Setup the powerup spawn timer
fn setup_powerup_timer(mut commands: Commands) {
    commands.insert_resource(PowerupSpawnTimer::default());
    commands.insert_resource(SelectedPowerup::default());
    commands.insert_resource(RabbitTargeting::default());
}

/// Spawn powerups at random positions
fn spawn_powerups(
    mut commands: Commands,
    mut spawn_timer: ResMut<PowerupSpawnTimer>,
    time: Res<Time>,
    windows: Query<&Window>,
    asset_server: Res<AssetServer>,
) {
    spawn_timer.timer.tick(time.delta());

    if spawn_timer.timer.just_finished() {
        if let Ok(window) = windows.single() {
            let mut rng = rand::thread_rng();

            // Calculate safe spawn area similar to dandelions
            let margin = 50.0;
            let top_ui_height = window.height() * 0.12;
            let bottom_ui_height = window.height() * 0.08;

            let min_x = -window.width() / 2.0 + margin;
            let max_x = window.width() / 2.0 - margin;
            let min_y = -window.height() / 2.0 + bottom_ui_height + margin;
            let max_y = window.height() / 2.0 - top_ui_height - margin;

            let x = rng.gen_range(min_x..max_x);
            let y = rng.gen_range(min_y..max_y);

            let powerup_type = PowerupType::random();

            // Spawn the powerup
            commands.spawn((
                Sprite {
                    image: asset_server.load(powerup_type.asset_path()),
                    ..default()
                },
                Transform::from_translation(Vec3::new(x, y, 15.0)).with_scale(Vec3::splat(0.8)),
                Powerup { powerup_type },
                PowerupEntity,
            ));

            // Spawn blue effect
            commands.spawn((
                Sprite {
                    image: asset_server.load("seed.png"),
                    color: Color::srgba(0.0, 0.5, 1.0, 0.8),
                    ..default()
                },
                Transform::from_translation(Vec3::new(x, y, 12.0)).with_scale(Vec3::splat(0.1)),
                PowerupEffect {
                    timer: Timer::from_seconds(1.0, TimerMode::Once),
                    initial_scale: 0.1,
                },
                PowerupEntity,
            ));

            info!("Spawned {:?} powerup at ({:.1}, {:.1})", powerup_type, x, y);
        }
    }
}

/// Handle clicks on powerups to select them
fn handle_powerup_clicks(
    mut commands: Commands,
    powerup_query: Query<(Entity, &Powerup, &Transform)>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut selected_powerup: ResMut<SelectedPowerup>,
) {
    if !mouse_input.just_pressed(MouseButton::Left) {
        return;
    }

    let world_pos = match get_world_click_position(&windows, &camera_query) {
        Some(pos) => pos,
        None => return,
    };

    for (entity, powerup, transform) in powerup_query.iter() {
        let powerup_pos = transform.translation.truncate();
        let distance = world_pos.distance(powerup_pos);
        let click_radius = 30.0; // Reasonable click area

        if distance <= click_radius {
            // Select the powerup
            selected_powerup.powerup_type = Some(powerup.powerup_type);

            // Remove the powerup from the world
            commands.entity(entity).despawn();

            info!("Selected {:?} powerup", powerup.powerup_type);
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

/// Handle using selected powerups
fn handle_powerup_usage(
    mut commands: Commands,
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut selected_powerup: ResMut<SelectedPowerup>,
    asset_server: Res<AssetServer>,
) {
    if !mouse_input.just_pressed(MouseButton::Left) {
        return;
    }

    if let Some(powerup_type) = selected_powerup.powerup_type {
        let world_pos = match get_world_click_position(&windows, &camera_query) {
            Some(pos) => pos,
            None => return,
        };

        // Use the powerup at the clicked location
        use_powerup(powerup_type, world_pos, &mut commands, &asset_server);

        // Clear the selected powerup
        selected_powerup.powerup_type = None;

        info!("Used {:?} powerup at ({:.1}, {:.1})", powerup_type, world_pos.x, world_pos.y);
    }
}

/// Execute powerup effect at the specified location
fn use_powerup(powerup_type: PowerupType, position: Vec2, commands: &mut Commands, asset_server: &AssetServer) {
    match powerup_type {
        PowerupType::Bunny => {
            spawn_rabbits(commands, asset_server, position);
            info!("Bunny powerup activated at ({:.1}, {:.1})", position.x, position.y);
        }
        PowerupType::Flamethrower => {
            spawn_fire_ignition(commands, asset_server, position);
            info!("Flamethrower powerup activated at ({:.1}, {:.1})", position.x, position.y);
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

/// Spawn 3 rabbits at the specified location
fn spawn_rabbits(commands: &mut Commands, asset_server: &AssetServer, position: Vec2) {
    for i in 0..3 {
        let angle = (i as f32) * (2.0 * std::f32::consts::PI / 3.0); // 120 degrees apart
        let offset = Vec2::new(angle.cos(), angle.sin()) * 20.0;
        let spawn_pos = position + offset;

        commands.spawn((
            Sprite {
                image: asset_server.load("bunny.png"),
                ..default()
            },
            Transform::from_translation(Vec3::new(spawn_pos.x, spawn_pos.y, 12.0)).with_scale(Vec3::splat(0.6)),
            Rabbit::default(),
            PowerupEntity,
        ));
    }
}

/// Spawn fire ignition at the specified location
fn spawn_fire_ignition(commands: &mut Commands, asset_server: &AssetServer, position: Vec2) {
    commands.spawn((
        Sprite {
            image: asset_server.load("seed.png"), // Using seed as fire effect placeholder
            color: Color::srgba(1.0, 0.3, 0.0, 0.8),
            ..default()
        },
        Transform::from_translation(Vec3::new(position.x, position.y, 12.0)).with_scale(Vec3::splat(1.0)),
        FireIgnition::default(),
        PowerupEntity,
    ));
}

/// Update rabbit behavior - target and move towards dandelions with team coordination
fn update_rabbits(
    mut commands: Commands,
    mut rabbit_query: Query<(Entity, &mut Transform, &mut Rabbit)>,
    dandelion_query: Query<(Entity, &Transform, &Dandelion), (With<Dandelion>, Without<Rabbit>)>,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut game_data: ResMut<GameData>,
    mut area_tracker: ResMut<DandelionAreaTracker>,
    mut rabbit_targeting: ResMut<RabbitTargeting>,
) {
    // First, clean up any invalid targets from the targeting resource
    let valid_dandelions: std::collections::HashSet<Entity> = dandelion_query.iter().map(|(e, _, _)| e).collect();
    rabbit_targeting.targets.retain(|&dandelion, _| valid_dandelions.contains(&dandelion));

    for (rabbit_entity, mut rabbit_transform, mut rabbit) in rabbit_query.iter_mut() {
        rabbit.lifetime.tick(time.delta());

        // Find optimal dandelion target if no current target or target is invalid
        if rabbit.target.is_none() || dandelion_query.get(rabbit.target.unwrap()).is_err() {
            // Release any previous target claim
            if let Some(old_target) = rabbit.target {
                rabbit_targeting.release_target(old_target);
            }

            let new_target = find_best_dandelion_target(rabbit_entity, rabbit_transform.translation.truncate(), &dandelion_query, &rabbit_targeting);

            // Claim the new target
            if let Some(target_entity) = new_target {
                rabbit_targeting.claim_target(rabbit_entity, target_entity);
            }

            rabbit.target = new_target;
        }

        // Move towards target
        if let Some(target_entity) = rabbit.target {
            if let Ok((_, target_transform, target_dandelion)) = dandelion_query.get(target_entity) {
                let rabbit_pos = rabbit_transform.translation.truncate();
                let target_pos = target_transform.translation.truncate();
                let direction = (target_pos - rabbit_pos).normalize_or_zero();

                // Move towards target
                let movement = direction * rabbit.speed * time.delta_secs();
                rabbit_transform.translation += movement.extend(0.0);

                // Check if rabbit reached the dandelion
                let distance = rabbit_pos.distance(target_pos);
                if distance <= 25.0 {
                    // Close enough to eat
                    // Release the target claim
                    rabbit_targeting.release_target(target_entity);

                    // Remove the dandelion
                    commands.entity(target_entity).despawn();

                    // Update tracking - use actual dandelion size
                    area_tracker.total_area -= target_dandelion.size.visual_area();
                    game_data.add_dandelion_kill();
                    game_data.dandelion_count = game_data.dandelion_count.saturating_sub(1);

                    rabbit.dandelions_eaten += 1;
                    rabbit.target = None; // Look for new target

                    info!(
                        "Rabbit ate a {} dandelion! Total eaten: {}",
                        match target_dandelion.size {
                            crate::enemies::DandelionSize::Tiny => "tiny",
                            crate::enemies::DandelionSize::Small => "small",
                            crate::enemies::DandelionSize::Medium => "medium",
                            crate::enemies::DandelionSize::Large => "large",
                            crate::enemies::DandelionSize::Huge => "huge",
                        },
                        rabbit.dandelions_eaten
                    );

                    // If rabbit ate 2 dandelions, spawn a new rabbit
                    if rabbit.dandelions_eaten >= 2 {
                        let spawn_pos = rabbit_transform.translation.truncate();
                        spawn_rabbits(&mut commands, &asset_server, spawn_pos);
                        info!("Rabbit spawned a new rabbit after eating 2 dandelions!");

                        // Clear all targets for this rabbit before removing it
                        rabbit_targeting.clear_rabbit_targets(rabbit_entity);

                        // Remove this rabbit after spawning
                        commands.entity(rabbit_entity).despawn();
                        continue;
                    }
                }
            }
        }
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

    for (dandelion_entity, dandelion_transform, dandelion) in dandelion_query.iter() {
        // Skip if already being targeted by another rabbit
        if rabbit_targeting.is_targeted(dandelion_entity) && rabbit_targeting.get_targeting_rabbit(dandelion_entity) != Some(rabbit_entity) {
            continue;
        }

        let dandelion_pos = dandelion_transform.translation.truncate();
        let distance = rabbit_pos.distance(dandelion_pos);

        // Calculate score based on distance and dandelion size
        // Closer dandelions get higher scores, larger dandelions get bonus points
        let size_bonus = match dandelion.size {
            crate::enemies::DandelionSize::Tiny => 1.0,
            crate::enemies::DandelionSize::Small => 1.2,
            crate::enemies::DandelionSize::Medium => 1.5,
            crate::enemies::DandelionSize::Large => 2.0,
            crate::enemies::DandelionSize::Huge => 3.0,
        };

        // Score is inverse of distance with size bonus
        let score = (1000.0 / (distance + 1.0)) * size_bonus;

        if score > best_score {
            best_score = score;
            best_target = Some(dandelion_entity);
        }
    }

    best_target
}

/// Update fire ignition effects and damage dandelions
fn update_fire_ignition(
    mut commands: Commands,
    mut fire_query: Query<(Entity, &mut Transform, &mut FireIgnition, &mut Sprite), With<FireIgnition>>,
    dandelion_query: Query<(Entity, &Transform, &Dandelion), (With<Dandelion>, Without<FireIgnition>)>,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut game_data: ResMut<GameData>,
    mut area_tracker: ResMut<DandelionAreaTracker>,
) {
    for (_fire_entity, mut fire_transform, mut fire, mut sprite) in fire_query.iter_mut() {
        fire.damage_timer.tick(time.delta());
        fire.lifetime.tick(time.delta());

        // Expand the fire effect
        let lifetime_progress = fire.lifetime.elapsed_secs() / fire.lifetime.duration().as_secs_f32();
        let scale = 1.0 + lifetime_progress * 2.0;
        fire_transform.scale = Vec3::splat(scale);

        // Update fire radius based on scale
        fire.radius = 100.0 * scale;

        // Fade out over time
        let alpha = 0.8 * (1.0 - lifetime_progress);
        sprite.color.set_alpha(alpha);

        // Damage dandelions in radius
        if fire.damage_timer.just_finished() {
            let fire_pos = fire_transform.translation.truncate();
            let mut dandelions_to_ignite = Vec::new();

            for (dandelion_entity, dandelion_transform, dandelion) in dandelion_query.iter() {
                let dandelion_pos = dandelion_transform.translation.truncate();
                let distance = fire_pos.distance(dandelion_pos);

                if distance <= fire.radius {
                    dandelions_to_ignite.push((dandelion_entity, dandelion_pos, dandelion.size));
                }
            }

            // Remove ignited dandelions and potentially spawn chain reactions
            for (dandelion_entity, dandelion_pos, dandelion_size) in dandelions_to_ignite {
                // Remove the dandelion
                commands.entity(dandelion_entity).despawn();

                // Update tracking
                area_tracker.total_area -= dandelion_size.visual_area();
                game_data.add_dandelion_kill();
                game_data.dandelion_count = game_data.dandelion_count.saturating_sub(1);

                info!(
                    "Fire ignited a {:?} dandelion at ({:.1}, {:.1})",
                    dandelion_size, dandelion_pos.x, dandelion_pos.y
                );

                // Spawn new fire at dandelion location for chain reaction (smaller)
                let chain_fire = FireIgnition {
                    radius: 60.0,                                        // Smaller chain reaction
                    lifetime: Timer::from_seconds(1.0, TimerMode::Once), // Shorter duration
                    ..Default::default()
                };

                commands.spawn((
                    Sprite {
                        image: asset_server.load("seed.png"),
                        color: Color::srgba(1.0, 0.5, 0.1, 0.6),
                        ..default()
                    },
                    Transform::from_translation(Vec3::new(dandelion_pos.x, dandelion_pos.y, 12.0)).with_scale(Vec3::splat(0.8)),
                    chain_fire,
                    PowerupEntity,
                ));
            }
        }
    }
}

/// Clean up expired rabbits
fn cleanup_expired_rabbits(mut commands: Commands, rabbit_query: Query<(Entity, &Rabbit)>, mut rabbit_targeting: ResMut<RabbitTargeting>) {
    for (entity, rabbit) in rabbit_query.iter() {
        if rabbit.lifetime.just_finished() {
            // Clear all targets for this rabbit before removing it
            rabbit_targeting.clear_rabbit_targets(entity);
            commands.entity(entity).despawn();
            info!("Rabbit expired after 3 seconds");
        }
    }
}

/// Clean up expired fire effects
fn cleanup_expired_fire(mut commands: Commands, fire_query: Query<(Entity, &FireIgnition)>) {
    for (entity, fire) in fire_query.iter() {
        if fire.lifetime.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}

/// Cleanup powerup entities when exiting playing state
fn cleanup_powerups(mut commands: Commands, powerup_entities: Query<Entity, With<PowerupEntity>>) {
    commands.remove_resource::<PowerupSpawnTimer>();
    commands.remove_resource::<SelectedPowerup>();
    commands.remove_resource::<RabbitTargeting>();

    for entity in &powerup_entities {
        commands.entity(entity).despawn();
    }

    info!("Powerups cleaned up");
}
