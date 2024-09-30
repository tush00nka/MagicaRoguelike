use bevy::prelude::*;

use crate::{experience::{ExpGained, PlayerExperience}, GameState};

pub struct ExperienceUIPlugin;

impl Plugin for ExperienceUIPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnExit(GameState::MainMenu), spawn_ui)
            .add_systems(Update, update_ui);
    }
}

#[derive(Component)]
pub struct ExpBarUI;

#[derive(Component)]
struct ExpBar;

fn spawn_ui(
    mut commands: Commands,
) {

    commands.spawn(ImageBundle { // фон полоски опыта
        image: UiImage::solid_color(Color::hsl(25.0, 1.0, 0.1)),
        style: Style {
            width: Val::Px(96.0*2.0),
            height: Val::Px(12.0),
            ..default()
        },
        ..default()
    })
    .insert(ExpBarUI)
    .with_children(|parent| { // сама полоска опыта
        parent.spawn(ImageBundle {
            image: UiImage::solid_color(Color::hsl(35.0, 1.0, 0.5)),
            style: Style {
                width: Val::Percent(0.0),
                height: Val::Px(12.0),
                ..default()
            },
            ..default()
            }).insert(ExpBar); 
        }
    );

}

fn update_ui(
    mut bar_query: Query<&mut Style, With<ExpBar>>, 
    player_exp: Res<PlayerExperience>,
    mut ev_exp_gained: EventReader<ExpGained>,
) {

    for _ev in ev_exp_gained.read() {
        if let Ok(mut style) = bar_query.get_single_mut() {
            let percent = (player_exp.current as f32 / player_exp.to_lv_up as f32) * 100.0; 
            style.width = Val::Percent(percent);
        }
    }
}