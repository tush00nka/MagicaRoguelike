use bevy::prelude::*;

use crate::{
    item::{
        ItemPickedUpEvent,
        ItemType
    },
    player::Player
};

pub struct SpeedPotionPlugin;

impl Plugin for SpeedPotionPlugin {
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
            if ev.item_type == ItemType::SpeedPotion {
                println!("Speed Potion effect applied");
                player.speed += 1000.;
            }   
        }
    } 
}