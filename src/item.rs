use std::f32::consts::PI;

use bevy::prelude::*;
use avian2d::prelude::*;

use bevy_common_assets::json::JsonAssetPlugin;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use serde_json::{Map, Value};

use crate::{camera::YSort, player::Player, save::{Save, SaveHandle}};

pub struct ItemPlugin;

impl Plugin for ItemPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(JsonAssetPlugin::<ItemDatabase>::new(&["json"]))
            .add_event::<SpawnItemEvent>()
            .add_event::<ItemPickedUpEvent>()
            .add_systems(Startup, load_item_database)
            .add_systems(Startup, spawn_item_hint)
            .add_systems(Update, (
                spawn_item,
                pick_up_item,
                item_wobble,
                update_item_hint,
                item_pickup_animation
            ));
    }
}

#[allow(unused)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum ItemType { // Keep enum variants in alphabetical order or it will break, pls, thanks
    Amulet,
    Aquarius,
    Bacon,
    Blank,
    BlindRage,
    BloodGoblet,
    ElementWheel,
    Fan,
    FieryShard,
    GhostInTheShell,
    Glider,
    Heart,
    LizardTail,
    Mineral,
    NotchedPickaxe,
    Shield,
    SpeedPotion,
    Valve,
    VampireTooth,
    WaterbendingScroll,
    WispInAJar,
}

impl ItemType {
    pub fn from_index(index: u32) -> Self {
        match index {
            0 => ItemType::Amulet,
            1 => ItemType::Bacon,
            2 => ItemType::Heart,
            3 => ItemType::LizardTail,
            4 => ItemType::SpeedPotion,
            5 => ItemType::WispInAJar,
            6 => ItemType::WaterbendingScroll,
            7 => ItemType::Mineral,
            8 => ItemType::Glider,
            9 => ItemType::GhostInTheShell,
            10 => ItemType::VampireTooth,
            11 => ItemType::BloodGoblet,
            12 => ItemType::BlindRage,
            13 => ItemType::Valve,
            14 => ItemType::FieryShard,
            15 => ItemType::ElementWheel,
            16 => ItemType::NotchedPickaxe,
            17 => ItemType::Fan,
            18 => ItemType::Shield,
            19 => ItemType::Blank,
            20 => ItemType::Aquarius,
            _ => ItemType::Amulet, 
        }
    }
}

// я не знаю, что это за волшебный код,
// но он делает именно то, что я хочу
impl Distribution<ItemType> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ItemType {
        ItemType::from_index(rng.gen_range(0..=20))
    }
}


#[derive(serde::Deserialize, Asset, TypePath)]
pub struct ItemDatabase {
    pub items: Vec<Map<String, Value>>,
}

#[derive(Resource)]
pub struct ItemDatabaseHandle(pub Handle<ItemDatabase>); 

fn load_item_database(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(ItemDatabaseHandle(asset_server.load("items.json")));
}

#[derive(Component)]
pub struct Item {
    pub item_type: ItemType,
    name: String,
    description: String,
}

const PLAYER_DETECTION_RADIUS: f32 = 64.0;

#[derive(Component)]
struct ItemHint;

#[derive(Event)]
pub struct SpawnItemEvent {
    pub pos: Vec3,
    pub item_type: ItemType,
    pub texture_path: String,
    pub item_name: String,
    pub item_description: String,
}

#[derive(Event)]
pub struct ItemPickedUpEvent {
    pub item_type: ItemType,
}

#[derive(Component)]
pub struct ItemPickupAnimation {
    timer: Timer,
}

#[derive(Component)]
struct HeldItem; 

fn spawn_item(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut ev_spawn_item: EventReader<SpawnItemEvent>,
) {
    for ev in ev_spawn_item.read() {
        commands.spawn(SpriteBundle {
            texture: asset_server.load(ev.texture_path.clone()),
            transform: Transform::from_translation(ev.pos),
            ..default()
        })
        .insert(Collider::circle(8.0,))
        .insert(Sensor)
        .insert(Item {
            item_type: ev.item_type,
            name: ev.item_name.clone(),
            description: ev.item_description.clone(),
        })
        .insert(YSort(8.0));
    }
}

fn spawn_item_hint(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Text2dBundle {
        text: Text::from_sections([
            TextSection::new(
                "item_name",
                TextStyle {
                    font: asset_server.load("fonts/ebbe_bold.ttf"),
                    font_size: 24.0,
                    color: Color::hsl(40.0, 1., 0.5),
                }
            ),
            TextSection::new(
                "item_description",
                TextStyle {
                    font: asset_server.load("fonts/ebbe_bold.ttf"),
                    font_size: 16.0,
                    color: Color::WHITE,
                }
            )
        ]).with_justify(JustifyText::Center),
        text_anchor: bevy::sprite::Anchor::Center,
        transform: Transform {
            translation: Vec3::ZERO.with_z(9.0),
            scale: Vec3::splat(0.5),
            ..default()
        },
        visibility: Visibility::Hidden,
        ..default()
    })
    .insert(ItemHint);
}

