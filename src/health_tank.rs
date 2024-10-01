use bevy::prelude::*;
use avian2d::prelude::*;

use crate::{health::{Health, PlayerHPChanged}, player::Player};

pub struct HealthTankPlugin;

impl Plugin for HealthTankPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<SpawnHealthTankEvent>()
            .add_systems(Update, (spawn_health_tank, pick_up_health));
    }
}

#[derive(Component)]
pub struct HealthTank{
    pub hp: i32,
}

#[derive(Event)]
pub struct SpawnHealthTankEvent {
    pub pos: Vec3,
    pub hp: i32, 
}

fn spawn_health_tank(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut ev_spawn_health_tank: EventReader<SpawnHealthTankEvent>,
) {
    for ev in ev_spawn_health_tank.read() {
        commands.spawn(SpriteBundle {
            texture: asset_server.load("textures/health_tank.png"),
            transform: Transform::from_translation(ev.pos),
            ..default()
        })
        .insert(Collider::circle(8.0,))
        .insert(Sensor)
        .insert(HealthTank { hp: ev.hp });
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