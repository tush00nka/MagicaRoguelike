use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    elements::{ElementResistance, ElementType},
    health::Health,
    mobs::{mob::*, Boss},
    Bundle, Timer,
};

#[derive(Bundle)]
pub struct BossBundle {
    pub mob_bundle: MobBundle,
    pub boss: Boss,
}

impl MobBundle {
    pub fn koldun() -> Self {
        Self {
            phys_bundle: PhysicalBundle {
                collider: Collider::circle(24.),
                ..default()
            },
            resistance: ElementResistance {
                elements: vec![
                    ElementType::Earth,
                    ElementType::Air,
                    ElementType::Fire,
                    ElementType::Water,
                ],
                resistance_percent: vec![20, 20, 20, 20, 20],
            },
            mob_type: (MobType::Koldun),
            mob: Mob::new(40),
            loot: MobLoot { orbs: 100 },
            body_type: RigidBody::Dynamic,
            health: Health::new(2000),
        }
    }
}

impl BossBundle {
    pub fn koldun() -> Self {
        Self {
            mob_bundle: MobBundle::koldun(),
            boss: Boss {
                attack_cooldown: Timer::from_seconds(5.0, TimerMode::Repeating),
            },
        }
    }
}
