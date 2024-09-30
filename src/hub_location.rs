use crate::{chapter::ChapterManager, gamemap::{Floor, Wall}, GameState};
use avian2d::prelude::*;
use bevy::prelude::*;
pub struct HubPlugin;

impl Plugin for HubPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<crate::level_completion::PortalEvent>()
            .add_systems(OnEnter(GameState::Hub), spawn_hub);
    }
}

fn spawn_hub(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut player_query: Query<&mut Transform, With<crate::player::Player>>,
    mut ev_spawn_portal: EventWriter<crate::level_completion::PortalEvent>,
    chapter_manager: Res<ChapterManager>,
) {
    let tile_size = 32.0;
    for x in 0..8 {
        for y in 0..8 {
            if x == 0 || x == 7 || y == 0 || y == 7  {
                let texture_path = {
                    if y > 0 {
                        if y == 7 && 0 < x && x < 7 {
                            format!("textures/t_wall_top_{}.png", chapter_manager.get_current_chapter())
                        }
                        else {
                            format!("textures/t_wall_{}.png", chapter_manager.get_current_chapter())
                        }
                    } else {
                        format!("textures/t_wall_{}.png", chapter_manager.get_current_chapter())
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
            else {
                commands
                    .spawn(SpriteBundle {
                        texture: asset_server.load(format!("textures/t_floor_{}.png", chapter_manager.get_current_chapter())),
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
        }
    }
    if let Ok(mut transform) = player_query.get_single_mut() {
        transform.translation = Vec3::new(32., 32., 1.0);
    }
    ev_spawn_portal.send(crate::level_completion::PortalEvent {
        pos: Vec3::new(6. * 32., 6. * 32., 1.0),
    });
}
