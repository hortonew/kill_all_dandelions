use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Level configuration and progression system
#[derive(Resource, Clone, Serialize, Deserialize)]
pub struct LevelData {
    pub current_level: u32,
    pub levels: Vec<Level>,
    pub level_progress: Vec<LevelProgress>,
}

impl Default for LevelData {
    fn default() -> Self {
        let levels = Self::create_default_levels();
        let level_count = levels.len();
        Self {
            current_level: 1,
            levels,
            level_progress: vec![LevelProgress::default(); level_count],
        }
    }
}

impl LevelData {
    /// Create the default set of levels with increasing difficulty
    fn create_default_levels() -> Vec<Level> {
        vec![
            Level {
                id: 1,
                name: "Weed Rising".to_string(),
                target_points: 500,
                time_limits: TimeLimits {
                    three_star: Duration::from_secs(60), // 1 minute
                    two_star: Duration::from_secs(90),   // 1.5 minutes
                    one_star: Duration::from_secs(120),  // 2 minutes
                },
                enemy_scaling: EnemyScaling {
                    health_multiplier: 1.0,
                    spawn_rate_multiplier: 1.0,
                    difficulty_threshold: 200,
                },
                unlock_requirements: UnlockRequirements {
                    required_level: 0,
                    required_stars: 0,
                },
            },
            Level {
                id: 2,
                name: "Golden Seed".to_string(),
                target_points: 800,
                time_limits: TimeLimits {
                    three_star: Duration::from_secs(90),
                    two_star: Duration::from_secs(120),
                    one_star: Duration::from_secs(180),
                },
                enemy_scaling: EnemyScaling {
                    health_multiplier: 1.2,
                    spawn_rate_multiplier: 1.1,
                    difficulty_threshold: 300,
                },
                unlock_requirements: UnlockRequirements {
                    required_level: 1,
                    required_stars: 1,
                },
            },
            Level {
                id: 3,
                name: "Morning Spore".to_string(),
                target_points: 1200,
                time_limits: TimeLimits {
                    three_star: Duration::from_secs(120),
                    two_star: Duration::from_secs(150),
                    one_star: Duration::from_secs(240),
                },
                enemy_scaling: EnemyScaling {
                    health_multiplier: 1.5,
                    spawn_rate_multiplier: 1.2,
                    difficulty_threshold: 400,
                },
                unlock_requirements: UnlockRequirements {
                    required_level: 2,
                    required_stars: 2,
                },
            },
            Level {
                id: 4,
                name: "Weedborn".to_string(),
                target_points: 1800,
                time_limits: TimeLimits {
                    three_star: Duration::from_secs(150),
                    two_star: Duration::from_secs(180),
                    one_star: Duration::from_secs(300),
                },
                enemy_scaling: EnemyScaling {
                    health_multiplier: 1.8,
                    spawn_rate_multiplier: 1.3,
                    difficulty_threshold: 500,
                },
                unlock_requirements: UnlockRequirements {
                    required_level: 3,
                    required_stars: 4,
                },
            },
            Level {
                id: 5,
                name: "Weed of Ascension".to_string(),
                target_points: 2500,
                time_limits: TimeLimits {
                    three_star: Duration::from_secs(180),
                    two_star: Duration::from_secs(240),
                    one_star: Duration::from_secs(360),
                },
                enemy_scaling: EnemyScaling {
                    health_multiplier: 2.2,
                    spawn_rate_multiplier: 1.4,
                    difficulty_threshold: 600,
                },
                unlock_requirements: UnlockRequirements {
                    required_level: 4,
                    required_stars: 6,
                },
            },
            Level {
                id: 6,
                name: "Hero of HOAges".to_string(),
                target_points: 3500,
                time_limits: TimeLimits {
                    three_star: Duration::from_secs(210),
                    two_star: Duration::from_secs(270),
                    one_star: Duration::from_secs(420),
                },
                enemy_scaling: EnemyScaling {
                    health_multiplier: 2.5,
                    spawn_rate_multiplier: 1.5,
                    difficulty_threshold: 700,
                },
                unlock_requirements: UnlockRequirements {
                    required_level: 5,
                    required_stars: 8,
                },
            },
            Level {
                id: 7,
                name: "The Weed of the Many".to_string(),
                target_points: 5000,
                time_limits: TimeLimits {
                    three_star: Duration::from_secs(240),
                    two_star: Duration::from_secs(300),
                    one_star: Duration::from_secs(480),
                },
                enemy_scaling: EnemyScaling {
                    health_multiplier: 3.0,
                    spawn_rate_multiplier: 1.6,
                    difficulty_threshold: 800,
                },
                unlock_requirements: UnlockRequirements {
                    required_level: 6,
                    required_stars: 10,
                },
            },
            Level {
                id: 8,
                name: "Dungeon Crawler Crabcrass".to_string(),
                target_points: 7500,
                time_limits: TimeLimits {
                    three_star: Duration::from_secs(300),
                    two_star: Duration::from_secs(360),
                    one_star: Duration::from_secs(600),
                },
                enemy_scaling: EnemyScaling {
                    health_multiplier: 3.5,
                    spawn_rate_multiplier: 1.8,
                    difficulty_threshold: 900,
                },
                unlock_requirements: UnlockRequirements {
                    required_level: 7,
                    required_stars: 12,
                },
            },
            Level {
                id: 9,
                name: "Thatch of the Emerald Lawn".to_string(),
                target_points: 10000,
                time_limits: TimeLimits {
                    three_star: Duration::from_secs(360),
                    two_star: Duration::from_secs(420),
                    one_star: Duration::from_secs(720),
                },
                enemy_scaling: EnemyScaling {
                    health_multiplier: 4.0,
                    spawn_rate_multiplier: 2.0,
                    difficulty_threshold: 1000,
                },
                unlock_requirements: UnlockRequirements {
                    required_level: 8,
                    required_stars: 15,
                },
            },
            Level {
                id: 10,
                name: "Moworrow and Moworrow and Moworrow".to_string(),
                target_points: 15000,
                time_limits: TimeLimits {
                    three_star: Duration::from_secs(420),
                    two_star: Duration::from_secs(480),
                    one_star: Duration::from_secs(840),
                },
                enemy_scaling: EnemyScaling {
                    health_multiplier: 5.0,
                    spawn_rate_multiplier: 2.5,
                    difficulty_threshold: 1200,
                },
                unlock_requirements: UnlockRequirements {
                    required_level: 9,
                    required_stars: 18,
                },
            },
            Level {
                id: 11,
                name: "Weed are Legion".to_string(),
                target_points: 20000,
                time_limits: TimeLimits {
                    three_star: Duration::from_secs(480),
                    two_star: Duration::from_secs(540),
                    one_star: Duration::from_secs(960),
                },
                enemy_scaling: EnemyScaling {
                    health_multiplier: 6.0,
                    spawn_rate_multiplier: 3.0,
                    difficulty_threshold: 1500,
                },
                unlock_requirements: UnlockRequirements {
                    required_level: 10,
                    required_stars: 20,
                },
            },
            Level {
                id: 12,
                name: "This is How You Lose the Weed War".to_string(),
                target_points: 30000,
                time_limits: TimeLimits {
                    three_star: Duration::from_secs(600),
                    two_star: Duration::from_secs(720),
                    one_star: Duration::from_secs(1200),
                },
                enemy_scaling: EnemyScaling {
                    health_multiplier: 7.0,
                    spawn_rate_multiplier: 3.5,
                    difficulty_threshold: 2000,
                },
                unlock_requirements: UnlockRequirements {
                    required_level: 11,
                    required_stars: 25,
                },
            },
        ]
    }

