use bevy::prelude::*;

use crate::{ui::*, utils::*, GameState};

pub struct GameOverPlugin;

impl Plugin for GameOverPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::GameOver), spawn_gameover_ui)
            .add_systems(Update, handle_buttons.run_if(in_state(GameState::GameOver)))
            .add_systems(OnEnter(GameState::MainMenu), despawn_gameover_ui)
            .add_systems(
                OnEnter(GameState::GameOver),
                (
                    despawn_all_with::<crate::exp_tank::ExpTank>,
                    despawn_all_with::<crate::health_tank::HealthTank>,
                    despawn_all_with::<crate::gamemap::Floor>,
                    despawn_all_with::<crate::gamemap::Wall>,
                    despawn_all_with::<crate::exp_orb::ExpOrb>,
                    despawn_all_with::<crate::shield_spell::Shield>,
                    despawn_all_with::<crate::blank_spell::Blank>,
                    despawn_all_with::<crate::black_hole::BlackHole>,
                    despawn_all_with::<crate::level_completion::Portal>,
                    despawn_all_with::<crate::mobs::Mob>,
                    despawn_all_with::<crate::wand::Wand>,
                    despawn_all_with::<crate::projectile::Projectile>,
                    despawn_all_with::<crate::shield_spell::Shield>,
                    despawn_all_with::<crate::item::Item>,
                    despawn_all_with::<crate::ui::ElementBarUI>,
                    despawn_all_with::<crate::ui::ExpBarUI>,
                    despawn_all_with::<crate::ui::HPBarUI>,
                    despawn_all_with::<crate::ui::ItemUI>,
                    despawn_all_with::<crate::obstacles::Obstacle>,
                ),
            );
    }
}

#[derive(Component)]
pub struct GameOverUI;

fn spawn_gameover_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Px(40.0),
                height: Val::Percent(50.0),
                justify_self: JustifySelf::Center,
                align_self: AlignSelf::Center,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        })
        .insert(GameOverUI)
        .with_children(|parent| {
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Px(512.0),
                        height: Val::Px(24.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::top(Val::Px(4.0)),
                        ..default()
                    },
                    background_color: Color::WHITE.into(),
                    ..default()
                })
                .insert(MainMenuButton::MAIN_MENU)
                .with_children(|button| {
                    button.spawn(TextBundle::from_section(
                        "в главное меню",
                        TextStyle {
                            font: asset_server.load("fonts/ebbe_bold.ttf"),
                            font_size: 16.0,
                            color: Color::BLACK,
                            ..default()
                        },
                    ));
                });

            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Px(512.0),
                        height: Val::Px(24.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::top(Val::Px(4.0)),
                        ..default()
                    },
                    background_color: Color::WHITE.into(),
                    ..default()
                })
                .insert(MainMenuButton::QUIT)
                .with_children(|button| {
                    button.spawn(TextBundle::from_section(
                        "что закибербуллили тебя, да? ну не знаю, выключи комьютер",
                        TextStyle {
                            font: asset_server.load("fonts/ebbe_bold.ttf"),
                            font_size: 16.0,
                            color: Color::BLACK,
                            ..default()
                        },
                    ));
                });
        });
}

fn despawn_gameover_ui(mut commands: Commands, ui_query: Query<Entity, With<GameOverUI>>) {
    for e in ui_query.iter() {
        // удаляем меню гейовера
        commands.entity(e).despawn_recursive();
    }
}
