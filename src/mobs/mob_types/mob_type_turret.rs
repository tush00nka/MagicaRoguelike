//bundle for turrets(can shoot)
use {bevy::prelude::*, rand::Rng, std::time::Duration};

use crate::{elements::ElementType, mobs::mob::*, Bundle, Timer};

#[derive(Bundle)]
pub struct TurretBundle {
    mob_bundle: MobBundle,
    shoot_ability: ShootAbility,
}

impl TurretBundle {
    pub fn jungle_turret() -> Self {
        let timer: u64 = rand::thread_rng().gen_range(1500..2000);
        
        Self {
            mob_bundle: MobBundle::turret(),
            shoot_ability: ShootAbility {
                time_to_shoot: Timer::new(Duration::from_millis(timer), TimerMode::Repeating),
                element: ElementType::Earth,
                proj_type: ProjectileType::Gatling,
            },
        }
    }
}
