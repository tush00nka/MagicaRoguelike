use bevy::prelude::*;

use crate::{mobs::{mob::*, Boss}, Bundle, Timer};

#[derive(Bundle)]
pub struct BossBundle{
    pub mob_bundle: MobBundle,
    pub boss: Boss,
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
