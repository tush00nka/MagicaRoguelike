//bundle for melee only mobs

use {bevy::prelude::TimerMode, rand::Rng, std::time::Duration};

use crate::{mobs::mob::*, pathfinding::Pathfinder, Bundle, Timer};

#[derive(Bundle)]
pub struct MeleeMobBundle {
    mob_bundle: MobBundle,
    path_finder: Pathfinder,
    target: MobTarget,
}

impl MeleeMobBundle {
    pub fn knight() -> Self {
        Self {
            mob_bundle: MobBundle::knight(),
            path_finder: Pathfinder {
                path: vec![],
                update_path_timer: Timer::new(
                    Duration::from_millis(rand::thread_rng().gen_range(500..999)),
                    TimerMode::Repeating,
                ),
                speed: 2000.,
            },
            target: MobTarget::Player,
        }
    }

    pub fn mossling() -> Self {
        Self {
            mob_bundle: MobBundle::mossling(),
            path_finder: Pathfinder {
                path: vec![],
                update_path_timer: Timer::new(
                    Duration::from_millis(rand::thread_rng().gen_range(500..999)),
                    TimerMode::Repeating,
                ),
                speed: 2500.,
            },
            target: MobTarget::Player,
        }
    }
}
