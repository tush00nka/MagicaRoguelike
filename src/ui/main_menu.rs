use bevy::prelude::*;
use serde::Deserialize;
use serde_json::{Map, Value};

use crate::{
    audio::PlayAudioEvent, item::{ItemDatabase, ItemDatabaseHandle}, mobs::{MobDatabase, MobDatabaseHandle}, save::{DeleteSaveEvent, Save, SaveHandle}, GameState, MainMenuState
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
            .add_systems(OnEnter(MainMenuState::ViewItems), spawn_view_items)
            .add_systems(OnEnter(MainMenuState::ViewMobs), spawn_view_mobs)
            .add_systems(Update, (handle_buttons, escape_from_everywhere)
                .run_if(in_state(GameState::MainMenu)))
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
    NaviagteMenu(MainMenuState),
    DeleteSave,
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
    pub const DELETE_SAVE: Self = Self {
        button: ButtonType::DeleteSave,
        height: 32.
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
                    "Начать игру", 
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
                    "Справочник", 
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
                    "Выход", 
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
            row_gap: Val::Percent(10.0),
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
    .with_children(|parent| {
        parent.spawn(ImageBundle {
            image: UiImage::new(asset_server.load("textures/ui/almanach_spells_option.png")),
            style: Style {
                width: Val::Px(256.0),
                height: Val::Px(256.0),
                ..default()
            },
            ..default()
        });
    })
        .insert(MainMenuButton::VIEW_SPELLS)
        .id();
    let option2 = commands.spawn(button.clone())
        .with_children(|parent| {
            parent.spawn(ImageBundle {
                image: UiImage::new(asset_server.load("textures/ui/almanach_items_option.png")),
                style: Style {
                    width: Val::Px(256.0),
                    height: Val::Px(256.0),
                    ..default()
                },
                ..default()
            });
        })
        .insert(MainMenuButton::VIEW_ITEMS)
        .id();
    let option3 = commands.spawn(button.clone())
    .with_children(|parent| {
        parent.spawn(ImageBundle {
            image: UiImage::new(asset_server.load("textures/ui/almanach_mobs_option.png")),
            style: Style {
                width: Val::Px(256.0),
                height: Val::Px(256.0),
                ..default()
            },
            ..default()
        });
    })
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

    let delete_save_content = commands.spawn(NodeBundle {
        style: Style {
            width: Val::Px(196.0),
            height: Val::Px(64.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        ..default()
    }).id();

    let delete_save_button = commands.spawn((
        ButtonBundle {
            style: Style {
                width: Val::Px(196.0),
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
    .insert(MainMenuButton::DELETE_SAVE)
    .insert(StateScoped(MainMenuState::AlmanachSelection))
    .with_children(|parent| {
        parent.spawn(TextBundle::from_section(
            "Сбросить прогресс",
            TextStyle {
                font: asset_server.load("fonts/ebbe_bold.ttf"),
                font_size: 16.0,
                color: Color::BLACK,
            }
        ).with_text_justify(JustifyText::Center));
    }).id();

    commands.entity(delete_save_content).push_children(&[delete_save_button]);

    commands.entity(content).push_children(&[option1, option2, option3]);
    commands.entity(canvas).push_children(&[content, delete_save_content]);
}

fn spawn_view_spells(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    spell_books: ResMut<Assets<SpellBook>>,
    spell_book_handle: Res<SpellBookHandle>,
    saves: Res<Assets<Save>>,
    save_handle: Res<SaveHandle>,
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
            height: Val::Percent(90.0),
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

        let save = saves.get(save_handle.0.id()).unwrap();
        let seen: bool = save.seen_spells.contains(&spell.get("tag").unwrap().as_str().unwrap().to_string());

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
                    "etc" => "ui/etc.png",
                    "lower" => "ui/lower.png",
                    "greater" => "ui/greater.png",
                    "equals" => "ui/equals.png",
                    _ => ""
                };

                parent.spawn(ImageBundle {
                    style: Style {
                        width: Val::Px(32.),
                        height: Val::Px(32.),
                        ..default()
                    },
                    image: UiImage::new(asset_server.load(format!("textures/{}", texture_path)))
                        .with_color(if seen { Color::WHITE } else { Color::BLACK }),
                    ..default()
                });
            }
    
            let display_name: &str = if seen { name } else { "???" };

            parent.spawn(TextBundle::from_section(
            display_name,
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

fn spawn_view_items(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    item_database: Res<Assets<ItemDatabase>>,
    handle: Res<ItemDatabaseHandle>,
    saves: Res<Assets<Save>>,
    save_handle: Res<SaveHandle>,
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
    .insert(StateScoped(MainMenuState::ViewItems))
    .id();

    let content = commands.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(90.0),
            height: Val::Percent(90.0),
            justify_self: JustifySelf::Center,
            align_self: AlignSelf::Center,
            display: Display::Grid,
            grid_auto_flow: GridAutoFlow::Row,

            grid_template_columns: RepeatedGridTrack::flex(8, 1.0),
            grid_template_rows: RepeatedGridTrack::flex(4, 1.0),
            row_gap: Val::Px(4.0),
            column_gap: Val::Px(4.0),

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

    let items = &item_database.get(handle.0.id()).unwrap().items;

    let mut entries: Vec<Entity> = vec![];

    let save = saves.get(save_handle.0.id()).unwrap();

    for item in items.iter() {
        let name = item["name"].as_str().unwrap();
        let description = item["description"].as_str().unwrap();
        let texture_name = item["texture_name"].as_str().unwrap();

        let texture_path = format!("textures/items/{}", texture_name);

        let seen: bool = save.seen_items.contains(&item.get("texture_name").unwrap().as_str().unwrap().to_string());

        let entry = commands.spawn((
            ImageBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(4.)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    aspect_ratio: Some(1.0),
                    ..default()
                },
                image: UiImage::new(asset_server.load("textures/ui/button.png")),
                ..default()
            },
            ImageScaleMode::Sliced(slicer.clone())
        ))
        .with_children(|parent| { 

            parent.spawn(ImageBundle {
                image: UiImage::new(asset_server.load(texture_path))
                    .with_color(if seen { Color::WHITE } else { Color::BLACK }),
                style: Style {
                    width: Val::Px(48.),
                    height: Val::Px(48.),
                    ..default()
                },
                ..default()
            });

            let display_name = if seen { name } else { "???" };
            let display_description = if seen { description } else { "???" };

            parent.spawn(TextBundle::from_sections([
                TextSection::new(format!("{}\n\n", display_name), TextStyle {
                        font: asset_server.load("fonts/ebbe_bold.ttf"),
                        font_size: 16.0,
                        color: Color::BLACK,
                        ..default()
                    },
                ),

                TextSection::new(display_description, TextStyle {
                    font: asset_server.load("fonts/ebbe_bold.ttf"),
                    font_size: 10.0,
                    color: Color::BLACK,
                    ..default()
                },
            )])
            .with_text_justify(JustifyText::Center)
            .with_style(Style {
                width: Val::Percent(100.),
                height: Val::Percent(50.),
                ..default()
            }));
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
    .insert(StateScoped(MainMenuState::ViewItems))
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

fn spawn_view_mobs(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mob_database: Res<Assets<MobDatabase>>,
    handle: Res<MobDatabaseHandle>,
    saves: Res<Assets<Save>>,
    save_handle: Res<SaveHandle>,
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
    .insert(StateScoped(MainMenuState::ViewMobs))
    .id();

    let content = commands.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(90.0),
            height: Val::Percent(90.0),
            justify_self: JustifySelf::Center,
            align_self: AlignSelf::Center,
            display: Display::Grid,
            grid_auto_flow: GridAutoFlow::Row,

            grid_template_columns: RepeatedGridTrack::flex(8, 1.0),
            grid_template_rows: RepeatedGridTrack::flex(4, 1.0),
            row_gap: Val::Px(4.0),
            column_gap: Val::Px(4.0),

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

    let items = &mob_database.get(handle.0.id()).unwrap().mobs;

    let mut entries: Vec<Entity> = vec![];
    let save = saves.get(save_handle.0.id()).unwrap();

    for item in items.iter() {
        let name = item["name"].as_str().unwrap();
        let description = item["description"].as_str().unwrap();
        let texture_name = item["texture_name"].as_str().unwrap();

        let texture_path = format!("textures/ui/mob_portraits/{}", texture_name);

        let seen: bool = save.seen_mobs.contains(&texture_name.to_string());

        let entry = commands.spawn((
            ImageBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(4.)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    aspect_ratio: Some(1.0),
                    ..default()
                },
                image: UiImage::new(asset_server.load("textures/ui/button.png")),
                ..default()
            },
            ImageScaleMode::Sliced(slicer.clone())
        ))
        .with_children(|parent| { 

            parent.spawn(ImageBundle {
                image: UiImage::new(asset_server.load(texture_path))
                    .with_color(if seen { Color::WHITE } else { Color::BLACK }),
                style: Style {
                    width: Val::Px(48.),
                    height: Val::Px(48.),
                    ..default()
                },
                ..default()
            });

            let display_name = if seen { name } else { "???" };
            let display_description = if seen { description } else { "???" };

            parent.spawn(TextBundle::from_sections([
                TextSection::new(format!("{}\n\n", display_name), TextStyle {
                        font: asset_server.load("fonts/ebbe_bold.ttf"),
                        font_size: 16.0,
                        color: Color::BLACK,
                        ..default()
                    },
                ),

                TextSection::new(display_description, TextStyle {
                    font: asset_server.load("fonts/ebbe_bold.ttf"),
                    font_size: 10.0,
                    color: Color::BLACK,
                    ..default()
                },
            )])
            .with_text_justify(JustifyText::Center)
            .with_style(Style {
                width: Val::Percent(100.),
                height: Val::Percent(50.),
                ..default()
            }));
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
    .insert(StateScoped(MainMenuState::ViewMobs))
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

fn escape_from_everywhere(
    mut next_state: ResMut<NextState<MainMenuState>>,
    current_state: Res<State<MainMenuState>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        match current_state.get() {
            MainMenuState::Settings => next_state.set(MainMenuState::Main),
            MainMenuState::AlmanachSelection => next_state.set(MainMenuState::Main),
            MainMenuState::ViewSpells => next_state.set(MainMenuState::AlmanachSelection),
            MainMenuState::ViewItems => next_state.set(MainMenuState::AlmanachSelection),
            MainMenuState::ViewMobs => next_state.set(MainMenuState::AlmanachSelection),
            _ => {}
        }
    }
}

pub fn handle_buttons(
    mut game_state: ResMut<NextState<GameState>>,
    mut main_menu_state: ResMut<NextState<MainMenuState>>,
    mut buttons_query: Query<(&Interaction, &MainMenuButton, &mut Style), Changed<Interaction>>,
    mut app_exit_events: ResMut<Events<bevy::app::AppExit>>,
    mut ev_play_audio: EventWriter<PlayAudioEvent>, 
    mut ev_delete_save: EventWriter<DeleteSaveEvent>
) {
    for (interaction, button, mut style) in buttons_query.iter_mut() {
        match *interaction {
            Interaction::Hovered => {
                style.height = Val::Px(button.height + 12.0);
            }, // добавить анимации
            Interaction::Pressed => {
                ev_play_audio.send(PlayAudioEvent::from_file("tick.ogg"));

                match button.button {
                    ButtonType::NewRun => { game_state.set(GameState::Loading); }, // идём в загрузку
                    ButtonType::Quit => { app_exit_events.send(AppExit::Success); }, // выходим из игры
                    ButtonType::MainMenu => {
                        game_state.set(GameState::MainMenu);
                        main_menu_state.set(MainMenuState::Main); // just in case
                    }, // в главное меню
                    ButtonType::NaviagteMenu(state) => {
                        main_menu_state.set(state);
                    },
                    ButtonType::DeleteSave => {
                        ev_delete_save.send(DeleteSaveEvent);
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