//Амулет - дает дополнительный орб с опытом за каждую копию
use bevy::prelude::*;

use crate::{
    experience::PlayerExperience,
    item::{
        ItemPickedUpEvent,
        ItemType
    }
};

pub struct AmuletPlugin;

impl Plugin for AmuletPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, apply_effect);
    }
}

fn apply_effect(
    mut ev_item_picked_up: EventReader<ItemPickedUpEvent>,
    mut player_experience: ResMut<PlayerExperience>,
) {
    for ev in ev_item_picked_up.read() {
        if ev.item_type == ItemType::Amulet {
            player_experience.orb_bonus += 1;
        }   
    }
}