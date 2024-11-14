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
pub struct MainMenuButton {
    pub button: ButtonType,
}

#[allow(unused)]
impl MainMenuButton {
    pub const NEW_RUN: Self = Self {
        button: ButtonType::NewRun,
    };
    pub const SETTINGS: Self = Self {
        button: ButtonType::Settings,
    };
    pub const QUIT: Self = Self {
        button: ButtonType::Quit,
    };
    pub const MAIN_MENU: Self = Self {
        button: ButtonType::MainMenu,
    };
}

fn spawn_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(ImageBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            ..default()
        },
        image: UiImage {
            texture: asset_server.load("textures/ui/main_menu_bg.png"),
            ..default()
        },
        ..default()
    })
    .insert(MainMenuUI)
    .with_children(|parent| {
    
        let slicer = TextureSlicer {
            border: BorderRect::square(16.0),
            center_scale_mode: SliceScaleMode::Stretch,
            sides_scale_mode: SliceScaleMode::Stretch,
            ..default()
        };
    
        parent.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(50.0),
                height: Val::Percent(50.0),
                justify_self: JustifySelf::Center,
                align_self: AlignSelf::Center,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()            
            },
            ..default()
        })
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

            parent.spawn(ButtonBundle {
                style: Style {
                    width: Val::Px(360.0),
                    height: Val::Px(32.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::top(Val::Px(4.0)),
                    ..default()
                },
                image: UiImage {
                    texture: asset_server.load("textures/ui/button.png"),
                    ..default()
                },
                ..default()
            })
            .insert(ImageScaleMode::Sliced(slicer.clone()))
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
                    width: Val::Px(360.0),
                    height: Val::Px(32.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::top(Val::Px(4.0)),
                    ..default()
                },
                image: UiImage {
                    texture: asset_server.load("textures/ui/button.png"),
                    ..default()
                },
                ..default()
            })
            .insert(ImageScaleMode::Sliced(slicer.clone()))
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
                style.height = Val::Px(48.0);
            }, // добавить анимации
            Interaction::Pressed => {
                match button.button {
                    ButtonType::NewRun => { game_state.set(GameState::Loading); }, // идём в загрузку
                    ButtonType::Settings => { game_state.set(GameState::Settings); }, // открываем настройки
                    ButtonType::Quit => { app_exit_events.send(AppExit::Success); }, // выходим из игры
                    ButtonType::MainMenu => { game_state.set(GameState::MainMenu); } 
                }
            },
            Interaction::None => {
                style.height = Val::Px(32.0);
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