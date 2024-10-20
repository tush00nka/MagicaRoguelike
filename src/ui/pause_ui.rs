use bevy::prelude::*;

use crate::TimeState;

pub struct PauseUIPlguin;

impl Plugin for PauseUIPlguin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, init_pause_menu)
            .add_systems(OnEnter(TimeState::Paused), show_pause_menu)
            .add_systems(OnEnter(TimeState::Unpaused), hide_pause_menu);
    }
}

#[derive(Component)]
pub struct PauseMenu;

fn init_pause_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.),
            height: Val::Percent(100.), 
            justify_content: JustifyContent::Center,
            align_content: AlignContent::Center,
            ..default()
        },
        background_color: BackgroundColor(Color::hsla(0., 0., 0., 0.5)),
        ..default()
    })
    .with_children(|parent| {
        parent.spawn(TextBundle {
            style: Style {
                height: Val::Percent(25.), 
                justify_self: JustifySelf::Center,
                align_self: AlignSelf::Center,
                ..default()
            },
            text: Text::from_section("да мам, я не играю...", TextStyle {
                font: asset_server.load("fonts/ebbe_bold.ttf"),
                font_size: 24.,
                color: Color::WHITE,
            }),
            ..default()
        });
    })
    .insert(PauseMenu)
    .insert(Visibility::Hidden);
}

fn hide_pause_menu(
    mut query: Query<&mut Visibility, With<PauseMenu>>, 
) {
    for mut v in query.iter_mut() {
        *v = Visibility::Hidden;
    }
}


fn show_pause_menu(
    mut query: Query<&mut Visibility, With<PauseMenu>>, 
) {
    for mut v in query.iter_mut() {
        *v = Visibility::Visible;
    }
}