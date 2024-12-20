use crate::{camera::YSort, chapter::ChapterManager, gamemap::{Floor, Wall, ROOM_SIZE, TILE_SIZE}, item::{ItemDatabase, ItemDatabaseHandle, ItemType, SpawnItemEvent}, GameState};
use avian2d::prelude::*;
use bevy::prelude::*;
pub struct HubPlugin;

impl Plugin for HubPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<crate::level_completion::PortalEvent>()
            .add_systems(OnEnter(GameState::Hub), spawn_hub)
            .add_systems(OnExit(GameState::Hub), leave_hub);
    }
}

fn spawn_hub(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut ev_spawn_portal: EventWriter<crate::level_completion::PortalEvent>,
    mut ev_spawn_item: EventWriter<crate::item::SpawnItemEvent>,

    item_database: Res<Assets<ItemDatabase>>,
    handle: Res<ItemDatabaseHandle>,
) {
    let lower = ROOM_SIZE/2 - 4;
    let upper = ROOM_SIZE/2 + 4;

    commands.insert_resource(ClearColor(Color::srgb(69./255., 35./255., 13./255.)));

    for x in lower..=upper {
        for y in lower..=upper {
            if x == lower || x == upper || y == lower || y == upper  {
                let texture_path = {
                    if y > lower {
                        if y == upper && lower < x && x < upper {
                            "textures/t_wall_top_hub.png"
                        }
                        else {
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
                    .insert(Wall)
                    .insert(YSort(16.0));
            }
            else {
                let floor = commands.spawn(SpriteBundle {
                        texture: asset_server.load("textures/t_floor_hub.png"),
                        transform: Transform::from_xyz(
                            TILE_SIZE * x as f32,
                            TILE_SIZE * y as f32,
                            -100.0,
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

    for i in (lower+2..=upper-2).step_by(2) {
        let random_item: ItemType = rand::random();

        let item_name: String = item_database.get(handle.0.id()).unwrap().items[random_item as usize]["name"].as_str().unwrap().to_string();
        let texture_name: String = item_database.get(handle.0.id()).unwrap().items[random_item as usize]["texture_name"].as_str().unwrap().to_string();
        let item_description: String = item_database.get(handle.0.id()).unwrap().items[random_item as usize]["description"].as_str().unwrap().to_string();

        let texture_path = format!("textures/items/{}", texture_name);

        ev_spawn_item.send(SpawnItemEvent {
            pos: Vec3::new(i as f32 * TILE_SIZE, (upper - 3) as f32 * TILE_SIZE, 1.),
            item_type: random_item,
            texture_path,
            item_name,
            item_description
        });
    }

    ev_spawn_portal.send(crate::level_completion::PortalEvent {
        pos: Vec3::new((upper - 1) as f32 * TILE_SIZE, (lower + 1) as f32 * TILE_SIZE, 1.0),
    });
}

fn leave_hub(
    mut commands: Commands,
    chapter_manager: Res<ChapterManager>,
) {
    commands.insert_resource(ClearColor(chapter_manager.get_current_color()));

}
