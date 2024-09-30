use avian2d::prelude::Collision;
use bevy::prelude::*;
use crate::player::*;
pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<PlayerHPChanged>()
            .add_event::<DeathEvent>()
            .add_systems(FixedUpdate, (pick_up_health, death));
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

#[derive(Component)]
pub struct HealthTank{
    pub hp: i32,
}

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

fn pick_up_health(
    mut commands: Commands,
    tank_query: Query<(Entity, &HealthTank)>,
    mut player_hp_query: Query<(&Player, &mut Health)>,
    mut ev_hp_gained: EventWriter<PlayerHPChanged>,
    mut ev_collision: EventReader<Collision>,
) {
    for Collision(contacts) in ev_collision.read() {

        let tank_e: Option<Entity>;

        if tank_query.contains(contacts.entity2) && player_hp_query.contains(contacts.entity1) {
            tank_e = Some(contacts.entity2);
        }
        else if tank_query.contains(contacts.entity1) && player_hp_query.contains(contacts.entity2) {
            tank_e = Some(contacts.entity1);
        }
        else {
            tank_e = None;
        }

        for (candiate_e, tank) in tank_query.iter() {
            if tank_e.is_some() && tank_e.unwrap() == candiate_e {
                for (_player, mut health) in player_hp_query.iter_mut() {
                    health.heal(tank.hp);
                }
                ev_hp_gained.send(PlayerHPChanged);
                commands.entity(tank_e.unwrap()).despawn();
            }
        }

    }
}