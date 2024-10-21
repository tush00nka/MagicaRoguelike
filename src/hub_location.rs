use crate::{chapter::ChapterManager, gamemap::{Floor, Wall, ROOM_SIZE, TILE_SIZE}, item::{ItemType, SpawnItemEvent}, GameState};
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
    mut ev_spawn_portal: EventWriter<crate::level_completion::PortalEvent>,
    mut ev_spawn_item: EventWriter<crate::item::SpawnItemEvent>,
    chapter_manager: Res<ChapterManager>,
) {
    let lower = ROOM_SIZE/2 - 4;
    let upper = ROOM_SIZE/2 + 4;

    for x in lower..=upper {
        for y in lower..=upper {
            if x == lower || x == upper || y == lower || y == upper  {
                let texture_path = {
                    if y > lower {
                        if y == upper && lower < x && x < upper {
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
                            TILE_SIZE * x as f32,
                            TILE_SIZE * y as f32,
                            0.0,
                        ),
                        ..default()
                    })
                    .insert(RigidBody::Static)
                    .insert(Collider::rectangle(TILE_SIZE - 0.01, TILE_SIZE - 0.01))
                    .insert(Wall);
            }
            else {
                commands
                    .spawn(SpriteBundle {
                        texture: asset_server.load(format!("textures/t_floor_{}.png", chapter_manager.get_current_chapter())),
                        transform: Transform::from_xyz(
                            TILE_SIZE * x as f32,
                            TILE_SIZE * y as f32,
                            0.0,
                        ),
                        ..default()
                    })
                    .insert(Floor);
            }
        }
    }

    for i in (lower+2..=upper-2).step_by(2) {
        let random_item: ItemType = rand::random();
        ev_spawn_item.send(SpawnItemEvent {
            pos: Vec3::new(i as f32 * TILE_SIZE, (upper - 3) as f32 * TILE_SIZE, 1.),
            item_type: random_item,
            texture_path: random_item.get_texture_path().to_string(),
        });
    }

    ev_spawn_portal.send(crate::level_completion::PortalEvent {
        pos: Vec3::new((upper - 1) as f32 * TILE_SIZE, (lower + 1) as f32 * TILE_SIZE, 1.0),
    });
}
