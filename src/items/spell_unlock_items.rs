//Предметы, которые открывают новые заклинания при подборе(если подобрать несколько таких, дают бонусы к характеристикам этих заклинаний)
use bevy::prelude::*;

use crate::{
    elements::{ElementType, Spell, SpellPool},
    item::{
        ItemPickedUpEvent,
        ItemType
    }, player::PlayerStats, save::{ Save, SaveHandle }
};

pub struct SpellUnlocksPlugin;

impl Plugin for SpellUnlocksPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, apply_effect);
    }
}

fn apply_effect(
    mut ev_item_picked_up: EventReader<ItemPickedUpEvent>,
    mut spell_pool: ResMut<SpellPool>,
    mut player_stats: ResMut<PlayerStats>,
    mut saves: ResMut<Assets<Save>>,
    save_handle: Res<SaveHandle>
) {
    for ev in ev_item_picked_up.read() {

        let (spell_to_unlock, spell_save_name) = 
            match ev.item_type {
                ItemType::FieryShard => (Some(Spell::FireElemental), "fire_elemental"),
                ItemType::Valve => (Some(Spell::Steam), "steam"),
                ItemType::ElementWheel => (Some(Spell::BlackHole), "black_hole"),
                ItemType::NotchedPickaxe => (Some(Spell::EarthElemental), "earth_elemental"),
                ItemType::Fan => (Some(Spell::AirElemental), "air_elemental"),
                ItemType::Shield => (Some(Spell::Shield), "shield"),
                ItemType::Blank => (Some(Spell::Blank), "blank"),
                ItemType::Aquarius => (Some(Spell::WaterElemental), "water_elemental"),
                _ => (None, ""),
            };

        if spell_to_unlock.is_some() {
            spell_pool.unlock(spell_to_unlock.unwrap());
        
            let save = saves.get_mut(save_handle.0.id()).unwrap();

            if !save.seen_spells.contains(&spell_save_name.to_string()) {
                save.seen_spells.push(spell_save_name.to_string());
            }

            if spell_to_unlock.unwrap() == Spell::Steam {
                player_stats.element_damage_percent[ElementType::Steam as usize] += 0.1;
            }
        }
    }
}