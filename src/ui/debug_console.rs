use bevy::prelude::*;
use bevy_simple_text_input::{
    TextInputBundle,
    TextInputPlugin,
    TextInputSubmitEvent,
    TextInputSystem, TextInputValue
};

use crate::{chapter::ChapterManager, exp_tank::SpawnExpTankEvent, health_tank::SpawnHealthTankEvent, invincibility::Invincibility, item::{ItemDatabase, ItemDatabaseHandle, ItemType, SpawnItemEvent}, player::Player, GameState};

pub struct DebugConsolePlugin;

impl Plugin for DebugConsolePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(TextInputPlugin)
            .add_systems(Startup, spawn_console)
            .add_systems(Update, toggle_console
                .run_if(in_state(GameState::InGame)
                .or_else(in_state(GameState::Hub)))
                .after(TextInputSystem))
            .add_systems(Update, handle_commands.after(TextInputSystem));
    }
}

#[derive(Component)]
struct Console;

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
                visibility: Visibility::Hidden,
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
            Console
        ));
    });
}

fn toggle_console(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Visibility, &mut TextInputValue),With<Console>>
) {
    if keyboard.just_pressed(KeyCode::Backquote) {
        let Ok((mut visibility, mut value)) = query.get_single_mut() else {
            return;
        };

        value.0 = "".to_string();

        *visibility = match *visibility {
            Visibility::Hidden => Visibility::Visible,
            Visibility::Visible => Visibility::Hidden,
            _ => Visibility::Inherited,
        };
    }
}

fn handle_commands(
    mut commands: Commands,

    mut ev_input_submit: EventReader<TextInputSubmitEvent>,

    mut ev_spawn_exp: EventWriter<SpawnExpTankEvent>,
    mut ev_spawn_hp: EventWriter<SpawnHealthTankEvent>,
    mut ev_spawn_item: EventWriter<SpawnItemEvent>,

    item_database: Res<Assets<ItemDatabase>>,
    item_database_handle: Res<ItemDatabaseHandle>,

    mut chapter_manager: ResMut<ChapterManager>,
    mut next_state: ResMut<NextState<GameState>>,

    player_query: Query<(Entity, &Transform), With<Player>>,
) {
    let Ok((player_entity, player_transform)) = player_query.get_single() else {
        return;
    };

    for ev in ev_input_submit.read() {
        let command: Vec<&str> = ev.value.split_whitespace().collect();

        match command[0] {
            "spawn" => {
                match command[1] {
                    "item" => {
                        let Ok(item_id) = command[2].parse::<usize>() else { return; };

                        let items = &item_database.get(item_database_handle.0.id()).unwrap().items;

                        let item_name: String = items[item_id]["name"].as_str().unwrap().to_string();
                        let texture_name: String = items[item_id]["texture_name"].as_str().unwrap().to_string();
                        let texture_path = format!("textures/items/{}", texture_name);
                        let item_description: String = items[item_id]["description"].as_str().unwrap().to_string();

                        ev_spawn_item.send(SpawnItemEvent {
                            pos: player_transform.translation,
                            item_type: ItemType::from_index(item_id as u32),
                            texture_path,
                            item_name,
                            item_description,
                            
                        });
                    },
                    "mob" => {}, // TODO: implement mob spawn, if needed
                    "exp" => { // spawns exp tank with set amount of orbs in it
                        let Ok(orbs) = command[2].parse::<u32>() else { return; };
                        ev_spawn_exp.send(SpawnExpTankEvent {
                            pos: player_transform.translation,
                            orbs,
                        });
                    },
                    "hp" => {
                        let Ok(hp) = command[2].parse::<i32>() else { return; };
                        ev_spawn_hp.send(SpawnHealthTankEvent {
                            pos: player_transform.translation,
                            hp,
                        });
                    },
                    _ => {}
                }
            },
            "inv" => { // cast invincibility with custom time
                let Ok(duration) = command[1].parse::<f32>() else { return; };
                commands.entity(player_entity).insert(Invincibility::new(duration));
            }
            "goto" => {
                let Ok(chapter) = command[1].parse::<u8>() else { return; };
                let Ok(level) = command[2].parse::<u8>() else { return; };
                chapter_manager.current_chapter = chapter;
                chapter_manager.current_level = level;

                next_state.set(GameState::Hub);
            }
            _ => {}
        }
    }
}