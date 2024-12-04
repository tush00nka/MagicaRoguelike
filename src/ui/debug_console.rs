use bevy::prelude::*;
use bevy_simple_text_input::{
    TextInputBundle,
    TextInputPlugin,
    TextInputSubmitEvent,
    TextInputSystem
};

pub struct DebugConsolePlugin;

impl Plugin for DebugConsolePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(TextInputPlugin)
            .add_systems(Startup, spawn_console)
            .add_systems(Update, handle_commands.after(TextInputSystem));
    }
}

fn spawn_console(
    mut commands: Commands,
) {
    commands.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            align_items: AlignItems::End,
            justify_content: JustifyContent::Start,
            ..default()
        },
        ..default()
    })
    .with_children(|parent| {
        parent.spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Px(32.),
                    ..default()
                },
                background_color: BackgroundColor(Color::BLACK),
                ..default()
            },
            TextInputBundle::default().with_text_style(TextStyle {
                font_size: 24.,
                color: Color::WHITE,
                ..default()
            }),
        ));
    });
}

fn handle_commands(
    mut ev_input_submit: EventReader<TextInputSubmitEvent>,
) {
    for ev in ev_input_submit.read() {
        println!("{}", ev.value);
    }
}