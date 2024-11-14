use std::collections::HashMap;

use bevy::prelude::*;

use crate::{
    item::{ItemPickedUpEvent, ItemType}, GameState
};

pub struct ItemUIPlugin;

impl Plugin for ItemUIPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<ItemInventory>()
            .add_event::<UpdateInventoryEvent>()
            .add_systems(OnExit(GameState::MainMenu), spawn_ui)
            .add_systems(Update, (add_item_to_inventory, update_ui));
    }
}

#[derive(Resource, Default)]
pub struct ItemInventory(pub HashMap<ItemType, i32>);

impl ItemInventory {
    fn add(&mut self, item: ItemType) {
        if self.0.contains_key(&item) {
            *self.0.get_mut(&item).unwrap() += 1;
        }
        else {
            self.0.insert(item, 1);
        }
    }

    pub fn remove(&mut self, item: ItemType) {
        if self.0.contains_key(&item) {
            *self.0.get_mut(&item).unwrap() -= 1;
        }

        if *self.0.get(&item).unwrap() <= 0 {
            self.0.remove(&item);
        }
    }
}

#[derive(Event)]
pub struct UpdateInventoryEvent;

#[derive(Component)]
pub struct ItemUI;

fn spawn_ui(
    mut commands: Commands,
) {
    commands.spawn(NodeBundle {
        style: Style {
            top: Val::Px(0.0),
            left: Val::Px(432.0),
            width: Val::Percent(100.0),
            height: Val::Px(32.0),
            ..default()
        },
        ..default()
    })
    .insert(ItemUI);
}

fn add_item_to_inventory(
    mut ev_item_picked_up: EventReader<ItemPickedUpEvent>,
    mut ev_update_inventory: EventWriter<UpdateInventoryEvent>,
    mut inventory: ResMut<ItemInventory>,
) { 
    for ev in ev_item_picked_up.read() {
        inventory.add(ev.item_type);
        ev_update_inventory.send(UpdateInventoryEvent);
    }
}

fn update_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    ui_query: Query<Entity, With<ItemUI>>,
    mut ev_update_inventory: EventReader<UpdateInventoryEvent>,
    inventory: Res<ItemInventory>,
) {
    for _ev in ev_update_inventory.read() {
        if let Ok(ui) = ui_query.get_single() {
            commands.entity(ui).despawn_recursive();
        }
    
        commands.spawn(NodeBundle {
            style: Style {
                top: Val::Px(0.0),
                left: Val::Px(432.0),
                width: Val::Percent(100.0),
                height: Val::Px(32.0),
                ..default()
            },
            ..default()
        })
        .insert(ItemUI)
        .with_children(|parent| {
            for (item_type, count) in inventory.0.iter() {
                parent.spawn(ImageBundle {
                    style: Style {
                        width: Val::Px(32.0),
                        height: Val::Px(32.0),
                        ..default()
                    },
                    image: UiImage::new(asset_server.load(item_type.get_texture_path().to_string())),
                    ..default()
                })
                .with_children(|icon| {
                    icon.spawn(TextBundle::from_section(
                        format!("{}", count),
                        TextStyle {
                            font: asset_server.load("fonts/ebbe_bold.ttf"),
                            font_size: 12.,
                            ..default() 
                        }));
                });
            }
        });
    }
}