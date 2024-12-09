//Бекон - дает дополнительную секунду к неуязвимости после урона за каждую копию
use bevy::prelude::*;

use crate::{
    item::{
        ItemPickedUpEvent,
        ItemType
    },
    player::PlayerStats
};

pub struct BaconPlugin;

impl Plugin for BaconPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, apply_effect);
    }
}

fn apply_effect(
    mut ev_item_picked_up: EventReader<ItemPickedUpEvent>,
    mut player_stats: ResMut<PlayerStats>,
) {
    for ev in ev_item_picked_up.read() {
        if ev.item_type == ItemType::Bacon {
            player_stats.invincibility_time += 1.;
        }   
    }
}