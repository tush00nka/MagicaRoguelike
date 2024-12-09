//Клык вампира - восстанавливает здоровье при убийстве врагов.
use bevy::prelude::*;

use crate::{
    health::Health,
    item::{
        ItemPickedUpEvent,
        ItemType
    }, 
    mobs::MobDeathEvent,
    player::{
        Player,
        PlayerStats
    }
};

pub struct VampireToothPlugin;

impl Plugin for VampireToothPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (heal_on_mob_death, apply_effect));
    }
}

fn apply_effect(
    mut ev_item_picked_up: EventReader<ItemPickedUpEvent>,
    mut player_stats: ResMut<PlayerStats>,
) {
    for ev in ev_item_picked_up.read() {
        if ev.item_type == ItemType::VampireTooth {
            player_stats.vampirism += 1;
        }   
    }
}

fn heal_on_mob_death(
    player_stats: Res<PlayerStats>,
    mut player_query: Query<&mut Health, With<Player>>,
    mut ev_mob_death: EventReader<MobDeathEvent>,
) {
    for _ev in ev_mob_death.read() {
        let Ok(mut player_health) = player_query.get_single_mut() else {
            return;
        };

        player_health.heal(player_stats.vampirism);
    }
}