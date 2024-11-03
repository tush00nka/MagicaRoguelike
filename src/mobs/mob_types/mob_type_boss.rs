use {bevy::prelude::*, rand::Rng, std::time::Duration};

use crate::{elements::ElementType, mobs::mob::*, pathfinding::Pathfinder, Bundle, Timer, mobs::ProjectileType::Gatling};

#[derive(Bundle)]
pub struct BossBundle<T: Component> {
    pub mob_bundle: MobBundle,
    pub path_finder: Pathfinder,
    pub shoot_ability: ShootAbility,
    pub target: T,
}

impl BossBundle<RunawayRush> {
    pub fn koldun() -> Self {
        Self {
            mob_bundle: MobBundle::koldun(),
            path_finder: Pathfinder {
                path: vec![],
                update_path_timer: Timer::new(
                    Duration::from_millis(rand::thread_rng().gen_range(500..999)),
                    TimerMode::Repeating,
                ),
                speed: 4000.,
            },
            shoot_ability: ShootAbility{
                time_to_shoot: Timer::new(Duration::from_millis(rand::thread_rng().gen_range(1000..1500)), TimerMode::Repeating,),
                element: ElementType::Steam,
                proj_type: Gatling,
            },
            target: RunawayRush,
        }
    }
}