fn pick_up_item(
    mut commands: Commands,
    asset_server: Res<AssetServer>,

    item_query: Query<(Entity, &Item)>,
    player_query: Query<(Entity, &CollidingEntities), With<Player>>,

    held_query: Query<Entity, With<HeldItem>>,

    mut ev_item_picked_up: EventWriter<ItemPickedUpEvent>,

    item_database: Res<Assets<ItemDatabase>>,
    handle: Res<ItemDatabaseHandle>,

    mut saves: ResMut<Assets<Save>>,
    save_handle: Res<SaveHandle>
) {
    let Ok((player_e, colliding_e)) = player_query.get_single() else {
        return;
    };

    let save = saves.get_mut(save_handle.0.id()).unwrap();

    if !held_query.is_empty() {
        return;
    } 

    for (item_e, item) in item_query.iter() {
        if colliding_e.contains(&item_e) {            
            ev_item_picked_up.send(ItemPickedUpEvent {
                item_type: item.item_type,
            });

            println!("Picked up: {:?}", item.item_type);

            let texture_name: String = item_database.get(handle.0.id()).unwrap().items[item.item_type as usize]["texture_name"].as_str().unwrap().to_string();
            let texture_path = format!("textures/items/{}", texture_name);

            if !save.seen_items.contains(&texture_name) {
                save.seen_items.push(texture_name);
            }

            commands.entity(player_e)
            .insert(ItemPickupAnimation {
                timer: Timer::from_seconds(1.0, TimerMode::Once),
            })
            .with_children(|parent| {
                parent.spawn(SpriteBundle {
                    texture: asset_server.load(texture_path),
                    transform: Transform::from_translation(Vec3::new(0.0, 16.0, 1.0)),
                    ..default()
                }).insert(HeldItem);
            }); 

            commands.entity(item_e).despawn();

            break;
        }
    }
}

fn item_wobble(
    mut item_query: Query<&mut Transform, (With<Item>, Without<Player>)>,
    player_query: Query<&Transform, (With<Player>, Without<Item>)>,
    time: Res<Time>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    if item_query.is_empty() {
        return;
    }

    let mut sorted: Vec<Mut<'_, Transform>> = item_query.iter_mut()
        .sort_by::<&Transform>(|t1, t2| {
            t1.translation.distance(player_transform.translation)
                .total_cmp(&t2.translation.distance(player_transform.translation))
        }).collect();

    // делим массив на содержащий первый (ближайший) трансформ и на содержащий все остальные
    let (split1, split2) = sorted.split_at_mut(1);

    // берем этот ближайший странсформ из первого сплита
    let item_transform = &mut split1[0];

    // сбрасываем вращение у всех остальных
    for i in 1..split2.len() {
        split2[i].rotation = Quat::IDENTITY;
    }

    // вращаем ближайший
    if item_transform.translation.distance(player_transform.translation) <= PLAYER_DETECTION_RADIUS {
        let angle = (time.elapsed_seconds() * 10.0).sin() * PI / 12.0;
        item_transform.rotation = Quat::from_rotation_z(angle);
    }   
    else {
        item_transform.rotation = Quat::IDENTITY;
    }
}

fn update_item_hint(
    mut hint_query: Query<(&mut Visibility, &mut Transform, &mut Text), (With<ItemHint>, Without<Player>, Without<Item>)>,
    item_query: Query<(&Transform, &Item), (Without<Player>, Without<ItemHint>)>,
    player_query: Query<&Transform, (With<Player>, Without<Item>, Without<ItemHint>)>,
) {
    let Ok((mut hint_visibility, mut hint_transform, mut hint_text)) = hint_query.get_single_mut() else {
        return;
    };

    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    if item_query.is_empty() {
        *hint_visibility = Visibility::Hidden;
        return;
    }

    let sorted: Vec<(&Transform, &Item)> = item_query.iter()
        .sort_by::<&Transform>(|t1, t2| {
            t1.translation.distance(player_transform.translation)
                .total_cmp(&t2.translation.distance(player_transform.translation))
        }).collect();

    let (item_transform, item) = sorted[0];

    if item_transform.translation.distance(player_transform.translation) <= PLAYER_DETECTION_RADIUS {
        *hint_visibility = Visibility::Visible;
        hint_transform.translation.x = item_transform.translation.x;
        hint_transform.translation.y = item_transform.translation.y + 24.0;
        hint_text.sections[0].value = format!("{}\n", item.name);
        hint_text.sections[1].value = item.description.to_string();
    }
    else {
        *hint_visibility = Visibility::Hidden;
    }
}

fn item_pickup_animation(
    mut commands: Commands,
    mut player_query: Query<(Entity, &mut Transform, &mut ItemPickupAnimation, &mut TextureAtlas), With<Player>>,
    held_item_query: Query<Entity, With<HeldItem>>,
    time: Res<Time>,
) {
    let Ok((entity, mut transform, mut anim, mut atlas)) = player_query.get_single_mut() else {
        return;
    };

    let Ok(held_e) = held_item_query.get_single() else {
        return;
    };

    if anim.timer.elapsed_secs() <= time.delta_seconds() {
        atlas.index = 8; // the sprite of player holding smth

         // фикс того, что в моменте подбора игрок может быть в состоянии поворота
        transform.scale = Vec3::splat(1.0);
    }

    anim.timer.tick(time.delta());

    if anim.timer.just_finished() {
        atlas.index = 0;
        commands.entity(entity).remove::<ItemPickupAnimation>();
        commands.entity(held_e).despawn();
    }
}