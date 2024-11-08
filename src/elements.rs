use std::f32::consts::PI;

use bevy::prelude::*;
use rand::{distributions::Standard, prelude::Distribution, Rng};

use crate::{
    black_hole::SpawnBlackHoleEvent, blank_spell::SpawnBlankEvent, health::Health, item::ItemPickupAnimation, mouse_position::MouseCoords, player::{Player, PlayerDeathEvent, PlayerStats}, projectile::SpawnProjectileEvent, shield_spell::SpawnShieldEvent, wand::Wand, GameState
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

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ElementType {
    Fire,
    Water,
    Earth,
    Air,
    Steam,
}

impl Distribution<ElementType> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ElementType {
        match rng.gen_range(0..5) {
            0 => ElementType::Fire,
            1 => ElementType::Water,
            2 => ElementType::Earth,
            3 => ElementType::Air,
            4 => ElementType::Steam,
            _ => ElementType::Fire,
        }
    }
}

impl ElementType {
    pub fn color(&self) -> Color {
    match self {
            ElementType::Fire => Color::srgb(2.5, 1.25, 1.0),
            ElementType::Water => Color::srgb(1.0, 1.5, 2.5),
            ElementType::Earth => Color::srgb(0.45, 0.15, 0.15),
            ElementType::Air => Color::srgb(1.5, 2.0, 1.5),
            ElementType::Steam => Color::srgb(1.5, 2.0, 1.5)
        }
    }
}

#[derive(Resource, Copy, Clone)]
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

#[derive(Component)]
pub struct ElementResistance {
    //resistance component, applies any amount of elementres to entity
    pub elements: Vec<ElementType>,
    pub resistance_percent: Vec<i16>, // earth wind fire water steam
}

impl ElementResistance {
    pub fn calculate_for(&self, damage: &mut i32, damage_element: Option<ElementType>) {
        if damage_element.is_some() {
            if self.elements.contains(&damage_element.unwrap()) {
                *damage = (*damage as f32 * (1. - self.resistance_percent[damage_element.unwrap() as usize] as f32 / 100.)) as i32;
            }
        }
    }

    pub fn add(&mut self, element: ElementType, percent: i16) {
        if !self.elements.contains(&element) {
            self.elements.push(element);
        }

        self.resistance_percent[element as usize] += percent;
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
    time: Res<Time<Virtual>>,
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

        if new_element.is_some() && bar.len() < bar.max && !time.is_paused() {
            ev_bar_filled.send(ElementBarFilled(new_element.unwrap()));
            bar.add(new_element.unwrap());
        }

    });
}

fn cast_spell(
    mouse_coords: Res<MouseCoords>,
    player_stats: Res<PlayerStats>,

    wand_query: Query<&Transform, With<Wand>>,
    
    mut player_query: Query<(&mut Health, Entity), (With<Player>, Without<ItemPickupAnimation>)>,
    mut ev_death: EventWriter<PlayerDeathEvent>,

    mut ev_spawn_shield: EventWriter<SpawnShieldEvent>,
    mut ev_spawn_blank: EventWriter<SpawnBlankEvent>,
    mut ev_spawn_black_hole: EventWriter<SpawnBlackHoleEvent>,
    mut ev_spawn_projectile: EventWriter<SpawnProjectileEvent>,

    mut element_bar: ResMut<ElementBar>,
    mut ev_bar_clear: EventWriter<ElementBarClear>,

    mouse: Res<ButtonInput<MouseButton>>,

    time: Res<Time<Virtual>>,
) {
    if mouse.just_pressed(MouseButton::Left) && element_bar.len() > 0 && !time.is_paused() {

        let Ok((mut player_health, player_e)) = player_query.get_single_mut() else {
            return;
        };

        // отнимаем хп, если предмет
        if player_stats.spell_cast_hp_fee > 0 {
            player_health.damage(player_stats.spell_cast_hp_fee);
            if player_health.current <= 0 {
                ev_death.send(PlayerDeathEvent (player_e));
            }
        }

        ev_bar_clear.send(ElementBarClear);

        let bar = element_bar.clone();
        element_bar.clear();

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

        let color = element.color();

        if let Ok(wand_transform) = wand_query.get_single() {

            if bar.water == 1 
            && bar.earth > 1
            && bar.fire <= 0
            && bar.air <= 0 {
                ev_spawn_shield.send(SpawnShieldEvent {
                    duration: bar.earth as f32 * 2.
                });

                return;
            }

            if bar.water == 1
            && bar.air > 1
            && bar.fire <= 0
            && bar.earth <= 0 {
                ev_spawn_blank.send(SpawnBlankEvent {
                    range: bar.air as f32 * 2.,
                    speed: 10.0,
                });

                return;
            }

            if bar.fire == bar.water
            && bar.water == bar.earth
            && bar.earth == bar.air 
            && bar.air == bar.fire {
                ev_spawn_black_hole.send(SpawnBlackHoleEvent {
                    spawn_pos: wand_transform.translation.with_z(0.9),
                    target_pos: mouse_coords.0.extend(0.9),
                    lifetime: 1.5 * bar.len() as f32, // seconds
                    strength: 1_000. * bar.len() as f32,
                });

                return;
            }

            if bar.fire > 0 && bar.earth <= 0 && bar.air <= 0 {                
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
            && bar.air <= 0 {    
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
    }
}