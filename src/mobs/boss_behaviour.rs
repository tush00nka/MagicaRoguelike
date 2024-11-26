use std::f32::consts::PI;

use bevy::prelude::*;
use rand::Rng;

use std::convert::TryFrom;
use std::convert::TryInto;

use crate::health::Health;
use crate::{
    elements::ElementType,
    gamemap::{ROOM_SIZE, TILE_SIZE},
    player::Player,
    projectile::SpawnProjectileEvent,
};

use super::FirstPhase;
use super::MobType;
use super::SecondPhase;
use super::SummonQueue;
use super::ThirdPhase;

pub struct BossBehavoiurPlugin;

impl Plugin for BossBehavoiurPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<BossAttackEvent>()
            .add_systems(Update, (charge_attack, perform_attack));
    }
}

#[derive(Component)]
pub struct BossAttackSystem {
    pub weight_array: Vec<i16>,
    pub cooldown_array: Vec<Timer>,
    pub cooldown_between_attacks: Timer,
    pub cooldown_mask: u32,
}

#[derive(Component)]
pub struct Boss {
    pub attack_cooldown: Timer,
}

const WALL_DIRECTIONS: [Vec2; 4] = [
    Vec2 { x: 1.0, y: 0.0 },
    Vec2 { x: -1.0, y: 0.0 },
    Vec2 { x: 0.0, y: 1.0 },
    Vec2 { x: 0.0, y: -1.0 },
];

#[derive(PartialEq)]
#[repr(u8)]
pub enum BossAttackFlag {
    ProjectileSpells = 0,
    DefensiveSpells,
    SpawnSpells,
}
#[derive(PartialEq)]
#[repr(u8)]
enum BossAttackType {
    SpawnEarthElemental = 0,
    SpawnAirElemental,
    Radial,
    ProjectilePattern,
    Shield,
    SpawnFireElemental,
    SpawnWaterElemental,
    FastPierce,
    Blank,
    Wall,
    SpawnClayGolem,
    MegaStan,
}

#[derive(Event)]
struct BossAttackEvent(BossAttackType);

fn charge_attack(
    mut boss_query: Query<&mut Boss>,
    time: Res<Time>,
    mut ev_boss_attack: EventWriter<BossAttackEvent>,
) {
    let Ok(mut boss) = boss_query.get_single_mut() else {
        return;
    };

    boss.attack_cooldown.tick(time.delta());

    let mut rng = rand::thread_rng();

    if boss.attack_cooldown.just_finished() {
        match rng.gen_range(0..2) {
            0 => {
                ev_boss_attack.send(BossAttackEvent(BossAttackType::Wall));
            }
            1 => {
                ev_boss_attack.send(BossAttackEvent(BossAttackType::Radial));
            }
            _ => {}
        }
    }
}

fn perform_attack(
    mut ev_boss_attack: EventReader<BossAttackEvent>,
    mut ev_spawn_projectile: EventWriter<SpawnProjectileEvent>,

    player_query: Query<&Transform, With<Player>>,
) {
    /*    for ev in ev_boss_attack.read() {
        let element: ElementType = rand::random();

        match ev.0 {
            BossAttackType::Wall(direction) => {
                let to_skip =
                    rand::thread_rng().gen_range((ROOM_SIZE / 2 - 7)..(ROOM_SIZE / 2 + 8));

                for i in (ROOM_SIZE / 2 - 7)..(ROOM_SIZE / 2 + 8) {
                    if i == to_skip {
                        continue;
                    }

                    let position = match direction {
                        Vec2::NEG_Y => Vec3::new(
                            i as f32 * TILE_SIZE,
                            (ROOM_SIZE / 2 + 7) as f32 * TILE_SIZE,
                            1.0,
                        ),
                        Vec2::Y => Vec3::new(
                            i as f32 * TILE_SIZE,
                            (ROOM_SIZE / 2 - 7) as f32 * TILE_SIZE,
                            1.0,
                        ),
                        Vec2::NEG_X => Vec3::new(
                            (ROOM_SIZE / 2 + 7) as f32 * TILE_SIZE,
                            i as f32 * TILE_SIZE,
                            1.0,
                        ),
                        Vec2::X => Vec3::new(
                            (ROOM_SIZE / 2 - 7) as f32 * TILE_SIZE,
                            i as f32 * TILE_SIZE,
                            1.0,
                        ),
                        _ => Vec3::ZERO,
                    };

                    ev_spawn_projectile.send(SpawnProjectileEvent {
                        texture_path: "textures/earthquake.png".to_string(),
                        color: element.color(),
                        translation: position,
                        angle: direction.to_angle(),
                        collider_radius: 1.0,
                        speed: 75.0,
                        damage: 20,
                        element,
                        is_friendly: false,
                        trajectory: crate::projectile::Trajectory::Straight,
                    });
                }
            }
            BossAttackType::Radial(amount, radius) => {
                let Ok(player_transform) = player_query.get_single() else {
                    return;
                };

                let offset = 2.0 * PI / (amount as f32);

                let to_skip = rand::thread_rng().gen_range(0..amount);

                for i in 0..amount {
                    if i == to_skip {
                        continue;
                    }

                    let direction = -Vec2::from_angle(i as f32 * offset);
                    let position =
                        (player_transform.translation.truncate() - direction * radius).extend(1.0);

                    ev_spawn_projectile.send(SpawnProjectileEvent {
                        texture_path: "textures/fireball.png".to_string(),
                        color: element.color(),
                        translation: position,
                        angle: direction.to_angle(),
                        collider_radius: 1.0,
                        speed: 50.0,
                        damage: 20,
                        element,
                        is_friendly: false,
                        trajectory: crate::projectile::Trajectory::Straight,
                    });
                }
            }
        }
    } */
}

