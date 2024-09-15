use bevy::prelude::*;

use crate::projectile::Projectile;

pub struct ElementsPlugin;

impl Plugin for ElementsPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(ElementBar::default())
            .add_systems(Update, (fill_bar, cast_spell));
    }
}

#[derive(Debug, PartialEq)]
pub enum ElementType {
    Fire = 1000, Water = 100, Earth = 10, Air = 1
}

impl ElementType {
    fn value(&self) -> i32 {
        match *self {
            ElementType::Fire => 1000,
            ElementType::Water => 100,
            ElementType::Earth => 10,
            ElementType::Air => 1
        }
    }
}

#[derive(Resource)]
pub struct ElementBar {
    pub bar: Vec<ElementType>,
    pub max: i32,
}

impl ElementBar {
    fn clear(&mut self) {
        self.bar = vec![];
    }

    fn add(&mut self, element: ElementType) {
        if (self.bar.len() as i32) < self.max {
            self.bar.push(element);
        }
        else {
            println!("[I] Element bar is full!!");
        }
    }

    fn default() -> Self {
        ElementBar {
            bar: vec![],
            max: 2,
        }
    }
}



fn fill_bar(
    mut bar: ResMut<ElementBar>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Digit1) {
        bar.add(ElementType::Fire);
        println!("{:?}", bar.bar);
    }

    if keyboard.just_pressed(KeyCode::Digit2) {
        bar.add(ElementType::Water);
        println!("{:?}", bar.bar);
    }

    if keyboard.just_pressed(KeyCode::Digit3) {
        bar.add(ElementType::Earth);
        println!("{:?}", bar.bar);
    }

    if keyboard.just_pressed(KeyCode::Digit4) {
        bar.add(ElementType::Air);
        println!("{:?}", bar.bar);
    }
}

fn cast_spell(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mouse_coords: Res<crate::mouse_position::MouseCoords>,

    player_query: Query<&Transform, With<crate::player::Player>>,

    mut bar: ResMut<ElementBar>,
    mouse: Res<ButtonInput<MouseButton>>,
) {
    if mouse.just_pressed(MouseButton::Left) && !bar.bar.is_empty() {
        let recipe: i32 = bar.bar.iter().map(|e| e.value()).sum();

        let mut spell_desc: String = "".to_string();
        let mut dmg = 0;

        dmg += (recipe / 1000) * 50 * ((recipe % 10) + 1) * (((recipe % 100) / 10) + 1); // добавляем урон от огня
        dmg += ((recipe % 1000) / 2) * ((recipe % 10) + 1) * (((recipe % 100) / 10) + 1); // урон от воды
        dmg += (recipe % 100) / 2 ; // урон от земли
        dmg += (recipe % 10) * 10; // урон от воздуха 

        if recipe >= 1000 {
            spell_desc += "fire element\n";
        }

        if recipe % 1000 >= 100 {
            spell_desc += "water element\n";
        }

        if recipe % 100 >= 10 {
            spell_desc += "AoE, e.g. earthquake\n";
        }

        if recipe % 10 > 0 {
            spell_desc += "throwable, e.g. fireball\n";

            if let Ok(player_transform) = player_query.get_single() {

                let hue = {
                    if recipe >= 1000 {
                        20.0
                    }
                    else if recipe % 1000 >= 100 {
                        200.0
                    } else {
                        300.0
                    }
                };

                commands.spawn(SpriteBundle {
                    transform: Transform::from_translation(player_transform.translation),
                    texture: asset_server.load("textures/fireball.png"),
                    sprite: Sprite {
                        color: Color::hsl(hue, 0.75, 0.5),
                        ..default()
                    },
                    ..default()
                }).insert(Projectile { direction: (mouse_coords.0 - player_transform.translation.truncate()).normalize_or_zero(), speed: 100.0, damage: dmg, is_friendly: true });
            }
        }

        println!("[{}] ({} DMG)", spell_desc, dmg);

        bar.clear();
        println!("{:?}", bar.bar);
    }
}