use bevy::prelude::*;

use crate::{
    player::PlayerStats,
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
    mut player_stats: ResMut<PlayerStats>,
) {
    for ev in ev_item_picked_up.read() {
        if ev.item_type == ItemType::GhostInTheShell {
            player_stats.projectile_deflect_chance += 0.1; // 10%
        }   
    }
}