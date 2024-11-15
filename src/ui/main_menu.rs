use bevy::prelude::*;
use serde::Deserialize;
use serde_json::{Map, Value};

use crate::{
    GameState,
    MainMenuState
};

use bevy_common_assets::json::JsonAssetPlugin;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(JsonAssetPlugin::<SpellBook>::new(&["json"]))
            .enable_state_scoped_entities::<MainMenuState>()
            .add_systems(Startup, load_spells)
            .add_systems(OnEnter(MainMenuState::Main), spawn_main_menu_ui)
            .add_systems(OnEnter(MainMenuState::AlmanachSelection), spawn_almanach_ui)
            .add_systems(OnEnter(MainMenuState::ViewSpells), spawn_view_spells)
            .add_systems(Update, handle_buttons)
            .add_systems(OnExit(GameState::MainMenu), despawn_ui);
    }
}

#[derive(Deserialize, Asset, TypePath)]
struct SpellBook {
    spells: Vec<Map<String, Value>>,
}

#[derive(Resource)]
struct SpellBookHandle(Handle<SpellBook>);

#[derive(Component)]
pub struct MainMenuUI;

#[allow(unused)]
pub enum ButtonType {
    NewRun,
    Quit,
    MainMenu,
    NaviagteMenu(MainMenuState)
}

#[derive(Component)]
pub struct MainMenuButton {
    pub button: ButtonType,
    pub height: f32,
}

#[allow(unused)]
impl MainMenuButton {
    pub const NEW_RUN: Self = Self {
        button: ButtonType::NewRun,
        height: 32.
    };
    pub const SETTINGS: Self = Self {
        button: ButtonType::NaviagteMenu(MainMenuState::Settings),
        height: 32.
    };
    pub const QUIT: Self = Self {
        button: ButtonType::Quit,
        height: 32.
    };
    pub const MAIN_MENU: Self = Self {
        button: ButtonType::MainMenu,
        height: 32.
    };
    pub const ALMANACH: Self = Self {
        button: ButtonType::NaviagteMenu(MainMenuState::AlmanachSelection),
        height: 32.
    };
    pub const VIEW_SPELLS: Self = Self {
        button: ButtonType::NaviagteMenu(MainMenuState::ViewSpells),
        height: 360.
    };
    pub const VIEW_ITEMS: Self = Self {
        button: ButtonType::NaviagteMenu(MainMenuState::ViewItems),
        height: 360.
    };
    pub const VIEW_MOBS: Self = Self {
        button: ButtonType::NaviagteMenu(MainMenuState::ViewMobs),
        height: 360.
    };
}

fn load_spells(
    mut commands: Commands, 
    asset_server: Res<AssetServer>
) {
    commands.insert_resource(SpellBookHandle(asset_server.load("spells.json")));
}

