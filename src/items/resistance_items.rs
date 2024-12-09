//Предметы дающие дополнительное сопротивление к элементам
use bevy::prelude::*;

use crate::{
    elements::{ElementResistance, ElementType},
    item::{
        ItemPickedUpEvent,
        ItemType
    }, player::Player
};

pub struct ResistanceItemsPlugin;

impl Plugin for ResistanceItemsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, apply_effect);
    }
}

fn apply_effect(
    mut ev_item_picked_up: EventReader<ItemPickedUpEvent>,
    mut player_query: Query<&mut ElementResistance, With<Player>>,
) {
    if let Ok(mut resistance) = player_query.get_single_mut() {
        for ev in ev_item_picked_up.read() {

            let element = 
                match ev.item_type {
                    ItemType::WispInAJar => Some(ElementType::Fire),
                    ItemType::WaterbendingScroll => Some(ElementType::Water),
                    ItemType::Mineral => Some(ElementType::Earth),
                    ItemType::Glider => Some(ElementType::Air),
                    _ => None,
                };

            if element.is_some() {
                resistance.add(element.unwrap(), 20);
            }   
        }
    } 
}