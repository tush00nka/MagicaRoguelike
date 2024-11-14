//bundle for mages
//we have 2 mages now - water and fire.
use {avian2d::prelude::*, bevy::prelude::*, rand::Rng, std::time::Duration};

use crate::{
    elements::{ElementResistance, ElementType},
    health::Health,
    mobs::mob::*,
    Bundle, Timer,
};

#[derive(Bundle)]
pub struct MageBundle {
    pub mob_bundle: MobBundle,
    pub teleport_ability: Teleport,
    pub shoot_ability: ShootAbility,
}

impl MobBundle {
    pub fn fire_mage() -> Self {
        Self {
            resistance: ElementResistance {
                elements: vec![ElementType::Fire],
                resistance_percent: vec![80, 0, 0, 0, 0],
            },
            mob_type: MobType::FireMage,
            body_type: RigidBody::Static,
            health: Health::new(80),
            ..default()
        }
    }

    pub fn water_mage() -> Self {
        Self {
            resistance: ElementResistance {
                elements: vec![ElementType::Water],
                resistance_percent: vec![0, 80, 0, 0, 0],
            },
            mob_type: MobType::WaterMage,
            body_type: RigidBody::Static,
            health: Health::new(80),
            ..default()
        }
    }
}

impl MageBundle {
    pub fn fire_mage() -> Self {
        let timer: u64 = rand::thread_rng().gen_range(3000..5000);

        Self {
            mob_bundle: MobBundle::fire_mage(),
            teleport_ability: Teleport {
                amount_of_tiles: 4,
                place_to_teleport: vec![],
                time_to_teleport: Timer::new(Duration::from_millis(timer), TimerMode::Repeating),
            },
            shoot_ability: ShootAbility {
                time_to_shoot: Timer::new(Duration::from_millis(timer), TimerMode::Repeating),
                element: ElementType::Fire,
                proj_type: ProjectileType::Missile,
            },
        }
    }
    pub fn water_mage() -> Self {
        let timer: u64 = rand::thread_rng().gen_range(3000..5000);

        Self {
            mob_bundle: MobBundle::water_mage(),
            teleport_ability: Teleport {
                amount_of_tiles: 4,
                place_to_teleport: vec![],
                time_to_teleport: Timer::new(Duration::from_millis(timer), TimerMode::Repeating),
            },
            shoot_ability: ShootAbility {
                time_to_shoot: Timer::new(Duration::from_millis(timer), TimerMode::Repeating),
                element: ElementType::Water,
                proj_type: ProjectileType::Missile,
            },
        }
    }
}
