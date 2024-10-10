use bevy::prelude::*;

use crate::{
    item::{
        ItemPickedUpEvent,
        ItemType
    },
    player::Player
};

pub struct BaconPlugin;

impl Plugin for BaconPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, apply_effect);
    }
}

fn apply_effect(
    mut ev_item_picked_up: EventReader<ItemPickedUpEvent>,
    mut player_query: Query<&mut Player>,
) {
    if let Ok(mut player) = player_query.get_single_mut() {
        for ev in ev_item_picked_up.read() {
            if ev.item_type == ItemType::Bacon {
                println!("Bacon effect applied");
                player.invincibility_time += 1.;
            }   
        }
    } 
}