    /// Get the current level configuration
    pub fn get_current_level(&self) -> Option<&Level> {
        self.levels.iter().find(|l| l.id == self.current_level)
    }

    /// Get level by ID
    pub fn get_level(&self, level_id: u32) -> Option<&Level> {
        self.levels.iter().find(|l| l.id == level_id)
    }

    /// Get progress for a specific level
    pub fn get_level_progress(&self, level_id: u32) -> Option<&LevelProgress> {
        if level_id > 0 && level_id as usize <= self.level_progress.len() {
            Some(&self.level_progress[level_id as usize - 1])
        } else {
            None
        }
    }

    /// Get mutable progress for a specific level
    pub fn get_level_progress_mut(&mut self, level_id: u32) -> Option<&mut LevelProgress> {
        if level_id > 0 && level_id as usize <= self.level_progress.len() {
            Some(&mut self.level_progress[level_id as usize - 1])
        } else {
            None
        }
    }

    /// Check if a level is unlocked
    pub fn is_level_unlocked(&self, level_id: u32) -> bool {
        if let Some(level) = self.get_level(level_id) {
            if level.unlock_requirements.required_level == 0 {
                return true; // First level is always unlocked
            }

            // Check if previous level completed with required stars
            let total_stars: u32 = self.level_progress[0..level.unlock_requirements.required_level as usize]
                .iter()
                .map(|p| p.best_stars)
                .sum();

            total_stars >= level.unlock_requirements.required_stars
        } else {
            false
        }
    }

