use std::f32::consts::PI;

use bevy::prelude::*;
use avian2d::prelude::*;

use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

use crate::{mouse_position::MouseCoords, player::Player};

pub struct ItemPlugin;

impl Plugin for ItemPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<SpawnItemEvent>()
            .add_event::<ItemPickedUpEvent>()
            .add_systems(Startup, spawn_item_hint)
            .add_systems(Update, (
                debug_spawn_random_item,
                spawn_item,
                pick_up_item,
                item_wobble,
                update_item_hint,
                item_pickup_animation
            ));
    }
}

#[allow(unused)]
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum ItemType {
    Amulet,
    Bacon,
    Heart,
    LizardTail,
    SpeedPotion,
    WispInAJar,
    WaterbendingScroll,
    Mineral,
    Glider,
}

impl ItemType {
    pub fn get_texture_path(&self) -> &str{
        match self {
            ItemType::Amulet => "textures/items/amulet.png",
            ItemType::Bacon => "textures/items/bacon.png",
            ItemType::Heart => "textures/items/heart.png",
            ItemType::LizardTail => "textures/items/lizard_tail.png",
            ItemType::SpeedPotion => "textures/items/speed_potion.png",
            ItemType::WispInAJar => "textures/items/wisp_in_a_jar.png",
            ItemType::WaterbendingScroll => "textures/items/waterbending_scroll.png",
            ItemType::Mineral => "textures/items/mineral.png",
            ItemType::Glider => "textures/items/glider.png",
        }
    }

    pub fn get_name(&self) -> &str {
        match self {
            ItemType::Amulet => "Амулет",
            ItemType::Bacon => "Бекон",
            ItemType::Heart => "Сердце",
            ItemType::LizardTail => "Хвост Ящерицы",
            ItemType::SpeedPotion => "Снадобье Скорости",
            ItemType::WispInAJar => "Дух в Банке",
            ItemType::WaterbendingScroll => "Свиток Магии Воды",
            ItemType::Mineral => "Минерал",
            ItemType::Glider => "Воздушный Руль",
        }
    }

    pub fn get_description(&self) -> &str {
        match self {
            ItemType::Amulet => "Больше опыта от всех источников",
            ItemType::Bacon => "Неуязвимость после получения урона длится дольше",
            ItemType::Heart => "Больше максимального здоровья",
            ItemType::LizardTail => "Вторая жизнь",
            ItemType::SpeedPotion => "Больше скорость ходьбы",
            ItemType::WispInAJar => "Сопротивление огню",
            ItemType::WaterbendingScroll => "Сопротивление воде",
            ItemType::Mineral => "Сопротивление земле",
            ItemType::Glider => "Сопротивление воздуху",
        }
    }
}

// я не знаю, что это за волшебный код,
// но он делает именно то, что я хочу
impl Distribution<ItemType> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ItemType {
        match rng.gen_range(0..=8) {
            0 => ItemType::Amulet,
            1 => ItemType::Bacon,
            2 => ItemType::Heart,
            3..5 => ItemType::LizardTail,
            5 => ItemType::SpeedPotion,
            6 => ItemType::WispInAJar,
            7 => ItemType::WaterbendingScroll,
            8 => ItemType::Mineral,
            9 => ItemType::Glider,
            _ => ItemType::WispInAJar,
        }
    }
}

#[derive(Component)]
pub struct Item {
    item_type: ItemType,
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
        });
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

fn debug_spawn_random_item(
    mut ev_spawn_item: EventWriter<SpawnItemEvent>,
    mouse_coords: Res<MouseCoords>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::KeyI) {
        let rand_item: ItemType = rand::random::<ItemType>();

        ev_spawn_item.send(SpawnItemEvent {
            pos: Vec3::new(mouse_coords.0.x, mouse_coords.0.y, 1.),
            item_type: rand_item,
            texture_path: rand_item.get_texture_path().to_string(),
            item_name: rand_item.get_name().to_string(),
            item_description: rand_item.get_description().to_string()
        });
    }
}

fn pick_up_item(
    mut commands: Commands,
    asset_server: Res<AssetServer>,

    item_query: Query<(Entity, &Item)>,
    player_query: Query<(Entity, &CollidingEntities), (With<Player>, Without<ItemPickupAnimation>)>,

    mut ev_item_picked_up: EventWriter<ItemPickedUpEvent>,
) {
    let Ok((player_e, colliding_e)) = player_query.get_single() else {
        return;
    };

    for (item_e, item) in item_query.iter() {
        if colliding_e.contains(&item_e) {
            ev_item_picked_up.send(ItemPickedUpEvent {
                item_type: item.item_type,
            });

            commands.entity(player_e)
                .insert(ItemPickupAnimation {
                    timer: Timer::from_seconds(1.0, TimerMode::Once),
                })
                .with_children(|parent| {
                    parent.spawn(SpriteBundle {
                        texture: asset_server.load(item.item_type.get_texture_path().to_string()),
                        transform: Transform::from_translation(Vec3::new(0.0, 16.0, 1.0)),
                        ..default()
                    }).insert(HeldItem);
                });


            commands.entity(item_e).despawn();
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
    mut player_query: Query<(Entity, &mut ItemPickupAnimation, &mut TextureAtlas), With<Player>>,
    held_item_query: Query<Entity, With<HeldItem>>,
    time: Res<Time>,
) {
    let Ok((entity, mut anim, mut atlas)) = player_query.get_single_mut() else {
        return;
    };

    let Ok(held_e) = held_item_query.get_single() else {
        return;
    };

    if anim.timer.elapsed_secs() <= time.delta_seconds() {
        atlas.index = 8; // the sprite of player holding smth
    }

    anim.timer.tick(time.delta());

    if anim.timer.just_finished() {
        atlas.index = 0;
        commands.entity(entity).remove::<ItemPickupAnimation>();
        commands.entity(held_e).despawn();
    }
}