impl TryFrom<usize> for BossAttackType {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            value if value == BossAttackType::SpawnEarthElemental as usize => {
                Ok(BossAttackType::SpawnEarthElemental)
            }
            value if value == BossAttackType::SpawnAirElemental as usize => {
                Ok(BossAttackType::SpawnAirElemental)
            }
            value if value == BossAttackType::SpawnClayGolem as usize => {
                Ok(BossAttackType::SpawnClayGolem)
            }
            value if value == BossAttackType::SpawnFireElemental as usize => {
                Ok(BossAttackType::SpawnFireElemental)
            }
            value if value == BossAttackType::SpawnWaterElemental as usize => {
                Ok(BossAttackType::SpawnWaterElemental)
            }
            value if value == BossAttackType::Radial as usize => Ok(BossAttackType::Radial),
            value if value == BossAttackType::ProjectilePattern as usize => {
                Ok(BossAttackType::ProjectilePattern)
            }
            value if value == BossAttackType::Shield as usize => Ok(BossAttackType::Shield),
            value if value == BossAttackType::FastPierce as usize => Ok(BossAttackType::FastPierce),
            value if value == BossAttackType::Blank as usize => Ok(BossAttackType::Blank),
            value if value == BossAttackType::Wall as usize => Ok(BossAttackType::Wall),
            value if value == BossAttackType::MegaStan as usize => Ok(BossAttackType::MegaStan),
            _ => Err(()),
        }
    }
}
//weights depends on:
//Cooldown,
//range to the player
//phase
//boss hp
//base value
//position of player
//is there such mobs
pub fn recalculate_weights(
    mut boss_query: Query<(
        Entity,
        &mut BossAttackSystem,
        &mut SummonQueue,
        &Health,
        &Transform,
    )>,
    player_query: Query<&Transform, With<Player>>,
    phase_1: Query<&FirstPhase>,
    phase_2: Query<&SecondPhase>,
) {
    let Ok((boss_e, mut attack_system, mut summon_list, boss_hp, boss_transform)) =
        boss_query.get_single_mut()
    else {
        println!("Boss attack system error!");
        return;
    };
    let Ok(player_transform) = player_query.get_single() else {
        println!("Player died! or smth");
        return;
    };
    let phase: u8;

    if phase_1.contains(boss_e) {
        phase = 1;
    } else if phase_2.contains(boss_e) {
        phase = 2;
    } else {
        phase = 3;
    }

    for i in 0..attack_system.weight_array.len() {
        let mut base_weight: i16 = i as i16 * 100;

        if attack_system.cooldown_mask & 2u32.pow(i as u32) == 0 {
            attack_system.weight_array[i] = -10000;
            continue;
        }
        let attack_flag;
        let mut mob_spawn = MobType::Mossling;

        match BossAttackType::try_from(i).unwrap() {
            BossAttackType::SpawnEarthElemental => {
                base_weight += (phase == 3) as i16 * i16::MIN; //does there are some turrets in angles

                mob_spawn = MobType::EarthElemental;
                attack_flag = BossAttackFlag::SpawnSpells;
            }
            BossAttackType::SpawnAirElemental => {
                base_weight += (phase != 1) as i16 * i16::MIN; //if has more than 5-6 airelementals - turn to 0
                
                mob_spawn = MobType::AirElemental;
                attack_flag = BossAttackFlag::SpawnSpells;
            }
            BossAttackType::SpawnFireElemental => {
                base_weight += (phase == 3) as i16 * i16::MIN;

                mob_spawn = MobType::FireElemental;
                attack_flag = BossAttackFlag::SpawnSpells;
            }
            BossAttackType::SpawnClayGolem => {
                base_weight += (phase != 1) as i16 * i16::MIN;

                mob_spawn = MobType::ClayGolem;
                attack_flag = BossAttackFlag::SpawnSpells;
            }
            BossAttackType::SpawnWaterElemental => {
                base_weight += (phase == 3) as i16 * i16::MIN;

                mob_spawn = MobType::WaterElemental;
                attack_flag = BossAttackFlag::SpawnSpells;
            }
            BossAttackType::Radial => {
                base_weight += (phase == 1) as i16 * i16::MIN;
                attack_flag = BossAttackFlag::ProjectileSpells;
            }
            BossAttackType::Shield => {
                base_weight += (phase != 2) as i16 * i16::MIN;
                attack_flag = BossAttackFlag::DefensiveSpells;
            }
            BossAttackType::Blank => {
                base_weight += (phase != 3) as i16 * i16::MIN;
                attack_flag = BossAttackFlag::DefensiveSpells;
            }
            BossAttackType::Wall => {
                base_weight += (phase == 1) as i16 * i16::MIN;
                attack_flag = BossAttackFlag::ProjectileSpells;
            }
            BossAttackType::MegaStan => {
                base_weight += (phase <= 2) as i16 * i16::MIN;
                attack_flag = BossAttackFlag::ProjectileSpells;
            }
            BossAttackType::FastPierce => {
                base_weight += (phase == 1) as i16 * i16::MIN;
                attack_flag = BossAttackFlag::ProjectileSpells;
            }

            BossAttackType::ProjectilePattern => {
                base_weight += (phase != 3) as i16 * i16::MIN;
                attack_flag = BossAttackFlag::ProjectileSpells;
            }
        }

        if base_weight <= i16::MIN / 2 {
            attack_system.weight_array[i] = base_weight;
            continue;
        }

        let dist = player_transform
            .translation
            .distance(boss_transform.translation)
            .floor() as i16;

        match attack_flag {
            BossAttackFlag::DefensiveSpells => {
                base_weight += ((boss_hp.max - boss_hp.current) / 20 * 3) as i16
                    + dist * (dist <= 200 || dist >= 400) as i16;
            }

            BossAttackFlag::ProjectileSpells => {
                base_weight += ((boss_hp.max - boss_hp.current) / 20) as i16 + dist * 10;
                
            }

            BossAttackFlag::SpawnSpells => {

                base_weight += (summon_list.queue.len()
                    - summon_list
                        .queue
                        .iter()
                        .filter(|summon_unit| summon_unit.mob_type != MobType::Mossling)
                        .count()) as i16
                    * 100
                    - summon_list
                        .queue
                        .iter()
                        .filter(|x| x.mob_type == mob_spawn)
                        .count() as i16
                        * 100;

                base_weight += ((boss_hp.max - boss_hp.current) / 20) as i16
                    - (summon_list.amount_of_mobs * (150 * ((phase == 2) as i32 + 50))) as i16;
            }
        }

        attack_system.weight_array[i] = base_weight;
    }
    //calculate all factors, phase included once in a time like in 1 second
}
pub fn tick_cooldown_boss(mut attack_timers: Query<&mut BossAttackSystem>, time: Res<Time>) {
    let Ok(mut attack_system) = attack_timers.get_single_mut() else {
        println!("Boss attack system error!");
        return;
    };

    for i in 0..attack_system.cooldown_array.iter_mut().len() {
        if attack_system.cooldown_mask & 2u32.pow(i as u32) != 0 {
            attack_system.cooldown_array[i].tick(time.delta());

            if attack_system.cooldown_array[i].just_finished() {
                attack_system.cooldown_mask | 2u32.pow(i as u32);
            }
        }
    }
}
pub fn cast_blank() {
    //when weight overcomes certain value - cast and cooldown
}
pub fn cast_shield() {
    //when weight overcomes certain value - cast and cooldown
}
pub fn pick_attack_to_perform_koldun(In(entity): In<Entity>) -> Option<BossAttackType> {
    return Some(BossAttackType::Blank);
    //pick with random attack including weights, like idk, use coeff or smth
}

//pub fn cast_spell(In<Entity>){

//}
