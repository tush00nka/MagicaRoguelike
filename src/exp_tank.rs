use avian2d::{math::PI, prelude::*};
use bevy::prelude::*;

use crate::{
    exp_orb::{ExpOrb, ExpOrbDrop},
    mouse_position::MouseCoords,
    player::Player
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
    asset_server: Res<AssetServer>,
    mut collision_event_reader: EventReader<Collision>,
    player_query: Query<&Player>,
    tank_query: Query<(&Transform, &ExpTank, Entity)>,
) {
    for Collision(contacts) in collision_event_reader.read() {
        let tank_e: Option<Entity>;

        if tank_query.contains(contacts.entity2) && player_query.contains(contacts.entity1) {
            tank_e = Some(contacts.entity2);
        }
        else if tank_query.contains(contacts.entity1) && player_query.contains(contacts.entity2) {
            tank_e = Some(contacts.entity1);
        }
        else {
            tank_e = None;
        }

        for (tank_transform, tank, candidate_e) in tank_query.iter() {

            if tank_e.is_some() && tank_e.unwrap() == candidate_e {
                let offset = (2.0*PI)/tank.orbs as f32;

                for i in 0..tank.orbs {
    
                    // считаем точки, куда будем выбрасывать частицы опыта
                    let angle = offset * i as f32;
                    let direction = Vec2::from_angle(angle) * 16.0;
                    let destination = Vec3::new(tank_transform.translation.x + direction.x, tank_transform.translation.y + direction.y, tank_transform.translation.z);
    
                    commands.spawn(SpriteBundle {
                        sprite: Sprite {
                            color: Color::srgb(2.0, 2.0, 2.0),
                            ..default()
                        },
                        texture: asset_server.load("textures/exp_particle.png"),
                        transform: Transform::from_translation(tank_transform.translation),
                        ..default()
                    })
                    .insert(ExpOrb { exp: 5 })
                    .insert(ExpOrbDrop { drop_destination: destination });
                }

                commands.get_entity(tank_e.unwrap()).unwrap().despawn();
            }
        }
    }
}