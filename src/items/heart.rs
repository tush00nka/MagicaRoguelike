use bevy::prelude::*;

use crate::{
    health::{
        Health,
        PlayerHPChanged
    },
    item::{
        ItemPickedUpEvent,
        ItemType
    }, 
    player::Player
};

pub struct HeartPlugin;

impl Plugin for HeartPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, apply_effect);
    }
}

fn apply_effect(
    mut ev_item_picked_up: EventReader<ItemPickedUpEvent>,
    mut player_query: Query<&mut Health, With<Player>>,
    mut ev_player_hp_changed: EventWriter<PlayerHPChanged>,
) {
    if let Ok(mut health) = player_query.get_single_mut() {
        for ev in ev_item_picked_up.read() {
            if ev.item_type == ItemType::Heart {
                println!("Heart effect applied");
                health.max += 10;
                ev_player_hp_changed.send(PlayerHPChanged);
            }   
        }
    } 
}