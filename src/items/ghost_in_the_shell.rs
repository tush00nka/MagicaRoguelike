use bevy::prelude::*;

use crate::{
    player::Player,
    item::{
        ItemPickedUpEvent,
        ItemType,
    }
};

pub struct GhostInTheShellPlugin;

impl Plugin for GhostInTheShellPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, apply_effect);
    }
}

fn apply_effect(
    mut ev_item_picked_up: EventReader<ItemPickedUpEvent>,
    mut player_query: Query<&mut Player>,
) {
    for ev in ev_item_picked_up.read() {
        if ev.item_type == ItemType::GhostInTheShell {
            if let Ok(mut player) = player_query.get_single_mut() {
                player.projectile_deflect_chance += 0.1; // 5%
            } 
        }   
    }
}