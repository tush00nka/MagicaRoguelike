//Зелье скорости - ускоряет игрока за каждую копию
use bevy::prelude::*;

use crate::{
    item::{
        ItemPickedUpEvent,
        ItemType
    },
    player::PlayerStats
};

pub struct SpeedPotionPlugin;

impl Plugin for SpeedPotionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, apply_effect);
    }
}

fn apply_effect(
    mut ev_item_picked_up: EventReader<ItemPickedUpEvent>,
    mut player_stats: ResMut<PlayerStats>,
) {
    for ev in ev_item_picked_up.read() {
        if ev.item_type == ItemType::SpeedPotion {
            player_stats.speed += 1000.;
        }   
    }
}