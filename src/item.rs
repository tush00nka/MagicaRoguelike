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
            .add_systems(Update, (debug_spawn_random_item, spawn_item, pick_up_item));
    }
}

#[allow(unused)]
#[derive(Copy, Clone, PartialEq)]
pub enum ItemType {
    Amulet,
    Bacon,
    Heart,
    LizardTail,
    SpeedPotion,
    WispInAJar,
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
        }
    }
}

// я не знаю, что это за волшебный код,
// но он делает именно то, что я хочу
impl Distribution<ItemType> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ItemType {
        match rng.gen_range(0..=5) {
            0 => ItemType::Amulet,
            1 => ItemType::Heart,
            2 => ItemType::LizardTail,
            3 => ItemType::SpeedPotion,
            4 => ItemType::WispInAJar,
            5 => ItemType::Bacon,
            _ => ItemType::WispInAJar,
        }
    }
}

#[allow(unused)]
#[derive(Component)]
pub struct Item {
    item_type: ItemType,
}

#[derive(Event)]
pub struct SpawnItemEvent {
    pub pos: Vec3,
    pub item_type: ItemType,
    pub texture_path: String,
}

#[derive(Event)]
pub struct ItemPickedUpEvent {
    pub item_type: ItemType,
    pub texture_path: String,
}

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
        });
    }
}

fn debug_spawn_random_item(
    mut ev_spawn_item: EventWriter<SpawnItemEvent>,
    mouse_coords: Res<MouseCoords>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::KeyI) {
        let rand_item: ItemType = rand::random();

        ev_spawn_item.send(SpawnItemEvent {
            pos: Vec3::new(mouse_coords.0.x, mouse_coords.0.y, 1.),
            item_type: rand_item,
            texture_path: rand_item.get_texture_path().to_string(),
        });
    }
}

fn pick_up_item(
    mut commands: Commands,
    mut ev_collision: EventReader<Collision>,
    item_query: Query<(Entity, &Item)>,
    player_query: Query<Entity, With<Player>>,
    mut ev_item_picked_up: EventWriter<ItemPickedUpEvent>,
) {
    for Collision(contacts) in ev_collision.read() {
        let item_e: Option<Entity>;

        if item_query.contains(contacts.entity2) && player_query.contains(contacts.entity1) {
            item_e = Some(contacts.entity2);
        }
        else if item_query.contains(contacts.entity1) && player_query.contains(contacts.entity2) {
            item_e = Some(contacts.entity1);
        }
        else {
            item_e = None;
        }

        for (candidate_e, item) in item_query.iter() {

            if item_e.is_some() && item_e.unwrap() == candidate_e {
                ev_item_picked_up.send(ItemPickedUpEvent {
                    item_type: item.item_type,
                    texture_path: item.item_type.get_texture_path().to_string(),
                });
                commands.entity(item_e.unwrap()).despawn();
            }
        }
    }
}