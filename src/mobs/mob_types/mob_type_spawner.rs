//bundle for spawning mobs(like necromancer)
use {bevy::prelude::TimerMode, rand::Rng, std::time::Duration};

use crate::{mobs::mob::*, pathfinding::Pathfinder, Bundle, Timer};

#[derive(Bundle)]
pub struct SpawnerBundle {
    mob_bundle: MobBundle,
    summoning_ability: Summoning,
    path_finder: Pathfinder,
    target: MobTarget,
}

impl SpawnerBundle {
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
            target: MobTarget::Corpse,
        }
    }
}
