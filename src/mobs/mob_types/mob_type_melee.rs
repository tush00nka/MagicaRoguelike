//bundle for melee only mobs
use avian2d::prelude::*;
use bevy::prelude::*;
use std::time::Duration;

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

    pub fn clay_golem() -> Self {
        Self {
            phys_bundle: PhysicalBundle::default(),
            resistance: ElementResistance {
                elements: vec![ElementType::Fire, ElementType::Earth],
                resistance_percent: vec![40, 0, 40, 0, 0],
            },
            mob_type: MobType::ClayGolem,
            mob: Mob::new(20),
            health: Health::new(200),
            loot: MobLoot { orbs: 8 },
            ..default()
        }
    }
}

impl MeleeMobBundle<PlayerRush> {
    pub fn knight() -> Self {
        Self {
            mob_bundle: MobBundle::knight(),
            path_finder: Pathfinder::default(),
            behaviour: PlayerRush,
            attack: AttackComponent {
                damage: 25,
                ..default()
            },
        }
    }

    pub fn mossling() -> Self {
        Self {
            mob_bundle: MobBundle::mossling(),
            path_finder: Pathfinder::default(),
            behaviour: PlayerRush,
            attack: AttackComponent {
                damage: 25,
                ..default()
            },
        }
    }

    pub fn clay_golem() -> Self {
        Self {
            mob_bundle: MobBundle::clay_golem(),
            path_finder: Pathfinder {
                speed: 1500.,
                ..default()
            },
            behaviour: PlayerRush,
            attack: AttackComponent {
                range: 30.,
                attack_type: AttackType::Circle,
                cooldown: Timer::new(Duration::from_millis(3000), TimerMode::Repeating),
                damage: 50,
                element: Some(ElementType::Earth),
                ..default()
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
                element: Some(ElementType::Fire),
                damage: 25,
                ..default()
            },
        }
    }
}
