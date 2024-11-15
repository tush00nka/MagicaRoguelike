use bevy::prelude::*;
use rand::Rng;

use crate::{
    exp_tank::SpawnExpTankEvent,
    health_tank::SpawnHealthTankEvent,
    item::{
        ItemDatabase, ItemDatabaseHandle, ItemType, SpawnItemEvent
    },
    mobs::MobDeathEvent
};

pub struct LootPlugin;

impl Plugin for LootPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<MobDeathEvent>()
            .add_systems(Update, loot_drop);
    }
}

fn loot_drop(
    mut ev_mob_death: EventReader<MobDeathEvent>,
    mut ev_health_tank: EventWriter<SpawnHealthTankEvent>,
    mut ev_exp_tank: EventWriter<SpawnExpTankEvent>,
    mut ev_item: EventWriter<SpawnItemEvent>,

    item_database: Res<Assets<ItemDatabase>>,
    handle: Res<ItemDatabaseHandle>,
){
    let mut rng = rand::thread_rng();

    for ev in ev_mob_death.read() {
        let pos = Vec3::new(ev.pos.x, ev.pos.y, 1.);
        match rng.gen_range(0..255) {
            0..=63 => { 
                ev_health_tank.send(SpawnHealthTankEvent {
                    pos,
                    hp: 15
                }); 
            }
            64..=95 => { 
                ev_exp_tank.send(SpawnExpTankEvent {
                    pos,
                    orbs: 6
                });
            }
            96..=112 => {
                let item: ItemType = rand::random();

                let item_name: String = item_database.get(handle.0.id()).unwrap().items[item as usize]["name"].as_str().unwrap().to_string();
                let texture_name: String = item_database.get(handle.0.id()).unwrap().items[item as usize]["texture_name"].as_str().unwrap().to_string();
                let item_description: String = item_database.get(handle.0.id()).unwrap().items[item as usize]["description"].as_str().unwrap().to_string();
        
                let texture_path = format!("textures/items/{}", texture_name);

                ev_item.send(SpawnItemEvent {
                    pos,
                    item_type: item,
                    texture_path,
                    item_name,
                    item_description
                });
            }
            _ => {}
        }
    }
}