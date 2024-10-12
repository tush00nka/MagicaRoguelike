use std::f32::consts::PI;

use bevy::prelude::*;
use rand::Rng;

use crate::{
    projectile::SpawnProjectileEvent,
    shield_spell::SpawnShieldEvent,
    wand::Wand,
    GameState
};

pub struct ElementsPlugin;

impl Plugin for ElementsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<ElementBarFilled>()
            .add_event::<ElementBarClear>()
            .insert_resource(ElementBar::default())
            .add_systems(OnExit(GameState::MainMenu), init_bar)
            .add_systems(Update, (fill_bar, cast_spell));
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum ElementType {
    Fire,
    Water,
    Earth,
    Air,
    Steam,
}

#[derive(Resource)]
pub struct ElementBar {
    pub fire: u8,
    pub water: u8,
    pub earth: u8,
    pub air: u8,
    pub max: u8,
}

impl ElementBar {
    fn clear(&mut self) {
        self.fire = 0;
        self.water = 0;
        self.earth = 0;
        self.air = 0;
    }

    pub fn len(&self) -> u8{
        self.fire + self.water + self.earth + self.air
    }

    fn add(&mut self, element: ElementType) {
        if self.len() < self.max {
            match element {
                ElementType::Fire => self.fire+=1,
                ElementType::Water => self.water+=1,
                ElementType::Earth => self.earth+=1,
                ElementType::Air => self.air+=1,
                ElementType::Steam => {}
            }
        }
    }

    fn default() -> Self {
        ElementBar {
            fire: 0,
            water: 0,
            earth: 0,
            air: 0,
            max: 1,
        }
    }
}

#[derive(Event)]
pub struct ElementBarFilled(pub ElementType);

#[derive(Event)]
pub struct ElementBarClear;

fn init_bar(
    mut commands: Commands,
) {
    commands.insert_resource(ElementBar::default());
}

fn fill_bar(
    mut bar: ResMut<ElementBar>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut ev_bar_filled: EventWriter<ElementBarFilled>,
) {
    keyboard.get_just_pressed().for_each(|key| {
        let new_element: Option<ElementType>;

        match key {
            KeyCode::Digit1 => { new_element = Some(ElementType::Fire) }
            KeyCode::Digit2 => { new_element = Some(ElementType::Water) }
            KeyCode::Digit3 => { new_element = Some(ElementType::Earth) }
            KeyCode::Digit4 => { new_element = Some(ElementType::Air) }
            _ => { new_element = None }
        }

        if new_element.is_some() && bar.len() < bar.max {
            ev_bar_filled.send(ElementBarFilled(new_element.unwrap()));
            bar.add(new_element.unwrap());
        }

    });
}

