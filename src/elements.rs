use std::f32::consts::PI;

use bevy::prelude::*;
use avian2d::prelude::{Collider, Sensor};
use rand::Rng;

use crate::projectile::{Projectile, ProjectileBundle};

pub struct ElementsPlugin;

impl Plugin for ElementsPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(ElementBar::default())
            .add_event::<ElementBarFilled>()
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
    pub max: u8,
}

impl ElementBar {
    fn clear(&mut self) {
        self.bar = vec![];
    }

    fn add(&mut self, element: ElementType) {
        if (self.bar.len() as u8) < self.max {
            self.bar.push(element);
        }
        else {
            println!("[I] Element bar is full!!");
        }
    }

    fn default() -> Self {
        ElementBar {
            bar: vec![],
            max: 1,
        }
    }
}

#[derive(Event)]
pub struct ElementBarFilled;

fn fill_bar(
    mut bar: ResMut<ElementBar>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut ev_bar_filled: EventWriter<ElementBarFilled>,
) {

    keyboard.get_just_pressed().for_each(|key| {
        match key {
            KeyCode::Digit1 => { bar.add(ElementType::Fire) }
            KeyCode::Digit2 => { bar.add(ElementType::Water) }
            KeyCode::Digit3 => { bar.add(ElementType::Earth) }
            KeyCode::Digit4 => { bar.add(ElementType::Air) }
            _ => {}
        }

        ev_bar_filled.send(ElementBarFilled);
    });
}

