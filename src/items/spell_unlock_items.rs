use bevy::prelude::*;

use crate::{
    elements::{Spell, SpellPool},
    item::{
        ItemPickedUpEvent,
        ItemType
    }
};

pub struct SpellUnlocksPlugin;

impl Plugin for SpellUnlocksPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, apply_effect);
    }
}

fn apply_effect(
    mut ev_item_picked_up: EventReader<ItemPickedUpEvent>,
    mut spell_pool: ResMut<SpellPool>
) {
    for ev in ev_item_picked_up.read() {

        let spell_to_unlock = 
            match ev.item_type {
                ItemType::FieryShard => Some(Spell::FireElemental),
                ItemType::Valve => Some(Spell::Steam),
                ItemType::ElementWheel => Some(Spell::BlackHole),
                ItemType::NotchedPickaxe => Some(Spell::EarthElemental),
                ItemType::Fan => Some(Spell::AirElemental),
                ItemType::Shield => Some(Spell::Shield),
                ItemType::Blank => Some(Spell::Blank),
                ItemType::Aquarius => Some(Spell::WaterElemental),
                _ => None,
            };

        if spell_to_unlock.is_some() {
            spell_pool.unlock(spell_to_unlock.unwrap());
        }
    }
}