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
pub struct MainMenuUI;

#[allow(unused)]
pub enum ButtonType {
    NewRun,
    Settings,
    Quit,
    MainMenu
}

#[derive(Component)]
pub struct MainMenuButton(pub ButtonType);

#[allow(unused)]
impl MainMenuButton {
    pub const NEW_RUN: Self = Self(ButtonType::NewRun);
    pub const SETTINGS: Self = Self(ButtonType::Settings);
    pub const QUIT: Self = Self(ButtonType::Quit);
    pub const MAIN_MENU: Self = Self(ButtonType::MainMenu);
}

fn spawn_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(25.0),
            justify_self: JustifySelf::Start,
            align_self: AlignSelf::Start,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            ..default()            
        },
        ..default()
    })
    .insert(MainMenuUI)
    .with_children(|parent| {
        parent.spawn(ImageBundle {
            image: UiImage::new(asset_server.load("textures/main_menu_title.png")),
            style: Style {
                width: Val::Px(320.0),
                height: Val::Px(96.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::top(Val::Px(4.0)),
                ..default()
            },
            ..default()
        });
    });

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
    .insert(MainMenuUI)
    .with_children(|parent| {
        parent.spawn(ButtonBundle {
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

pub fn handle_buttons(
    mut game_state: ResMut<NextState<GameState>>,
    mut buttons_query: Query<(&Interaction, &MainMenuButton, &mut Style), Changed<Interaction>>,
    mut app_exit_events: ResMut<Events<bevy::app::AppExit>>,
) {
    for (interaction, button, mut style) in buttons_query.iter_mut() {
        match *interaction {
            Interaction::Hovered => {
                style.width = Val::Px(512.0 * 1.1);
                style.height = Val::Px(24.0 * 1.25);
            }, // добавить анимации
            Interaction::Pressed => {
                match button.0 {
                    ButtonType::NewRun => { game_state.set(GameState::Loading); }, // идём в загрузку
                    ButtonType::Settings => { game_state.set(GameState::Settings); }, // открываем настройки
                    ButtonType::Quit => { app_exit_events.send(AppExit::Success); }, // выходим из игры
                    ButtonType::MainMenu => { game_state.set(GameState::MainMenu); } 
                }
            },
            Interaction::None => {
                style.width = Val::Px(512.0);
                style.height = Val::Px(24.0);
            }, // откатывать анимации
        }
    }
}

fn despawn_ui(
    mut commands: Commands,
    ui_query: Query<Entity, With<MainMenuUI>>,
) {
    for e in ui_query.iter() { // удаляем главное меню
        commands.entity(e).despawn_recursive();
    }
}