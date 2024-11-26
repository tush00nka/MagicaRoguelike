use std::f32::consts::PI;

use bevy::prelude::*;
use rand::{distributions::Standard, prelude::Distribution, Rng};

use crate::{
    audio::PlayAudioEvent, black_hole::SpawnBlackHoleEvent, blank_spell::SpawnBlankEvent, health::Health, item::{ItemPickupAnimation, ItemType}, mobs::{MobSpawnEvent, MobType}, mouse_position::MouseCoords, player::{Player, PlayerDeathEvent, PlayerStats}, projectile::SpawnProjectileEvent, shield_spell::SpawnShieldEvent, ui::ItemInventory, wand::Wand, GameState
};

pub struct ElementsPlugin;

impl Plugin for ElementsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<ElementBarFilled>()
            .add_event::<ElementBarClear>()
            .add_event::<CastSpellEvent>()
            .insert_resource(ElementBar::default())
            .insert_resource(SpellPool::default())
            .add_systems(OnExit(GameState::MainMenu), init_spells)
            .add_systems(Update, (fill_bar, handle_recipe, cast_spell)
                .run_if(in_state(GameState::InGame)
                .or_else(in_state(GameState::Hub))));
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

    pub fn audio(&self) -> &str {
        match self {
            ElementType::Fire => "fire.ogg",
            ElementType::Water => "water.ogg",
            ElementType::Earth => "earth.ogg",
            ElementType::Air => "air.ogg",
            ElementType::Steam => "air.ogg"
        }
    }
}

#[derive(PartialEq, Clone, Copy, PartialOrd, Ord, Eq, Debug)]
pub enum Spell {
    Fire,
    Water,
    Earth,
    Air,
    Steam,
    Shield,
    BlackHole,
    Blank,
    FireElemental,
    WaterElemental,
    EarthElemental,
    AirElemental
}

#[derive(Resource)]
pub struct SpellPool {
    pub unlocked: Vec<Spell>,
}

impl SpellPool {
    fn is_unlocked(&self, spell: Spell) -> bool {
        self.unlocked.contains(&spell)
    }

    pub fn unlock(&mut self, spell: Spell) {
        if !self.unlocked.contains(&spell) {
            self.unlocked.push(spell);
        }
    }
}

