use crate::{chapter::ChapterManager, gamemap::{Floor, Wall}, item::{ItemType, SpawnItemEvent}, GameState};
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
    mut ev_spawn_item: EventWriter<crate::item::SpawnItemEvent>,
    chapter_manager: Res<ChapterManager>,
) {
    let tile_size = 32.0;
    for x in 0..=8 {
        for y in 0..=8 {
            if x == 0 || x == 8 || y == 0 || y == 8  {
                let texture_path = {
                    if y > 0 {
                        if y == 8 && 0 < x && x < 8 {
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
                    .insert(Wall);
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
                    .insert(Floor);
            }
        }
    }

    for i in (2..=6).step_by(2) {
        let random_item: ItemType = rand::random();
        ev_spawn_item.send(SpawnItemEvent {
            pos: Vec3::new(i as f32 * 32., 5. * 32., 1.),
            item_type: random_item,
            texture_path: random_item.get_texture_path().to_string(),
        });
    }

    if let Ok(mut transform) = player_query.get_single_mut() {
        transform.translation = Vec3::new(32., 32., 1.0);
    }

    ev_spawn_portal.send(crate::level_completion::PortalEvent {
        pos: Vec3::new(7. * 32., 1. * 32., 1.0),
    });
}
