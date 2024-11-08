use bevy::prelude::*;

use crate::{
    health::Health,
    item::{
        ItemPickedUpEvent,
        ItemType
    }, 
    player::{
        Player,
        PlayerStats
    },
    ui::ItemInventory
};

pub struct BlindRagePlugin;

impl Plugin for BlindRagePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (apply_effect, update_bonus));
    }
}

fn apply_effect(
    mut ev_item_picked_up: EventReader<ItemPickedUpEvent>,
    mut player_stats: ResMut<PlayerStats>,
) {
    for ev in ev_item_picked_up.read() {
        if ev.item_type == ItemType::BlindRage {
            player_stats.blind_rage_bonus += 1;
        }   
    }
}

fn update_bonus(
    mut player_stats: ResMut<PlayerStats>,
    health_query: Query<&Health, With<Player>>,
    item_inventoty: Res<ItemInventory>,
) {
    let Ok(health) = health_query.get_single() else {
        return;
    };

    let blind_rage_amount: u32;

    if item_inventoty.0.contains_key(&ItemType::BlindRage) {
        blind_rage_amount = *item_inventoty.0.get(&ItemType::BlindRage).unwrap() as u32;
    }
    else {
        blind_rage_amount = 0;
    }
    
    player_stats.blind_rage_bonus = blind_rage_amount * (health.max as f32 / health.current as f32).round() as u32;
}