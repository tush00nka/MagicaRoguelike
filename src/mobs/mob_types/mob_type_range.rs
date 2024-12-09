//bundle for range mobs
//ходят и стреляют
use {bevy::prelude::*, std::time::Duration};

use crate::{
    elements::{ElementResistance, ElementType},
    health::Health,
    mobs::mob::*,
    Bundle, Timer,
    pathfinding::Pathfinder,
};

#[derive(Bundle)]
pub struct RangeMobBundle {
    pub mob_bundle: MobBundle,
    pub attack_ability: AttackComponent,
    pub search_and_pursue: SearchAndPursue,
    path_finder: Pathfinder,
}

impl MobBundle {
    pub fn water_elemental() -> Self {
        Self {
            resistance: ElementResistance {
                elements: vec![ElementType::Water],
                resistance_percent: vec![0, 80, 0, 0, 0],
            },
            mob_type: MobType::WaterElemental,
            health: Health::new(120),
            ..default()
        }
    }
}

impl RangeMobBundle {
    pub fn water_elemental() -> Self {
        Self {
            mob_bundle: MobBundle::water_elemental(),
            attack_ability: AttackComponent {
                range: 300.,
                attack_type: AttackType::Range,
                cooldown: Timer::new(Duration::from_millis(3000), TimerMode::Repeating),
                damage: 30,
                element: Some(ElementType::Water),
                proj_type: Some(ProjectileType::Missile),
                ..default()
            },
            search_and_pursue: SearchAndPursue::range_units(),
            path_finder: Pathfinder::default(),
        }
    }
}
