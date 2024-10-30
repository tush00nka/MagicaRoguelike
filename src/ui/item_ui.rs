use bevy::prelude::*;

use crate::{
    item::ItemPickedUpEvent, GameState
};

pub struct ItemUIPlugin;

impl Plugin for ItemUIPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnExit(GameState::MainMenu), spawn_ui)
            .add_systems(Update, update_item_ui);
    }
}

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

fn update_item_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut ev_item_picked_up: EventReader<ItemPickedUpEvent>,
    ui_query: Query<Entity, With<ItemUI>>,
) { 
    for ev in ev_item_picked_up.read() {
        if let Ok(parent) = ui_query.get_single() {
            commands.entity(parent).with_children(|parent| {
                parent.spawn(ImageBundle {
                    style: Style {
                        width: Val::Px(32.0),
                        height: Val::Px(32.0),
                        ..default()
                    },
                    image: UiImage::new(asset_server.load(ev.texture_path.to_string())),
                    ..default()
                });
            });
        }
    }
}