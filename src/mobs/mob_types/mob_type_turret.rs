//bundle for turrets(can shoot)
use {avian2d::prelude::*, bevy::prelude::*, std::time::Duration};

use crate::{
    elements::{ElementResistance, ElementType},
    health::Health,
    mobs::mob::*,
    Bundle, Timer,
};

#[derive(Bundle)]
pub struct TurretBundle {
    mob_bundle: MobBundle,
    shoot_ability: AttackComponent,
    search_and_pursue: SearchAndPursue,
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
        Self {
            mob_bundle: MobBundle::jungle_turret(),
            shoot_ability: AttackComponent {
                range: 300.,
                attack_type: AttackType::Range,
                cooldown: Timer::new(Duration::from_millis(2000),TimerMode::Repeating),
                damage: 15,
                element: Some(ElementType::Earth),
                proj_type: Some(ProjectileType::Gatling),
                ..default()
            },
            search_and_pursue: SearchAndPursue::range_units(),
        }
    }

    pub fn earth_elemental() -> Self {
        Self {
            mob_bundle: MobBundle::earth_elemental(),
            shoot_ability: AttackComponent {
                range: 300.,
                attack_type: AttackType::Range,
                cooldown: Timer::new(Duration::from_millis(3500),TimerMode::Repeating),
                damage: 30,
                element: Some(ElementType::Earth),
                proj_type: Some(ProjectileType::Missile),                
                ..default()
            },
            search_and_pursue: SearchAndPursue::range_units(),
        }
    }
}
