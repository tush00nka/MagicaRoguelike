use bevy::prelude::*;
use crate::{
    mob::MobDeathEvent,
    player::PlayerDeathEvent
};
pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, death);
    }
}

#[derive(Component)]
pub struct Health {
    pub max: i32,
    pub current: i32,
}

impl Health {
    pub fn heal(&mut self, value: i32) {
        if self.current + value >= self.max {
            self.current = self.max;
        }
        else {
            self.current += value;
        }
    }
    pub fn damage(&mut self, value: i32,) {
        self.current -= value;
    }
}

fn death(
    mut commands: Commands,
    mut ev_player_death: EventReader<PlayerDeathEvent>,
    mut ev_mob_death: EventReader<MobDeathEvent>,
) {
    for ev in ev_player_death.read(){
         commands.entity(ev.0).despawn();
    }

    for mob_ev in ev_mob_death.read() {
        commands.entity(mob_ev.entity).despawn();
    }
}