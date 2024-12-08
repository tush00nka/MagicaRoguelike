use std::time::Duration;

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    elements::{ElementResistance, ElementType},
    health::Health,
    mobs::{mob::*, BossAttackSystem, BossAttackType},
    Bundle, Timer,
};
#[derive(Component)]
pub struct PhaseManager {
    pub current_phase: u8,
    pub max_phase: u8,
    pub phase_change_hp_multiplier: Vec<f32>,
}

#[derive(Clone)]
pub struct SummonUnit {
    pub entity: Option<Entity>,
    pub mob_type: MobType,
}

#[derive(Component, Clone)]
pub struct SummonQueue {
    pub queue: Vec<SummonUnit>,
    pub amount_of_mobs: u8,
    pub max_amount: u8,
}

#[derive(Component)]
pub struct NoSummons;
impl SummonQueue {
    pub fn push(&mut self, summon_unit: SummonUnit) {
        self.amount_of_mobs += 1;
        for i in (1..self.amount_of_mobs as usize).rev() {
            self.queue[i] = self.queue[i - 1].clone();
        }

        self.queue[0] = summon_unit;
    }

    pub fn pop(&mut self) -> SummonUnit {
        let index = self.amount_of_mobs - 1;
        self.amount_of_mobs -= 1;
        return self.queue[index as usize].clone();
    }

    pub fn is_overflowed(&mut self) -> bool {
        return self.amount_of_mobs >= self.max_amount;
    }

    pub fn shift(&mut self, index: usize) {
        let len = self.queue.len() - 1;

        for i in index..len {
            self.queue[i] = self.queue[i + 1].clone();
        }

        self.amount_of_mobs -= 1;
        self.queue[len] = SummonUnit {
            entity: None,
            mob_type: MobType::Mossling,
        };

        self.clone().print();
        println!("amount of mobs in queue: {}", self.amount_of_mobs.clone());
    }

    pub fn resize(&mut self, size: u8) {
        self.max_amount = size;
    }

    pub fn print(self) {
        println!("");
        println!("");
        println!("");
        println!("");

        for i in self.queue {
            println!("{}", i.mob_type as u32);
        }

        println!("");
        println!("");
        println!("");
        println!("");
    }
}

#[derive(Event)]
pub struct PushMobQueueEvent {
    pub owner: Entity,
    pub mob_type: MobType,
    pub mob_e: Entity,
}

#[derive(Component)]
pub struct BossMovement {
    pub speed: f32,
    pub timer: Timer,
    pub direction: Vec2,
}

#[derive(Bundle)]
pub struct BossBundle {
    pub mob_bundle: MobBundle,
    pub pathfinder: BossMovement,  //running away
    pub teleport_abilty: Teleport, //teleport in random place away from player
    pub summon_queue: SummonQueue, //wrap in like summon ability? to add for usual mobs
    pub boss_attacks: BossAttackSystem,
    pub phase_manager: PhaseManager,
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
            exp_loot: MobLoot { orbs: 100 },
            body_type: RigidBody::Dynamic,
            health: Health::new(4500),
            hit_list: HitList::default(),
            ..default()
        }
    }
}

impl BossBundle {
    pub fn koldun() -> Self {
        let mut cooldowns = vec![Timer::new(Duration::from_millis(12500), TimerMode::Repeating); 12];

        cooldowns[BossAttackType::MegaStan as usize] =
            Timer::new(Duration::from_millis(12400), TimerMode::Repeating);
      /*   cooldowns[BossAttackType::SpawnClayGolem as usize] =
            Timer::new(Duration::from_millis(9000), TimerMode::Repeating); */
/*        cooldowns[BossAttackType::SpawnAirElemental as usize] =
            Timer::new(Duration::from_millis(9100), TimerMode::Repeating);
        cooldowns[BossAttackType::ProjectilePattern as usize] =
            Timer::new(Duration::from_millis(9050), TimerMode::Repeating);
 */
        Self {
            mob_bundle: MobBundle::koldun(),
            boss_attacks: BossAttackSystem {
                //4 tiers of attacks
                weight_array: vec![0; 11], //amount of attacks
                cooldown_array: cooldowns,
                cooldown_between_attacks: Timer::new(
                    Duration::from_millis(3500),
                    TimerMode::Repeating,
                ),
                cooldown_mask: 0b0000011111111111, //bitmask for cooldown, use bitwise to get what you need, equal to 4095
            },
            pathfinder: BossMovement {
                speed: 2850.,
                timer: Timer::new(Duration::from_millis(4000), TimerMode::Repeating),
                direction: Vec2::ZERO,
            },
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
                    10
                ],
                amount_of_mobs: 0,
                max_amount: 10,
            },
            phase_manager: PhaseManager {
                current_phase: 3,
                max_phase: 3,
                phase_change_hp_multiplier: vec![0.5, 0.2],
            },
        }
    }
}