    /// Update level progress after completion
    pub fn complete_level(&mut self, level_id: u32, completion_time: Duration, final_score: u32) {
        // Get level data first to avoid borrow checker issues
        let time_limits = if let Some(level) = self.get_level(level_id) {
            level.time_limits.clone()
        } else {
            return;
        };

        if let Some(progress) = self.get_level_progress_mut(level_id) {
            let stars = calculate_stars(&time_limits, completion_time);

            // Update progress if this is a better result
            if final_score > progress.best_score || (final_score == progress.best_score && completion_time < progress.best_time) {
                progress.best_score = final_score;
                progress.best_time = completion_time;
                progress.best_stars = stars.max(progress.best_stars);
                progress.completed = true;
            }

            info!(
                "Level {} completed! Score: {}, Time: {:?}, Stars: {}",
                level_id, final_score, completion_time, stars
            );
        }
    }

    /// Get total stars earned across all levels
    pub fn get_total_stars(&self) -> u32 {
        self.level_progress.iter().map(|p| p.best_stars).sum()
    }

    /// Set the current level (for level selection)
    pub fn set_current_level(&mut self, level_id: u32) {
        if self.is_level_unlocked(level_id) {
            self.current_level = level_id;
        }
    }
}

/// Individual level configuration
#[derive(Clone, Serialize, Deserialize)]
pub struct Level {
    pub id: u32,
    pub name: String,
    pub target_points: u32,
    pub time_limits: TimeLimits,
    pub enemy_scaling: EnemyScaling,
    pub unlock_requirements: UnlockRequirements,
}

/// Time limits for star ratings
#[derive(Clone, Serialize, Deserialize)]
pub struct TimeLimits {
    pub three_star: Duration,
    pub two_star: Duration,
    pub one_star: Duration,
}

/// Enemy difficulty scaling for the level
#[derive(Clone, Serialize, Deserialize)]
pub struct EnemyScaling {
    pub health_multiplier: f32,
    pub spawn_rate_multiplier: f32,
    pub difficulty_threshold: u32, // Score when variety spawning begins
}

/// Requirements to unlock a level
#[derive(Clone, Serialize, Deserialize)]
pub struct UnlockRequirements {
    pub required_level: u32, // Previous level that must be completed
    pub required_stars: u32, // Total stars needed from previous levels
}

/// Player's progress on a specific level
#[derive(Clone, Serialize, Deserialize)]
pub struct LevelProgress {
    pub completed: bool,
    pub best_score: u32,
    pub best_time: Duration,
    pub best_stars: u32,
}

