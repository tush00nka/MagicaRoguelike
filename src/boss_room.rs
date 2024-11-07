use crate::{
    gamemap::{Floor, Wall, ROOM_SIZE, TILE_SIZE},
    GameState,
};
use avian2d::prelude::*;
use bevy::prelude::*;
pub struct BossRoomPlugin;

impl Plugin for BossRoomPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::LoadingBoss), spawn_boss_room);
    }
}

pub fn spawn_boss_room(asset_server: Res<AssetServer>, mut commands: Commands) {
    let lower = ROOM_SIZE / 2 - 8;
    let upper = ROOM_SIZE / 2 + 8;
    commands.insert_resource(ClearColor(Color::srgb(69. / 255., 35. / 255., 13. / 255.)));

    for x in lower..=upper {
        for y in lower..=upper {
            if x == lower || x == upper || y == lower || y == upper {
                let texture_path = {
                    if y > lower {
                        if y == upper && lower < x && x < upper {
                            "textures/t_wall_top_hub.png"
                        } else {
                            "textures/t_wall_hub.png"
                        }
                    } else {
                        "textures/t_wall_hub.png"
                    }
                };

                commands
                    .spawn(SpriteBundle {
                        texture: asset_server.load(texture_path),
                        transform: Transform::from_xyz(
                            TILE_SIZE * x as f32,
                            TILE_SIZE * y as f32,
                            0.0,
                        ),
                        ..default()
                    })
                    .insert(RigidBody::Static)
                    .insert(Collider::rectangle(TILE_SIZE - 0.01, TILE_SIZE - 0.01))
                    .insert(Wall);
            } else {
                let floor = commands.spawn(SpriteBundle {
                        texture: asset_server.load("textures/t_floor_hub.png"),
                        transform: Transform::from_xyz(
                            TILE_SIZE * x as f32,
                            TILE_SIZE * y as f32,
                            0.0,
                        ),
                        ..default()
                    })
                    .insert(Floor)
                    .id();

                    if y == upper-1 {
                        commands.entity(floor).with_children(|parent| {
                            parent.spawn(SpriteBundle {
                                texture: asset_server.load("textures/t_shadow.png"),
                                transform: Transform::from_xyz(
                                    0.0, 
                                    0.0,
                                    0.1,
                                ),
                                ..default()
                            });
                        });
                    }
            }
        }
    }
}
