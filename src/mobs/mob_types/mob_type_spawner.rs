//bundle for spawning mobs(like necromancer)
use {avian2d::prelude::*, bevy::prelude::*, rand::Rng, std::time::Duration};

use crate::{
    elements::{ElementResistance, ElementType},
    health::Health,
    mobs::mob::*,
    pathfinding::Pathfinder,
    Bundle, Timer,
};

#[derive(Bundle)]

pub struct SpawnerBundle<T: Component> {
    mob_bundle: MobBundle,
    summoning_ability: Summoning,
    path_finder: Pathfinder,
    target: T,
}

impl MobBundle {
    pub fn necromancer() -> Self {
        Self {
            phys_bundle: PhysicalBundle {
                collider: Collider::circle(8.),
                ..default()
            },
            resistance: ElementResistance {
                elements: vec![ElementType::Earth],
                resistance_percent: vec![0, 0, 30, 0, 0],
            },
            mob_type: MobType::Necromancer,
            loot: MobLoot { orbs: 5 },
            health: Health::new(140),
            ..default()
        }
    }
}

impl SpawnerBundle<RunawayRush> {
    pub fn necromancer() -> Self {
        Self {
            mob_bundle: MobBundle::necromancer(),
            summoning_ability: Summoning {
                time_to_spawn: Timer::new(
                    Duration::from_millis(rand::thread_rng().gen_range(1000..2000)),
                    TimerMode::Repeating,
                ),
                is_static: false,
            },
            path_finder: Pathfinder {
                path: vec![],
                update_path_timer: Timer::new(
                    Duration::from_millis(rand::thread_rng().gen_range(500..999)),
                    TimerMode::Repeating,
                ),
                speed: 3800.,
            },
            target: RunawayRush,
        }
    }
}
