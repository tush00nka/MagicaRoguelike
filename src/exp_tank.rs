use avian2d::{math::PI, prelude::*};
use bevy::prelude::*;

use crate::{
    exp_orb::SpawnExpOrbEvent,
    experience::PlayerExperience,
    mouse_position::MouseCoords,
    player::Player,
};

pub struct ExpTankPlugin;

impl Plugin for ExpTankPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<SpawnExpTankEvent>()
            .add_systems(Update, (spawn_tank, debug_tank, break_tank));
    }
}

#[derive(Component)]
pub struct ExpTank {
    pub orbs: u32,
}

#[derive(Event)]
pub struct SpawnExpTankEvent {
    pub pos: Vec3,
    pub orbs: u32,
}

fn spawn_tank(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut ev_spawn_exp_tank: EventReader<SpawnExpTankEvent>,
) {
    for ev in ev_spawn_exp_tank.read() {
        commands.spawn(SpriteBundle {
            texture: asset_server.load("textures/exp_tank.png"),
            transform: Transform::from_translation(ev.pos),
            ..default()
        })
        .insert(Collider::circle(8.0))
        .insert(Sensor)
        .insert(ExpTank { orbs: ev.orbs });
    }
}

fn debug_tank(
    mouse_coords: Res<MouseCoords>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut ev_spawn_exp_tank: EventWriter<SpawnExpTankEvent>,
) {
    if keyboard.just_pressed(KeyCode::KeyT) {
        ev_spawn_exp_tank.send(SpawnExpTankEvent {
            pos: Vec3::new(mouse_coords.0.x, mouse_coords.0.y, 2.),
            orbs: 6,
        });
    }
}

fn break_tank(
    mut commands: Commands,

    player_experience: Res<PlayerExperience>,

    player_query: Query<&CollidingEntities, With<Player>>,
    tank_query: Query<(Entity, &Transform, &ExpTank)>,

    mut ev_spawn: EventWriter<SpawnExpOrbEvent>,
) {
    let Ok(colliding_e) = player_query.get_single() else {
        return;
    };

    for (tank_e, tank_transform, tank) in tank_query.iter() {
        if colliding_e.contains(&tank_e) {

            let orbs_count = tank.orbs + player_experience.orb_bonus;
            let offset = (2.0*PI)/orbs_count as f32;

            for i in 0..orbs_count {
                // считаем точки, куда будем выбрасывать частицы опыта
                let angle = offset * i as f32;
                let direction = Vec2::from_angle(angle) * 16.0;
                let destination = Vec3::new(tank_transform.translation.x + direction.x,
                                                  tank_transform.translation.y + direction.y,
                                                    tank_transform.translation.z);

                ev_spawn.send(SpawnExpOrbEvent {
                    pos: tank_transform.translation,
                    destination,
                });
            }

            commands.entity(tank_e).despawn();
        }
    }
}