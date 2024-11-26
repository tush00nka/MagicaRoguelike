use std::time::Duration;

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    elements::{ElementResistance, ElementType},
    health::Health,
    mobs::{mob::*, BossAttackSystem},
    pathfinding::Pathfinder,
    Bundle, Timer,
};
#[derive(Component)]
pub struct FirstPhase;

#[derive(Component)]
pub struct SecondPhase;

#[derive(Component)]
pub struct ThirdPhase;

#[derive(Clone)]
pub struct SummonUnit {
    pub entity: Option<Entity>,
    pub mob_type: MobType,
}

#[derive(Component)]
pub struct SummonQueue {
    pub queue: Vec<SummonUnit>,
    pub amount_of_mobs: i32,
}

#[derive(Bundle)]
pub struct BossBundle {
    pub mob_bundle: MobBundle,
    pub pathfinder: Pathfinder,    //running away
    pub teleport_abilty: Teleport, //teleport in random place away from player
    pub summon_queue: SummonQueue, //wrap in like summon ability? to add for usual mobs
    pub boss_attacks: BossAttackSystem,
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
            health: Health::new(3000),
            hit_list: HitList::default(),
        }
    }
}

impl BossBundle {
    pub fn koldun() -> Self {
        Self {
            mob_bundle: MobBundle::koldun(),
            boss_attacks: BossAttackSystem {
                //4 tiers of attacks
                weight_array: vec![0; 12], //amount of attacks
                cooldown_array: vec![Timer::new(Duration::from_millis(7050), TimerMode::Once); 12],
                cooldown_between_attacks: Timer::new(
                    Duration::from_millis(3500),
                    TimerMode::Repeating,
                ),
                cooldown_mask: 0b0000111111111111, //bitmask for cooldown, use bitwise to get what you need, equal to 4095
            },
            pathfinder: Pathfinder::default(),
            teleport_abilty: Teleport {
                amount_of_tiles: 5,
                place_to_teleport: vec![],
                time_to_teleport: Timer::new(Duration::from_millis(5000), TimerMode::Repeating),
            },
            summon_queue: SummonQueue {
                queue: vec![
                    SummonUnit {
                        entity: None,
                        mob_type: MobType::Mossling
                    };
                    100
                ],
                amount_of_mobs: 0,
            },
        }
    }
}