fn cast_spell(
    mouse_coords: Res<crate::mouse_position::MouseCoords>,

    wand_query: Query<&Transform, With<Wand>>,

    mut ev_spawn_shield: EventWriter<SpawnShieldEvent>,
    mut ev_spawn_projectile: EventWriter<SpawnProjectileEvent>,

    mut bar: ResMut<ElementBar>,
    mut ev_bar_clear: EventWriter<ElementBarClear>,

    mouse: Res<ButtonInput<MouseButton>>,
) {
    if mouse.just_pressed(MouseButton::Left) && bar.len() > 0 {
        
        ev_bar_clear.send(ElementBarClear);

        let mut spell_desc: String = "".to_string();

        let mut dmg = 0;
        dmg += bar.fire as u32 * 20;
        dmg += bar.water as u32 * 20;
        dmg += bar.earth as u32 * 20;
        dmg += bar.air as u32 * 20;

        let mut rng = rand::thread_rng();

        let mut element: ElementType;
        let elements_to_comapre = vec![bar.fire, bar.water, bar.earth, bar.air];

        // need to rewrite to look better
        if *elements_to_comapre.iter().max().unwrap() == bar.fire {
            element = ElementType::Fire;
        }
        else if *elements_to_comapre.iter().max().unwrap() == bar.water {
            element = ElementType::Water;
        }
        else if *elements_to_comapre.iter().max().unwrap() == bar.earth {
            element = ElementType::Earth;
        }
        else {
            element = ElementType::Air;
        }

        // sub-element, cannot directly cast
        if bar.fire > 0 && bar.water > 0 {
            element = ElementType::Steam;
        }

        let color = {
            match element {
                ElementType::Fire => Color::srgb(2.5, 1.25, 1.0),
                ElementType::Water => Color::srgb(1.0, 1.5, 2.5),
                ElementType::Earth => Color::srgb(2.5, 1.25, 1.25),
                ElementType::Air => Color::srgb(1.5, 2.0, 1.5),
                ElementType::Steam => Color::srgb(1.5, 2.0, 1.5),
            }
        };

        if let Ok(wand_transform) = wand_query.get_single() {

            if bar.water == 1 
            && bar.earth > 1
            && bar.fire <= 0
            && bar.air <= 0 {
                ev_spawn_shield.send(SpawnShieldEvent { duration: bar.earth as f32 * 2. });
            }

            if bar.fire == bar.water
            && bar.water == bar.earth
            && bar.earth == bar.air 
            && bar.air == bar.fire {
                spell_desc += "black hole\n";
            }

            if bar.fire > 0 && bar.earth <= 0 && bar.air <= 0 {
                spell_desc += "fire element\n";
                
                let offset = PI/12.0;
                for _i in 0..bar.fire*3 {
                    let dir = (mouse_coords.0 - wand_transform.translation.truncate()).normalize_or_zero();
                    let angle = dir.y.atan2(dir.x) + rng.gen_range(-offset..offset);

                    ev_spawn_projectile.send(SpawnProjectileEvent {
                        texture_path: "textures/small_fire.png".to_string(),
                        color,
                        translation: wand_transform.translation,
                        angle,
                        radius: 6.,
                        speed: 150.0 + rng.gen_range(-25.0..25.0),
                        damage: dmg / bar.fire as u32,
                        element,
                        is_friendly: true,
                    });
                }
            }
        
            if bar.water > 0 && bar.earth <= 0 && bar.air <= 0 {
                spell_desc += "water element\n";

                let offset = PI/12.0;
                for _i in 0..bar.water*3 {

                    let dir = (mouse_coords.0 - wand_transform.translation.truncate()).normalize_or_zero();
                    let angle = dir.y.atan2(dir.x) + rng.gen_range(-offset..offset);

                    ev_spawn_projectile.send(SpawnProjectileEvent {
                        texture_path: "textures/small_fire.png".to_string(),
                        color,
                        translation: wand_transform.translation,
                        angle,
                        radius: 6.,
                        speed: 150.0 + rng.gen_range(-25.0..25.0),
                        damage: dmg / bar.water as u32,
                        element,
                        is_friendly: true,
                    });
                }
            }
        
            if bar.earth > 0
            && bar.air <= 0
            && (bar.water >= bar.earth || bar.water <= 0) {
                spell_desc += "AoE, e.g. earthquake\n";
    
                let offset = (2.0*PI)/(bar.len()*3) as f32;
                for i in 0..bar.len()*3 {

                    let angle = offset * i as f32;

                    ev_spawn_projectile.send(SpawnProjectileEvent {
                        texture_path: "textures/earthquake.png".to_string(),
                        color,
                        translation: wand_transform.translation,
                        angle,
                        radius: 12.,
                        speed: 100.0,
                        damage: dmg,
                        element,
                        is_friendly: true,
                    });
                }
            }
        
            if bar.air > 0 {
                spell_desc += "throwable, e.g. fireball\n";
    
                let dir = (mouse_coords.0 - wand_transform.translation.truncate()).normalize_or_zero();
                let angle = dir.y.atan2(dir.x);

                ev_spawn_projectile.send(SpawnProjectileEvent {
                    texture_path: "textures/fireball.png".to_string(),
                    color,
                    translation: wand_transform.translation,
                    angle,
                    radius: 8.0,
                    speed: 150.,
                    damage: dmg,
                    element,
                    is_friendly: true,
                });
            }
        }

        // println!("[{}] ({} DMG)", spell_desc, dmg);
        // println!("{:?}", bar.bar);

        bar.clear();
    }
}