impl Default for SpellPool {
    fn default() -> Self {
        Self {
            unlocked: vec![
                Spell::Fire,
                Spell::Water,
                Spell::Earth,
                Spell::Air
            ]
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

fn init_spells(
    mut commands: Commands,
) {
    commands.insert_resource(ElementBar::default());
    commands.insert_resource(SpellPool::default());
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

fn handle_recipe(
    player_stats: Res<PlayerStats>,

    wand_query: Query<&Transform, With<Wand>>,
    
    spell_pool: Res<SpellPool>,

    mut player_query: Query<(&mut Health, Entity, &Transform), (With<Player>, Without<ItemPickupAnimation>)>,
    mut ev_death: EventWriter<PlayerDeathEvent>,
    
    mut element_bar: ResMut<ElementBar>,
    mut ev_bar_clear: EventWriter<ElementBarClear>,

    mut ev_cast_spell: EventWriter<CastSpellEvent>,

    mouse: Res<ButtonInput<MouseButton>>,

    time: Res<Time<Virtual>>,
) {
    if mouse.just_pressed(MouseButton::Left) && element_bar.len() > 0 && !time.is_paused() {

        let Ok((mut player_health, player_e, transform)) = player_query.get_single_mut() else {
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

        let mut dmg = player_stats.get_bonused_damage(element);
        dmg *= bar.len() as u32;

        println!("{:?}", spell_pool.unlocked);

        if let Ok(wand_transform) = wand_query.get_single() {

            if spell_pool.is_unlocked(Spell::Shield)
            && bar.water == 1 
            && bar.earth > 1
            && bar.fire <= 0
            && bar.air <= 0 {
                ev_spawn_shield.send(SpawnShieldEvent {
                    duration: bar.earth as f32 * 2.,
                    owner: player_e,
                    is_friendly: true,
                    size: 32,
                });

                return;
            }

            if spell_pool.is_unlocked(Spell::Blank)
            && bar.water == 1
            && bar.air > 1
            && bar.fire <= 0
            && bar.earth <= 0 {
                ev_cast_spell.send(CastSpellEvent {
                    spell: Spell::Blank,
                    element,
                    origin: transform.translation,
                    damage: dmg,
                    bar
                });

                return;
            }

            if spell_pool.is_unlocked(Spell::BlackHole)
            && bar.fire == bar.water
            && bar.water == bar.earth
            && bar.earth == bar.air 
            && bar.air == bar.fire {

                ev_cast_spell.send(CastSpellEvent {
                    spell: Spell::BlackHole,
                    element,
                    origin: transform.translation,
                    damage: dmg,
                    bar
                });

                return;
            }
            
            //spawn ClayGolem -- TODO: prolly delete? as we agreed golem to be a regualr enemy
            // as we already have earth elemental???
            // ---
            // if bar.earth == 2 
            // && bar.air <= 0 
            // && bar.water >=2 
            // && bar.fire >=2 {
            //     ev_spawn_friend.send(MobSpawnEvent{mob_type: MobType::ClayGolem, pos: mouse_coords.0, is_friendly: true });
            //     return;
            // }

            //spawn FireElemental
            if spell_pool.is_unlocked(Spell::FireElemental)
            && bar.earth >= 1 
            && bar.air <= 0 
            && bar.water >=1 
            && bar.fire == 2 {                
                ev_cast_spell.send(CastSpellEvent {
                    spell: Spell::FireElemental,
                    element,
                    origin: Vec3::ZERO,
                    damage: dmg,
                    bar
                });

                return;
            }
            
            if spell_pool.is_unlocked(Spell::WaterElemental)
            && bar.earth >= 1 
            && bar.air <= 0 
            && bar.water == 2 
            && bar.fire >= 1 {
                ev_cast_spell.send(CastSpellEvent {
                    spell: Spell::WaterElemental,
                    element,
                    origin: Vec3::ZERO,
                    damage: dmg,
                    bar
                });

                return;
            }

            //spawn EarthElemental
            if spell_pool.is_unlocked(Spell::EarthElemental)
            && bar.earth == 2 
            && bar.air <= 0 
            && bar.water >=1 
            && bar.fire >=1 {

                ev_cast_spell.send(CastSpellEvent {
                    spell: Spell::EarthElemental,
                    element,
                    origin: Vec3::ZERO,
                    damage: dmg,
                    bar
                });

                return;
            }

            //spawn AirElemental
            if spell_pool.is_unlocked(Spell::AirElemental)
            && bar.earth <= 0 
            && bar.air == 2 
            && bar.water >= 1 
            && bar.fire >=1 {

                ev_cast_spell.send(CastSpellEvent {
                    spell: Spell::AirElemental,
                    element,
                    origin: Vec3::ZERO,
                    damage: dmg,
                    bar
                });

                return;
            }

            // sub-element, cannot directly cast
            if bar.fire > 0 && bar.water > 0
            && (bar.earth + bar.air) < (bar.fire + bar.water)
            && spell_pool.is_unlocked(Spell::Steam) {
                element = ElementType::Steam;

                ev_cast_spell.send(CastSpellEvent {
                    spell: Spell::Steam,
                    element,
                    origin: wand_transform.translation,
                    damage: dmg,
                    bar
                });

                return;
            }

            if bar.fire > bar.water && bar.earth <= 0 && bar.air <= 0 {   
                ev_cast_spell.send(CastSpellEvent {
                    spell: Spell::Fire,
                    element,
                    origin: wand_transform.translation,
                    damage: dmg,
                    bar
                });

                return;
            }
        
            if bar.water > bar.fire && bar.earth <= 0 && bar.air <= 0 {

                ev_cast_spell.send(CastSpellEvent {
                    spell: Spell::Water,
                    element,
                    origin: wand_transform.translation,
                    damage: dmg,
                    bar
                });

                return;
            }
        
            if bar.earth > 0
            && bar.air <= 0 {    

                ev_cast_spell.send(CastSpellEvent {
                    spell: Spell::Earth,
                    element,
                    origin: wand_transform.translation,
                    damage: dmg,
                    bar
                });

                return;
            }
        
            if bar.air > 0 {    

                ev_cast_spell.send(CastSpellEvent {
                    spell: Spell::Air,
                    element,
                    origin: wand_transform.translation,
                    damage: dmg,
                    bar
                });

                return;
            }
        }
    }
}

#[derive(Event)]
struct CastSpellEvent {
    spell: Spell,
    element: ElementType,
    origin: Vec3,
    damage: u32,
    bar: ElementBar,
}

fn cast_spell(
    mut ev_cast_spell: EventReader<CastSpellEvent>,

    mut ev_spawn_shield: EventWriter<SpawnShieldEvent>,
    mut ev_spawn_blank: EventWriter<SpawnBlankEvent>,
    mut ev_spawn_black_hole: EventWriter<SpawnBlackHoleEvent>,
    mut ev_spawn_projectile: EventWriter<SpawnProjectileEvent>,
    mut ev_spawn_friend: EventWriter<MobSpawnEvent>,

    mut ev_play_audio: EventWriter<PlayAudioEvent>,

    mouse_coords: Res<MouseCoords>,
    inventory: Res<ItemInventory>,
) {
    for ev in ev_cast_spell.read() {
        let bar = ev.bar;

        let element = ev.element;
        let color = element.color();
        let origin = ev.origin;
        let dmg = ev.damage;

        let audio_file = element.audio();
        ev_play_audio.send(PlayAudioEvent::from_file(audio_file));

        let mut rng = rand::thread_rng();

        match ev.spell {
            Spell::Fire => {
                let offset = PI/10.0;
                for _i in 0..bar.fire*3 {
                    let dir = (mouse_coords.0 - origin.truncate()).normalize_or_zero();
                    let angle = dir.y.atan2(dir.x) + rng.gen_range(-offset..offset);

                    let radius = 64.;
                    let counter_clockwise = rand::thread_rng().gen_bool(0.5);

                    let pivot = if counter_clockwise {
                        origin.truncate() + Vec2::from_angle(angle + PI/2.) * radius
                    } else {
                        origin.truncate() - Vec2::from_angle(angle + PI/2.) * radius
                    };

                    ev_spawn_projectile.send(SpawnProjectileEvent {
                        texture_path: "textures/small_fire.png".to_string(),
                        color,
                        translation: origin,
                        angle,
                        collider_radius: 6.,
                        // speed: 150.0 + rng.gen_range(-25.0..25.0),
                        speed: 2.5,
                        damage: dmg / bar.fire as u32,
                        element,
                        is_friendly: true,
                        trajectory: crate::projectile::Trajectory::Radial { radius, pivot, counter_clockwise },
                    });
                }
            },
            Spell::Water => {
                let offset = PI/12.0;
                for _i in 0..bar.water*3 {

                    let dir = (mouse_coords.0 - origin.truncate()).normalize_or_zero();
                    let angle = dir.y.atan2(dir.x) + rng.gen_range(-offset..offset);

                    ev_spawn_projectile.send(SpawnProjectileEvent {
                        texture_path: "textures/small_fire.png".to_string(),
                        color,
                        translation: origin,
                        angle,
                        collider_radius: 6.,
                        speed: 200.0 + rng.gen_range(-25.0..25.0),
                        damage: dmg / bar.water as u32,
                        element,
                        is_friendly: true,
                        trajectory: crate::projectile::Trajectory::Straight,
                    });
                }
            },
            Spell::Earth => {
                let offset = (2.0*PI)/(bar.len()*3) as f32;
                for i in 0..bar.len()*3 {

                    let angle = offset * i as f32;

                    ev_spawn_projectile.send(SpawnProjectileEvent {
                        texture_path: "textures/earthquake.png".to_string(),
                        color,
                        translation: origin,
                        angle,
                        collider_radius: 12.,
                        speed: 100.0,
                        damage: dmg,
                        element,
                        is_friendly: true,
                        trajectory: crate::projectile::Trajectory::Straight,
                    });
                }
            },
            Spell::Air => {
                let dir = (mouse_coords.0 - origin.truncate()).normalize_or_zero();
                let angle = dir.y.atan2(dir.x);

                ev_spawn_projectile.send(SpawnProjectileEvent {
                    texture_path: "textures/fireball.png".to_string(),
                    color,
                    translation: origin,
                    angle,
                    collider_radius: 8.0,
                    speed: 100.,
                    damage: dmg,
                    element,
                    is_friendly: true,
                    trajectory: crate::projectile::Trajectory::Straight,
                });
            },
            Spell::Steam => {
                let offset = PI/10.0;
                for _i in 0..(bar.fire+bar.water)*3 {
                    let dir = (mouse_coords.0 - origin.truncate()).normalize_or_zero();
                    let angle = dir.y.atan2(dir.x) + rng.gen_range(-offset..offset);

                    ev_spawn_projectile.send(SpawnProjectileEvent {
                        texture_path: "textures/small_fire.png".to_string(),
                        color,
                        translation: origin,
                        angle,
                        collider_radius: 6.,
                        speed: 150.0 + rng.gen_range(-25.0..25.0),
                        damage: dmg / (bar.fire+bar.water) as u32,
                        element,
                        is_friendly: true,
                        trajectory: crate::projectile::Trajectory::Straight,
                    });
                }
            },
            Spell::Shield => {
                ev_spawn_shield.send(SpawnShieldEvent {
                    duration: bar.earth as f32 * 2. + *inventory.amount_of_item(ItemType::Shield) as f32
                });
            },
            Spell::BlackHole => {
                ev_spawn_black_hole.send(SpawnBlackHoleEvent {
                    spawn_pos: origin.with_z(0.9),
                    target_pos: mouse_coords.0.extend(0.9),
                    lifetime: 1.5 * bar.len() as f32 + *inventory.amount_of_item(ItemType::ElementWheel) as f32, // seconds
                    strength: 1_000. * bar.len() as f32,
                });
            },
            Spell::Blank => {
                ev_spawn_blank.send(SpawnBlankEvent {
                    range: bar.air as f32 * 2. + *inventory.amount_of_item(ItemType::Blank) as f32,
                    position: origin,
                    speed: 10.0,
                    is_friendly: true,
                });
            },
            Spell::FireElemental => {
                ev_spawn_friend.send(MobSpawnEvent{mob_type: MobType::FireElemental, pos: mouse_coords.0, is_friendly: true });
            },
            Spell::WaterElemental => {
                ev_spawn_friend.send(MobSpawnEvent{mob_type: MobType::WaterElemental, pos: mouse_coords.0, is_friendly: true });
            },
            Spell::EarthElemental => {
                ev_spawn_friend.send(MobSpawnEvent{mob_type: MobType::EarthElemental, pos: mouse_coords.0, is_friendly: true });
            },
            Spell::AirElemental => {
                ev_spawn_friend.send(MobSpawnEvent{mob_type: MobType::AirElemental, pos: mouse_coords.0, is_friendly: true });
            },
        }
    }
}