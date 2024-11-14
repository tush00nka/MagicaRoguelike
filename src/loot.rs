use bevy::prelude::*;
use rand::Rng;

use crate::{
    exp_tank::SpawnExpTankEvent,
    health_tank::SpawnHealthTankEvent,
    item::{
        ItemType,
        SpawnItemEvent
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

                ev_item.send(SpawnItemEvent {
                    pos,
                    item_type: item,
                    texture_path: item.get_texture_path().to_string(),
                    item_name: item.get_name().to_string(),
                    item_description: item.get_description().to_string()
                });
            }
            _ => {}
        }
    }
}