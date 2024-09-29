use crate::gamemap::{Floor, TileType, Wall};
use crate::GameState;
use avian2d::prelude::*;
use bevy::prelude::*;
pub struct HubPlugin;

impl Plugin for HubPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<crate::level_completion::PortalEvent>();
        app.insert_resource(HubMap::default())
            .add_systems(OnEnter(GameState::Hub), create_hub) //mb otdelnyi state pod eto
            .add_systems(OnEnter(GameState::Hub), spawn_hub.after(create_hub));
    }
}

#[derive(Resource, Default)]
struct HubMap {
    grid: Vec<Vec<crate::gamemap::TileType>>,
}

fn create_hub(mut hub_map: ResMut<HubMap>) {
    for i in 0..8 {
        hub_map.grid.push(Vec::new());
        for j in 0..8 {
            //randomniye chisla
            if i == 0 || i == 7 || j == 0 || j == 7 {
                hub_map.grid[i].push(crate::gamemap::TileType::Wall);
            } else {
                hub_map.grid[i].push(crate::gamemap::TileType::Floor);
            }
        }
    }
}
fn spawn_hub(
    hub_map: ResMut<HubMap>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut player_query: Query<&mut Transform, With<crate::player::Player>>,
    mut ev_spawn_portal: EventWriter<crate::level_completion::PortalEvent>
) {
    let room_height = 8;
    let room_width = 8;
    let grid = &hub_map.grid;
    let tile_size = 32.0;
    for x in 0..room_width {
        for y in 0..room_height {
            match grid[x as usize][y as usize] {
                TileType::Floor => {
                    commands
                        .spawn(SpriteBundle {
                            texture: asset_server.load("textures/t_floor.png"),
                            transform: Transform::from_xyz(
                                tile_size * x as f32,
                                tile_size * y as f32,
                                0.0,
                            ),
                            ..default()
                        })
                        //.insert(RigidBody::Fixed)
                        //.insert(Collider::cuboid(16.0, 16.0))
                        .insert(Floor {});
                }
                TileType::Wall => {
                    let texture_path = {
                        if y > 0 {
                            match grid[x as usize][y as usize - 1] {
                                TileType::Floor => "textures/t_wall_top.png",
                                _ => "textures/t_wall.png",
                            }
                        } else {
                            "textures/t_wall.png"
                        }
                    };

                    commands
                        .spawn(SpriteBundle {
                            texture: asset_server.load(texture_path),
                            transform: Transform::from_xyz(
                                tile_size * x as f32,
                                tile_size * y as f32,
                                0.0,
                            ),
                            ..default()
                        })
                        .insert(RigidBody::Static)
                        .insert(Collider::rectangle(31.9, 31.9))
                        .insert(Wall {});
                }
                _ => {}
            }
        }
    }
    if let Ok(mut transform) = player_query.get_single_mut() {
        *transform = Transform::from_xyz(1. * 32., 1. * 32., 1.0);
    }
    ev_spawn_portal.send(crate::level_completion::PortalEvent {
        pos: Vec3::new(6. * 32., 6. * 32., 1.0),
    });
}
