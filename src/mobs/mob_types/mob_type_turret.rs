//bundle for turrets(can shoot)
use {avian2d::prelude::*, bevy::prelude::*, rand::Rng, std::time::Duration};

use crate::{
    elements::{ElementResistance, ElementType},
    health::Health,
    mobs::mob::*,
    Bundle, Timer,
};

#[derive(Bundle)]
pub struct TurretBundle {
    mob_bundle: MobBundle,
    shoot_ability: ShootAbility,
}

impl MobBundle {
    pub fn jungle_turret() -> Self {
        Self {
            resistance: ElementResistance {
                elements: vec![ElementType::Earth, ElementType::Water],
                resistance_percent: vec![0, 60, 60, 0, 0],
            },
            mob_type: MobType::JungleTurret,
            body_type: RigidBody::Static,
            health: Health::new(200),
            ..default()
        }
    }

    pub fn earth_elemental() -> Self {
        Self {
            resistance: ElementResistance {
                elements: vec![ElementType::Earth],
                resistance_percent: vec![0, 0, 80, 0, 0],
            },
            mob_type: MobType::EarthElemental,
            body_type: RigidBody::Kinematic,
            health: Health::new(80),
            ..default()
        }
    }
}

impl TurretBundle {
    pub fn jungle_turret() -> Self {
        let timer: u64 = rand::thread_rng().gen_range(1500..2000);

        Self {
            mob_bundle: MobBundle::jungle_turret(),
            shoot_ability: ShootAbility {
                time_to_shoot: Timer::new(Duration::from_millis(timer), TimerMode::Repeating),
                element: ElementType::Earth,
                proj_type: ProjectileType::Gatling,
            },
        }
    }

    pub fn earth_elemental() -> Self {
        let timer: u64 = rand::thread_rng().gen_range(3500..4500);

        Self {
            mob_bundle: MobBundle::earth_elemental(),
            shoot_ability: ShootAbility {
                time_to_shoot: Timer::new(Duration::from_millis(timer), TimerMode::Repeating),
                element: ElementType::Earth,
                proj_type: ProjectileType::Missile,
            },
        }
    }
}
