use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use bevy::utils::HashMap;

#[derive(PartialEq, Clone)]
pub enum TileType {
    Wall,
    Floor,
}
#[derive(PartialEq, Eq, Hash)]
pub struct RoomPoint {
    pub x: i32,
    pub y: i32,
}

pub type Room = HashMap<RoomPoint, TileType>;

pub fn room_generator(map_size: i32) -> Room {
    let mut our_room = Room::default();
    for x_coord in 0..=map_size {
        for y_coord in 0..=map_size {
            if x_coord == 0 || x_coord == map_size || y_coord == 0 || y_coord == map_size {
                our_room.insert(
                    RoomPoint {
                        x: x_coord,
                        y: y_coord,
                    },
                    TileType::Wall,
                );
            } else {
                our_room.insert(
                    RoomPoint {
                        x: x_coord,
                        y: y_coord,
                    },
                    TileType::Floor,
                );
            }
        }
    }
    our_room
}

#[derive(Component, Clone, Copy)]
struct Floor {}
#[derive(Component, Clone, Copy)]
struct Wall {}

pub struct GameMapPlugin;

impl Plugin for GameMapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_map);
    }
}


fn spawn_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    let komnata = room_generator(5);
    let tile_size = 32.0;

    for (toczka, tile_type) in komnata.iter() {
        match tile_type {
            TileType::Floor => {
                commands
                    .spawn(SpriteBundle {
                        texture: asset_server.load("textures/t_floor.png"),
                        transform: Transform::from_xyz(
                            tile_size * toczka.x as f32,
                            tile_size * toczka.y as f32,
                            0.0,
                        ),
                        ..default()
                    })
                    //  .insert(RigidBody::Fixed)
                    // .insert(Collider::cuboid(16.0, 16.0))
                    .insert(Floor {});
            }
            TileType::Wall => {
                commands
                    .spawn(SpriteBundle {
                        texture: asset_server.load("textures/t_wall.png"),
                        transform: Transform::from_xyz(
                            tile_size * toczka.x as f32,
                            tile_size * toczka.y as f32,
                            0.0,
                        ),
                        ..default()
                    })
                    .insert(RigidBody::Fixed)
                    .insert(Collider::cuboid(16.0, 16.0))
                    .insert(Wall {});
            }
        }
    }
}