use bevy::prelude::*;

use crate::{
    elements::{
        ElementBar,
        ElementBarClear,
        ElementBarFilled,
        ElementType
    },
    experience::{
        ExpGained,
        PlayerExperience
    },
    GameState
};

pub struct ElementsUIPlugin;

impl Plugin for ElementsUIPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnExit(GameState::MainMenu), spawn_ui)
            .add_systems(Update, (update_ui, add_slots_from_lv));
    }
}

#[derive(Component)]
struct ElementSlot(usize);

#[derive(Component)]
pub struct ElementBarUI;

fn spawn_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(NodeBundle {
        style: Style {
            top: Val::Px(48.0),
            width: Val::Px(24.0*6.0),
            height: Val::Px(24.0),
            ..default()
        },
        ..default()
    })
    .insert(ElementBarUI)
    .with_children(|parent| {
        parent.spawn(ImageBundle {
            style: Style {
                width: Val::Px(48.0),
                height: Val::Px(48.0),
                ..default()
            },
            image: UiImage::new(asset_server.load("textures/empty_slot.png")),
            ..default()
        }).insert(ElementSlot(0));
    });
}

fn update_ui(
    mut slot_query: Query<(&mut UiImage, &ElementSlot)>,
    element_bar: Res<crate::elements::ElementBar>,
    mut ev_bar_filled: EventReader<ElementBarFilled>,
    mut ev_bar_clear: EventReader<ElementBarClear>,
    asset_server: Res<AssetServer>,
) {
    for ev in ev_bar_filled.read() {
        for (mut image, slot) in slot_query.iter_mut() {
            if element_bar.len() == slot.0 as u8 + 1 {
                match ev.0 {
                    ElementType::Fire => image.texture = asset_server.load("textures/fire_slot.png"),
                    ElementType::Water => image.texture = asset_server.load("textures/water_slot.png"),
                    ElementType::Earth => image.texture = asset_server.load("textures/earth_slot.png"),
                    ElementType::Air => image.texture = asset_server.load("textures/air_slot.png"),
                    _ => image.texture = asset_server.load("textures/empty_slot.png")
                }
            }
        }
    }

    for _ev in ev_bar_clear.read() {
        for (mut image, _slot) in slot_query.iter_mut() {
            image.texture = asset_server.load("textures/empty_slot.png");
        }
    }
}

fn add_slots_from_lv(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut ev_exp_gained: EventReader<ExpGained>,
    player_experience: Res<PlayerExperience>,
    mut element_bar_query: Query<Entity, With<ElementBarUI>>,
    mut element_bar: ResMut<ElementBar>,
) {
    if let Ok(bar_e) = element_bar_query.get_single_mut() {
        for _ev in ev_exp_gained.read() {
            if element_bar.max < player_experience.lv {
                commands.entity(bar_e).with_children(|parent| {
                    for i in element_bar.max..player_experience.lv {
                        parent.spawn(ImageBundle {
                            style: Style {
                                width: Val::Px(48.0),
                                height: Val::Px(48.0),
                                ..default()
                            },
                            image: UiImage::new(asset_server.load("textures/empty_slot.png")),
                            ..default()
                        }).insert(ElementSlot(i as usize));
                    }
                });

                element_bar.max = player_experience.lv;
            }
        }   
    }
}