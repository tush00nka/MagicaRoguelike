//bundle for melee only mobs
use avian2d::prelude::*;
use bevy::prelude::*;
use {rand::Rng, std::time::Duration};

use crate::{
    elements::{ElementResistance, ElementType},
    health::Health,
    mobs::mob::*,
    pathfinding::Pathfinder,
    GameLayer, Timer,
};

#[derive(Bundle)]
pub struct MeleeMobBundle<T: Component> {
    mob_bundle: MobBundle,
    path_finder: Pathfinder,
    behaviour: T,
    attack: AttackComponent,
}

impl MobBundle {
    pub fn knight() -> Self {
        Self {
            mob_type: MobType::Knight,
            ..default()
        }
    }
    pub fn mossling() -> Self {
        Self {
            resistance: ElementResistance {
                elements: vec![ElementType::Earth, ElementType::Water],
                resistance_percent: vec![0, 15, 15, 0, 0],
            },
            ..default()
        }
    }
    pub fn fire_elemental() -> Self {
        Self {
            phys_bundle: PhysicalBundle {
                collision_layers: CollisionLayers::new(
                    GameLayer::Enemy,
                    [
                        GameLayer::Projectile,
                        GameLayer::Friend,
                        GameLayer::Enemy,
                        GameLayer::Player,
                    ],
                ),
                ..default()
            },
            resistance: ElementResistance {
                elements: vec![ElementType::Fire],
                resistance_percent: vec![80, 0, 0, 0, 0],
            },
            mob_type: MobType::FireElemental,
            mob: Mob::new(30),
            health: Health::new(90),
            ..default()
        }
    }
}

impl MeleeMobBundle<PlayerRush> {
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
            behaviour: PlayerRush,
            attack: AttackComponent {
                range: 24.,
                attack_type: AttackType::Slash,
                target: None,
                cooldown: Timer::new(Duration::from_millis(2000), TimerMode::Repeating),
                attacked: false,
                element: None,
                damage: 25,
            },
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
            behaviour: PlayerRush,
            attack: AttackComponent {
                range: 24.,
                attack_type: AttackType::Slash,
                target: None,
                cooldown: Timer::new(Duration::from_millis(2000), TimerMode::Repeating),
                attacked: false,
                element: None,
                damage: 25,
            },
        }
    }
}

#[derive(Bundle)]
pub struct MeleePhasingBundle {
    mob_bundle: MobBundle,
    phasing: Phasing,
    attack: AttackComponent,
}
impl MeleePhasingBundle {
    pub fn fire_elemental() -> Self {
        Self {
            mob_bundle: MobBundle::fire_elemental(),
            phasing: Phasing { speed: 2500. },
            attack: AttackComponent {
                range: 24.,
                attack_type: AttackType::Slash,
                target: None,
                cooldown: Timer::new(Duration::from_millis(2000), TimerMode::Repeating),
                attacked: false,
                element: None,
                damage: 25,
            },
        }
    }
}
