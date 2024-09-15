use bevy::prelude::*;

use crate::elements::ElementType;

pub struct ElementsUiPlugin;

impl Plugin for ElementsUiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, spawn_ui)
            .add_systems(Update, update_ui);
    }
}

#[derive(Component)]
struct ElementSlot(usize);

fn spawn_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(NodeBundle {
        style: Style {
            width: Val::Px(24.0*2.0),
            height: Val::Px(24.0),
            ..default()
        },
        ..default()
    }).with_children(|parent| {
        for i in 0..2 {
            parent.spawn(ImageBundle {
                style: Style {
                    width: Val::Px(48.0),
                    height: Val::Px(48.0),
                    ..default()
                },
                image: UiImage::new(asset_server.load("textures/empty_slot.png")),
                ..default()
            }).insert(ElementSlot(i));
        }
    });
}

fn update_ui(
    mut slot_query: Query<(&mut UiImage, &ElementSlot)>,
    element_bar: Res<crate::elements::ElementBar>,
    asset_server: Res<AssetServer>,
) {
    for (mut image, slot) in slot_query.iter_mut() {
        if element_bar.bar.len() > slot.0 {
            match element_bar.bar[slot.0] {
                ElementType::Fire => image.texture = asset_server.load("textures/fire_slot.png"),
                ElementType::Water => image.texture = asset_server.load("textures/water_slot.png"),
                ElementType::Earth => image.texture = asset_server.load("textures/earth_slot.png"),
                ElementType::Air => image.texture = asset_server.load("textures/air_slot.png"),
            }
        }
        else {
            image.texture = asset_server.load("textures/empty_slot.png");
        } 
    }
}