use bevy::prelude::*;
use avian2d::prelude::*;

use crate::{health::Health, player::Player};

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
    mut player_hp_query: Query<(&CollidingEntities, &mut Health), With<Player>>,
) {
    let Ok((colliding_e, mut health)) = player_hp_query.get_single_mut() else {
        return;
    };

    for (tank_e, tank) in tank_query.iter() {
        if colliding_e.contains(&tank_e) && health.current < health.max {
            health.heal(tank.hp);
            commands.entity(tank_e).despawn();
        }
    }
}