impl Default for LevelProgress {
    fn default() -> Self {
        Self {
            completed: false,
            best_score: 0,
            best_time: Duration::from_secs(999), // Very high default time
            best_stars: 0,
        }
    }
}

/// Calculate star rating based on completion time
pub fn calculate_stars(time_limits: &TimeLimits, completion_time: Duration) -> u32 {
    if completion_time <= time_limits.three_star {
        3
    } else if completion_time <= time_limits.two_star {
        2
    } else if completion_time <= time_limits.one_star {
        1
    } else {
        0 // Failed to complete in time
    }
}

/// Resource to track current level session
#[derive(Resource, Default)]
pub struct LevelSession {
    pub start_time: Option<Duration>,
    pub target_reached: bool,
}

impl LevelSession {
    pub fn start(&mut self, current_time: Duration) {
        self.start_time = Some(current_time);
        self.target_reached = false;
    }

    pub fn get_elapsed_time(&self, current_time: Duration) -> Option<Duration> {
        self.start_time.map(|start| current_time - start)
    }

    pub fn complete(&mut self) {
        self.target_reached = true;
    }
}

/// Events for level system
#[derive(Event)]
pub struct LevelCompleteEvent {
    pub level_id: u32,
    pub completion_time: Duration,
    pub final_score: u32,
    pub stars_earned: u32,
}

#[derive(Event)]
pub struct LevelStartEvent {
    pub level_id: u32,
}

#[derive(Event)]
pub struct LevelFailedEvent {
    pub level_id: u32,
    pub reason: FailureReason,
}

#[derive(Clone, Debug)]
pub enum FailureReason {
    TimeOut,
    PlayerQuit,
}

/// Plugin for the level system
pub struct LevelsPlugin;

impl Plugin for LevelsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LevelData>()
            .init_resource::<LevelSession>()
            .add_event::<LevelCompleteEvent>()
            .add_event::<LevelStartEvent>()
            .add_event::<LevelFailedEvent>()
            .add_systems(
                Update,
                (check_level_completion, handle_level_events).run_if(in_state(crate::GameState::Playing)),
            );
    }
}

/// Check if current level is completed based on score
fn check_level_completion(
    level_data: Res<LevelData>,
    game_data: Res<crate::playing::GameData>,
    mut level_session: ResMut<LevelSession>,
    mut level_complete_events: EventWriter<LevelCompleteEvent>,
    time: Res<Time>,
) {
    if let Some(current_level) = level_data.get_current_level() {
        if !level_session.target_reached && game_data.score >= current_level.target_points {
            level_session.complete();

            if let Some(completion_time) = level_session.get_elapsed_time(time.elapsed()) {
                let stars = calculate_stars(&current_level.time_limits, completion_time);

                level_complete_events.write(LevelCompleteEvent {
                    level_id: current_level.id,
                    completion_time,
                    final_score: game_data.score,
                    stars_earned: stars,
                });
            }
        }
    }
}

/// Handle level-related events
fn handle_level_events(
    mut level_complete_events: EventReader<LevelCompleteEvent>,
    mut level_start_events: EventReader<LevelStartEvent>,
    mut level_failed_events: EventReader<LevelFailedEvent>,
    mut level_data: ResMut<LevelData>,
    mut level_session: ResMut<LevelSession>,
    time: Res<Time>,
) {
    // Handle level completions
    for event in level_complete_events.read() {
        level_data.complete_level(event.level_id, event.completion_time, event.final_score);
        info!("Level {} completed with {} stars!", event.level_id, event.stars_earned);
    }

    // Handle level starts
    for event in level_start_events.read() {
        level_session.start(time.elapsed());
        info!("Level {} started", event.level_id);
    }

    // Handle level failures
    for event in level_failed_events.read() {
        info!("Level {} failed: {:?}", event.level_id, event.reason);
    }
}
