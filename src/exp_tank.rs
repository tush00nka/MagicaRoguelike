use avian2d::{math::PI, prelude::*};
use bevy::prelude::*;

use crate::{exp_orb::{ExpOrb, ExpOrbDrop}, mouse_position::MouseCoords, player::Player};

pub struct ExpTankPlugin;

impl Plugin for ExpTankPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (debug_tank, break_tank));
    }
}

#[derive(Component)]
struct ExpTank {
    orbs: u32,
}

fn debug_tank(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mouse_coords: Res<MouseCoords>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::KeyT) {
        commands.spawn(SpriteBundle {
            texture: asset_server.load("textures/exp_tank.png"),
            transform: Transform::from_xyz((mouse_coords.0.x / 32.0).round() * 32.0, (mouse_coords.0.y / 32.0).round() * 32.0, 2.0),
            ..default()
        })
        .insert(Collider::rectangle(24.0, 24.0))
        .insert(Sensor)
        .insert(ExpTank { orbs: 6 });
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
        if tank_query.contains(contacts.entity2) && player_query.contains(contacts.entity1) {
            for (tank_transform, tank, tank_e) in tank_query.iter() {
                if contacts.entity2 == tank_e {

                    let offset = (2.0*PI)/tank.orbs as f32;

                    for i in 0..tank.orbs {

                        // считаем точки, куда будем выбрасывать частицы опыта
                        let angle = offset * i as f32;
                        let direction = Vec2::from_angle(angle) * 32.0;
                        let destination = Vec3::new(tank_transform.translation.x + direction.x, tank_transform.translation.y + direction.y, tank_transform.translation.z);

                        commands.spawn(SpriteBundle {
                            texture: asset_server.load("textures/exp_particle.png"),
                            transform: Transform::from_translation(tank_transform.translation),
                            ..default()
                        })
                        .insert(ExpOrb { exp: 5 })
                        .insert(ExpOrbDrop { drop_destination: destination });
                    }
                }
            }
            commands.get_entity(contacts.entity2).unwrap().despawn();
        }
    }
}