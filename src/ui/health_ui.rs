use bevy::prelude::*;
use crate::{health::*, player::*, GameState};
pub struct HealthUIPlugin;

impl Plugin for HealthUIPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnExit(GameState::MainMenu), spawn_ui)
            .add_systems(Update, update_ui);
    }
}

#[derive(Component)]
pub struct HPBarUI;

#[derive(Component)]
struct HPBar;

#[derive(Component)]
struct HPText;

#[derive(Component)]
struct ExtraLivesText;


fn spawn_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(NodeBundle {
        style: Style {
            width: Val::Px(192.+32.),
            height: Val::Px(24.),
            left: Val::Px(0.),
            top: Val::Px(20.),
            align_items: AlignItems::Center,
            ..default()
        },
        ..default()
    })
    .insert(HPBarUI)
    .with_children(|parent| {
        parent.spawn(ImageBundle { // фон полоски ХП
            image: UiImage::solid_color(Color::hsl(0.0, 1.0, 0.1)),
            style: Style {
                width: Val::Px(192.),
                height: Val::Px(24.0),
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        })
        .with_children(
            |parent| { // сама полоска ХП
                parent.spawn(ImageBundle {
                    image: UiImage::solid_color(Color::hsl(0.0, 1.0, 0.4)),
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Px(24.0),
                        left: Val::Px(0.0),
                        top: Val::Px(0.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ..default()
                    })
                    .insert(HPBar); 
            }
        );

        parent.spawn(TextBundle {
            text: Text::from_section(
                "",
                TextStyle {
                    font: asset_server.load("fonts/ebbe_bold.ttf"),
                    font_size: 16.0,
                    color: Color::WHITE,
                    ..default()
                }),
            ..default()
        })
        .insert(ExtraLivesText);
    });

    commands.spawn(TextBundle {
        style: Style {
            width: Val::Px(192.),
            height: Val::Px(24.),
            left: Val::Px(0.),
            top: Val::Px(24.),
            ..default()
        },
        text: Text {
            sections: vec![TextSection {
                value: "100/100".to_string(),
                style: TextStyle {
                    font: asset_server.load("fonts/ebbe_bold.ttf"),
                    font_size: 16.0,
                    color: Color::WHITE,
                    ..default()
                }
            }],
            justify: JustifyText::Center,
            ..default()
        },
        ..default()
    })
    .insert(HPBarUI)
    .insert(HPText);
}

fn update_ui(
    mut bar_query: Query<&mut Style, With<HPBar>>, 
    mut text_query: Query<&mut Text, (With<HPText>, Without<ExtraLivesText>)>,
    mut extra_lives_query: Query<&mut Text, (With<ExtraLivesText>, Without<HPText>)>,
    player_hp_query: Query<&Health, (With<Player>, Changed<Health>)>,
) {
    if let Ok(health) = player_hp_query.get_single() {
        if let Ok(mut style) = bar_query.get_single_mut() {
            let percent = (health.current as f32 / health.max as f32) * 100.0; 
            style.width = Val::Percent(percent);
        }

        if let Ok(mut text) = text_query.get_single_mut() {
            text.sections[0].value = format!("{}/{}", health.current, health.max);
        }   

        if let Ok(mut text) = extra_lives_query.get_single_mut() {
            if health.extra_lives > 0 {
                text.sections[0].value = format!("x{}", health.extra_lives);
            }
            else {
                text.sections[0].value = "".to_string();
            }
        }
    }
}