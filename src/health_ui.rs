use bevy::prelude::*;
use crate::{player::*, GameState, health::*};
pub struct HealthUIPlugin;

impl Plugin for HealthUIPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<PlayerHPChanged>()
            .add_systems(OnExit(GameState::MainMenu), spawn_ui)
            .add_systems(Update, update_ui);
    }
}

#[derive(Component)]
pub struct HPBarUI;

#[derive(Component)]
struct HPBar;

fn spawn_ui(
    mut commands: Commands,
) {
    commands.spawn(ImageBundle { // фон полоски ХП
        image: UiImage::solid_color(Color::hsl(0.0, 1.0, 0.1)),
        style: Style {
            width: Val::Px(96.0*2.0),
            height: Val::Px(24.0),
            left: Val::Px(0.0),
            top: Val::Px(20.0),
            ..default()
        },
        ..default()
    })
    .insert(HPBarUI)
    .with_children(|parent| { // сама полоска ХП
        parent.spawn(ImageBundle {
            image: UiImage::solid_color(Color::hsl(0.0, 1.0, 0.4)),
            style: Style {
                width: Val::Percent(50.0),
                height: Val::Px(24.0),
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                ..default()
            },
            ..default()
            }).insert(HPBar); 
        }
    );
}

fn update_ui(
    mut bar_query: Query<&mut Style, With<HPBar>>, 
    player_hp_query: Query<&Health, With <Player>>,
    mut ev_hp_gained: EventReader<PlayerHPChanged>,
) {

    for _ev in ev_hp_gained.read() {
        if let Ok(mut style) = bar_query.get_single_mut() {
            for health in player_hp_query.iter() {
                let percent = (health.current as f32 / health.max as f32) * 100.0; 
                style.width = Val::Percent(percent);
            }
        }
    }
}