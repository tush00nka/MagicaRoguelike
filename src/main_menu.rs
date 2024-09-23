use bevy::prelude::*;

use crate::GameState;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::MainMenu), spawn_ui)
            .add_systems(Update, handle_buttons.run_if(in_state(GameState::MainMenu)))
            .add_systems(OnExit(GameState::MainMenu), despawn_ui);
    }
}

#[derive(Component)]
struct UI;

enum ButtonType {
    NewRun,
    Settings,
    Quit,
}

#[derive(Component)]
struct MainMenuButton(ButtonType);

impl MainMenuButton {
    const NEW_RUN: Self = Self(ButtonType::NewRun);
    const SETTINGS: Self = Self(ButtonType::Settings);
    const QUIT: Self = Self(ButtonType::Quit);
}

fn spawn_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(NodeBundle {
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
    .insert(UI)
    .with_children(|parent| {
        parent.spawn(ButtonBundle {
            style: Style {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::top(Val::Px(4.0)),
                ..default()
            },
            background_color: Color::WHITE.into(),
            ..default()
        })
        .insert(MainMenuButton::NEW_RUN)
        .with_children(|button| {
            button.spawn(TextBundle::from_section(
                "вот решил опять попробовать", 
                TextStyle {
                    font: asset_server.load("fonts/ebbe_bold.ttf"),
                    font_size: 16.0,
                    color: Color::BLACK,
                    ..default()
                },
            ));
        });

        parent.spawn(ButtonBundle {
            style: Style {
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
                "что я наделал", 
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

fn handle_buttons(
    mut game_state: ResMut<NextState<GameState>>,
    mut buttons_query: Query<(&Interaction, &MainMenuButton, &mut BackgroundColor), Changed<Interaction>>,
    mut app_exit_events: ResMut<Events<bevy::app::AppExit>>
) {
    for (interaction, button, mut color) in buttons_query.iter_mut() {
        match *interaction {
            Interaction::Hovered => {}, // добавить анимации
            Interaction::Pressed => {
                match button.0 {
                    ButtonType::NewRun => { game_state.set(GameState::InGame); },
                    ButtonType::Settings => { game_state.set(GameState::Settings); },
                    ButtonType::Quit => { app_exit_events.send(AppExit::Success); }
                }
            },
            Interaction::None => {}, // откатывать анимации
        }
    }
}

fn despawn_ui(
    mut commands: Commands,
    ui_query: Query<Entity, With<UI>>,
) {
    for e in ui_query.iter() {
        print!("SHIT");
        commands.entity(e).despawn_recursive();
    }
}