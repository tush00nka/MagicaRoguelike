use bevy::prelude::*;
use rand::Rng;

use crate::{
    TimeState,
    mob::MobDeathEvent,
    exp_tank::SpawnExpTankEvent,
    health_tank::SpawnHealthTankEvent,
};

pub struct LootPlugin;

impl Plugin for LootPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<MobDeathEvent>()
            .add_systems(Update, (loot_drop)
                .run_if(in_state(TimeState::Unpaused)));
    }
}

fn loot_drop(
    mut ev_mob_death: EventReader<MobDeathEvent>,
    mut ev_health_tank: EventWriter<SpawnHealthTankEvent>,
    mut ev_exp_tank: EventWriter<SpawnExpTankEvent>,
){
    let mut rng = rand::thread_rng();

    for ev in ev_mob_death.read() {
        let pos = Vec3::new(ev.pos.x, ev.pos.y, 1.);
        match rng.gen_range(0..255) {
            0..=63 => { ev_health_tank.send(SpawnHealthTankEvent {
                pos,
                hp: 15
            }); }
            64..95 => { ev_exp_tank.send(SpawnExpTankEvent {
                pos,
                orbs: 6
            }); }
            _ => {}
        }
    }
}