fn cast_spell(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mouse_coords: Res<crate::mouse_position::MouseCoords>,

    player_query: Query<&Transform, With<crate::player::Player>>,

    mut bar: ResMut<ElementBar>,
    mouse: Res<ButtonInput<MouseButton>>,

    mut ev_bar_filled: EventWriter<ElementBarFilled>,
) {
    if mouse.just_pressed(MouseButton::Left) && !bar.bar.is_empty() {
        
        ev_bar_filled.send(ElementBarFilled);

        let recipe: i32 = bar.bar.iter().map(|e| e.value()).sum();

        let mut spell_desc: String = "".to_string();
        let mut dmg = 0;

        dmg += (recipe / 1000) * 50 * ((recipe % 10) + 1) * (((recipe % 100) / 10) + 1); // добавляем урон от огня
        dmg += ((recipe % 1000) / 2) * ((recipe % 10) + 1) * (((recipe % 100) / 10) + 1); // урон от воды
        dmg += (recipe % 100) / 2 ; // урон от земли
        dmg += (recipe % 10) * 10; // урон от воздуха 

        let color = {
            if recipe >= 1000 {
                Color::hsl(20.0, 0.75, 0.5)
            }
            else if recipe % 1000 >= 100 {
                Color::hsl(200.0, 0.75, 0.5)
            } else if recipe % 100 >= 10 {
                Color::hsl(20.0, 0.5, 0.5)
            } else {
                Color::hsl(200.0, 0.25, 0.75)

            }
        };

        let mut rng = rand::thread_rng();

        let fire_elements = recipe / 1000;
        let water_elements = (recipe % 1000) / 100;
        let earth_elements = recipe % 100 / 10;
        let air_elements = recipe % 10;

        let total_elements = fire_elements + water_elements + earth_elements + air_elements;

        if let Ok(player_transform) = player_query.get_single() {

            match recipe {

                120 | 130 | 140 | 150 | 160 | 170 | 180 => {
                    spell_desc += "shield\n";
                }

                1111 | 2222 => {
                    spell_desc += "black hole\n";
                }

                _ => {
                    if recipe >= 1000 {
                        spell_desc += "fire element\n";
                        if recipe % 100 < 10 && recipe % 10 <= 0 {
                            let offset = PI/12.0;
                            for _i in 0..fire_elements*3 {
            
                                let dir = (mouse_coords.0 - player_transform.translation.truncate()).normalize_or_zero();
                                let angle = dir.y.atan2(dir.x) + rng.gen_range(-offset..offset);
            
                                commands.spawn(ProjectileBundle {
                                    sprite: SpriteBundle {
                                        transform: Transform {
                                            translation: player_transform.translation,
                                            rotation: Quat::from_rotation_z(angle),
                                            ..default()
                                        },
                                        texture: asset_server.load("textures/small_fire.png"),
                                        sprite: Sprite {
                                            color,
                                            ..default()
                                        },
                                        ..default()
                                    },
        
                                    projectile: Projectile {
                                        direction: Vec2::from_angle(angle),
                                        speed: 200.0 + rng.gen_range(0.0..50.0),
                                        damage: dmg/ fire_elements,
                                        is_friendly: true
                                    },
                                    collider: Collider::circle(8.0),
                                    sensor: Sensor,
                                });
                            }
                        }
                    }
            
                    if recipe % 1000 >= 100 {
                        spell_desc += "water element\n";
            
                        if recipe % 100 < 10 && recipe % 10 <= 0 {
                            let offset = PI/12.0;
                            for _i in 0..water_elements*3 {
            
                                let dir = (mouse_coords.0 - player_transform.translation.truncate()).normalize_or_zero();
                                let angle = dir.y.atan2(dir.x) + rng.gen_range(-offset..offset);
            
                                commands.spawn(ProjectileBundle {
                                    sprite: SpriteBundle {
                                        transform: Transform {
                                            translation: player_transform.translation,
                                            rotation: Quat::from_rotation_z(angle),
                                            ..default()
                                        },
                                        texture: asset_server.load("textures/small_fire.png"),
                                        sprite: Sprite {
                                            color,
                                            ..default()
                                        },
                                        ..default()
                                    },
        
                                    projectile: Projectile {
                                        direction: Vec2::from_angle(angle),
                                        speed: 200.0 + rng.gen_range(0.0..50.0),
                                        damage: dmg / water_elements,
                                        is_friendly: true
                                    },
                                    collider: Collider::circle(8.0),
                                    sensor: Sensor,
                                });
                            }
                        }
                    }
            
                    if recipe % 100 >= 10 {
                        spell_desc += "AoE, e.g. earthquake\n";
            
                        if recipe % 10 <= 0 {
                            let offset = (2.0*PI)/(total_elements*3) as f32;
                            for i in 0..total_elements*3 {
            
                                let angle = offset * i as f32;
            
                                commands.spawn(ProjectileBundle {
                                    sprite: SpriteBundle {
                                        transform: Transform {
                                            translation: player_transform.translation,
                                            rotation: Quat::from_rotation_z(angle),
                                            ..default()
                                        },
                                        texture: asset_server.load("textures/earthquake.png"),
                                        sprite: Sprite {
                                            color,
                                            ..default()
                                        },
                                        ..default()
                                    },
        
                                    projectile: Projectile {
                                        direction: Vec2::from_angle(angle),
                                        speed: 100.0,
                                        damage: dmg,
                                        is_friendly: true
                                    },
                                    collider: Collider::circle(12.0),
                                    sensor: Sensor,
                                });
                            }
                        }
                    }
            
                    if recipe % 10 > 0 {
                        spell_desc += "throwable, e.g. fireball\n";
            
                        let dir = (mouse_coords.0 - player_transform.translation.truncate()).normalize_or_zero();
                        let angle = dir.y.atan2(dir.x);
        
                        commands.spawn(ProjectileBundle {
                            sprite: SpriteBundle {
                                transform: Transform {
                                    translation: player_transform.translation,
                                    rotation: Quat::from_rotation_z(angle),
                                    scale: Vec3::ONE * total_elements as f32* 0.5,
                                    ..default()
                                },
                                texture: asset_server.load("textures/fireball.png"),
                                sprite: Sprite {
                                    color,
                                    ..default()
                                },
                                ..default()
                            },
        
                            projectile: Projectile {
                                direction: dir,
                                speed: 150.0,
                                damage: dmg,
                                is_friendly: true
                            },
                            collider: Collider::circle(8.0),
                            sensor: Sensor,
                        });
                    }
                }
            }
        }

        println!("[{}] ({} DMG)", spell_desc, dmg);

        bar.clear();
        println!("{:?}", bar.bar);
    }
}