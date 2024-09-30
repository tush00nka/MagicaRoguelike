use bevy::prelude::*;
use crate::player::*;
pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<PlayerHPChanged>()
            .add_event::<DeathEvent>()
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

#[derive(Event)]
pub struct PlayerHPChanged;

#[derive(Event)]
pub struct DeathEvent(pub Entity);

fn death(
    mut commands: Commands,
    mut ev_death: EventReader<DeathEvent>,
    mut ev_player_death: EventWriter<PlayerDeathEvent>,
    player_query: Query<Entity, With<Player>>,
) {
    let player_id = player_query.get_single().unwrap_or(Entity::PLACEHOLDER); 

    for ev in ev_death.read(){
        let dead_id = ev.0;
        if dead_id == player_id {
            ev_player_death.send(PlayerDeathEvent);
        }
        commands.entity(ev.0).despawn();
    }
}