fn spawn_main_menu_ui (
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
    .insert(StateScoped(MainMenuState::Main))
    .with_children(|parent| {

        let slicer = TextureSlicer {
            border: BorderRect::square(16.0),
            center_scale_mode: SliceScaleMode::Stretch,
            sides_scale_mode: SliceScaleMode::Stretch,
            ..default()
        };

        let button = (
            ButtonBundle {
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
            },
            ImageScaleMode::Sliced(slicer.clone()),
        );

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
    
            parent.spawn((button.clone(), MainMenuButton::NEW_RUN)).with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    "вот решил опять попробовать", 
                    TextStyle {
                        font: asset_server.load("fonts/ebbe_bold.ttf"),
                        font_size: 16.0,
                        color: Color::BLACK,
                        ..default()
                    },
                ));
            });

            parent.spawn((button.clone(), MainMenuButton::ALMANACH)).with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    "книжка с картинками", 
                    TextStyle {
                        font: asset_server.load("fonts/ebbe_bold.ttf"),
                        font_size: 16.0,
                        color: Color::BLACK,
                        ..default()
                    },
                ));
            });

            parent.spawn((button.clone(), MainMenuButton::QUIT)).with_children(|parent| {
                parent.spawn(TextBundle::from_section(
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

fn spawn_almanach_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let canvas = commands.spawn(ImageBundle { 
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },  
        background_color: BackgroundColor(Color::srgb(69. / 255., 35. / 255., 13. / 255.)),
        ..default()
    })
    .insert(MainMenuUI)
    .insert(StateScoped(MainMenuState::AlmanachSelection))
    .id();

    let content = commands.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(50.0),
            justify_self: JustifySelf::Center,
            align_self: AlignSelf::Center,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: Val::Px(8.0),
            ..default()            
        },
        ..default()
    }).id();

    let slicer = TextureSlicer {
        border: BorderRect::square(16.0),
        center_scale_mode: SliceScaleMode::Stretch,
        sides_scale_mode: SliceScaleMode::Stretch,
        ..default()
    };

    let button = (
        ButtonBundle {
            style: Style {
                width: Val::Px(320.0),
                height: Val::Px(360.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            image: UiImage::new(asset_server.load("textures/ui/button.png")),
            ..default()
        },
        ImageScaleMode::Sliced(slicer.clone())
    );

    let option1 = commands.spawn(button.clone())
        .insert(MainMenuButton::VIEW_SPELLS)
        .id();
    let option2 = commands.spawn(button.clone())
        .insert(MainMenuButton::VIEW_ITEMS)
        .id();
    let option3 = commands.spawn(button.clone())
        .insert(MainMenuButton::VIEW_MOBS)
        .id();

    let _back_button = commands.spawn((
        ButtonBundle {
            style: Style {
                width: Val::Px(64.0),
                height: Val::Px(32.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            image: UiImage::new(asset_server.load("textures/ui/button.png")),
            ..default()
        },
        ImageScaleMode::Sliced(slicer.clone())
    ))
    .insert(MainMenuButton::MAIN_MENU)
    .insert(StateScoped(MainMenuState::AlmanachSelection))
    .with_children(|parent| {
        parent.spawn(TextBundle::from_section(
            "Назад",
            TextStyle {
                font: asset_server.load("fonts/ebbe_bold.ttf"),
                font_size: 16.0,
                color: Color::BLACK,
            }
        ));
    });

    commands.entity(content).push_children(&[option1, option2, option3]);
    commands.entity(canvas).push_children(&[content]);
}

fn spawn_view_spells(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    spell_books: ResMut<Assets<SpellBook>>,
    spell_book_handle: Res<SpellBookHandle>,
) {
    let canvas = commands.spawn(ImageBundle { 
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },  
        background_color: BackgroundColor(Color::srgb(69. / 255., 35. / 255., 13. / 255.)),
        ..default()
    })
    .insert(MainMenuUI)
    .insert(StateScoped(MainMenuState::ViewSpells))
    .id();

    let content = commands.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(90.0),
            height: Val::Percent(50.0),
            justify_self: JustifySelf::Center,
            align_self: AlignSelf::Center,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()            
        },
        ..default()
    }).id();

    let slicer = TextureSlicer {
        border: BorderRect::square(16.0),
        center_scale_mode: SliceScaleMode::Stretch,
        sides_scale_mode: SliceScaleMode::Stretch,
        ..default()
    };

    let mut entries: Vec<Entity> = vec![];

    let spells = spell_books.get(spell_book_handle.0.id()).unwrap();

    for spell in spells.spells.iter() {
        let name = spell.get("name").unwrap().as_str().unwrap();
        let recipe = spell.get("recipe").unwrap().as_array().unwrap();

        let entry = commands.spawn((
            ImageBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Px(48.),
                    flex_direction: FlexDirection::Row,
                    padding: UiRect::all(Val::Px(4.)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                image: UiImage::new(asset_server.load("textures/ui/button.png")),
                ..default()
            },
            ImageScaleMode::Sliced(slicer.clone())
        ))
        .with_children(|parent| {
            for element in recipe.iter() {
                let texture_path = match element.as_str().unwrap() {
                    "fire" => "fire_slot.png",
                    "water" => "water_slot.png",
                    "earth" => "earth_slot.png",
                    "air" => "air_slot.png",
                    _ => ""
                };

                parent.spawn(ImageBundle {
                    style: Style {
                        width: Val::Px(32.),
                        height: Val::Px(32.),
                        ..default()
                    },
                    image: UiImage::new(asset_server.load(format!("textures/{}", texture_path))),
                    ..default()
                });
            }
    
            parent.spawn(TextBundle::from_section(
            name,
            TextStyle {
                    font: asset_server.load("fonts/ebbe_bold.ttf"),
                    font_size: 32.0,
                    color: Color::BLACK,
                    ..default()
                },
            )
            .with_style(Style {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                ..default()
            })
            .with_text_justify(JustifyText::Right));
        })
        .id();

        entries.push(entry);
    }

    let _back_button = commands.spawn((
        ButtonBundle {
            style: Style {
                width: Val::Px(64.0),
                height: Val::Px(32.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            image: UiImage::new(asset_server.load("textures/ui/button.png")),
            ..default()
        },
        ImageScaleMode::Sliced(slicer.clone())
    ))
    .insert(MainMenuButton::ALMANACH)
    .insert(StateScoped(MainMenuState::ViewSpells))
    .with_children(|parent| {
        parent.spawn(TextBundle::from_section(
            "Назад",
            TextStyle {
                font: asset_server.load("fonts/ebbe_bold.ttf"),
                font_size: 16.0,
                color: Color::BLACK,
            }
        ));
    });

    commands.entity(content).push_children(&entries);
    commands.entity(canvas).push_children(&[content]);
}

pub fn handle_buttons(
    mut game_state: ResMut<NextState<GameState>>,
    mut main_menu_state: ResMut<NextState<MainMenuState>>,
    mut buttons_query: Query<(&Interaction, &MainMenuButton, &mut Style), Changed<Interaction>>,
    mut app_exit_events: ResMut<Events<bevy::app::AppExit>>,
) {
    for (interaction, button, mut style) in buttons_query.iter_mut() {
        match *interaction {
            Interaction::Hovered => {
                style.height = Val::Px(button.height + 12.0);
            }, // добавить анимации
            Interaction::Pressed => {
                match button.button {
                    ButtonType::NewRun => { game_state.set(GameState::Loading); }, // идём в загрузку
                    ButtonType::Quit => { app_exit_events.send(AppExit::Success); }, // выходим из игры
                    ButtonType::MainMenu => {
                        game_state.set(GameState::MainMenu);
                        main_menu_state.set(MainMenuState::Main); // just in case
                    }, // в главное меню
                    ButtonType::NaviagteMenu(state) => {
                        main_menu_state.set(state);
                    }
                }
            },
            Interaction::None => {
                style.height = Val::Px(button.height);
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