use std::f32::consts::PI;
use std::time::Duration;

use bevy::prelude::*;
use rand::Rng;
use seldom_state::trigger::Done;

use std::convert::TryFrom;

use crate::alert::SpawnAlertEvent;
use crate::blank_spell::SpawnBlankEvent;
use crate::health::Health;
use crate::shield_spell::SpawnShieldEvent;
use crate::{
    elements::ElementType,
    gamemap::{ROOM_SIZE, TILE_SIZE},
    player::Player,
    projectile::SpawnProjectileEvent,
};

use super::SecondPhase;
use super::SummonQueue;
use super::{MobSpawnEvent, MobType};
//use super::ThirdPhase;
use super::FirstPhase;

pub struct BossBehavoiurPlugin;

impl Plugin for BossBehavoiurPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                tick_cooldown_boss,
                recalculate_weights,
                cast_shield,
                cast_blank,
                warn_player_abt_attack,
                perform_attack,
                tick_every_spell_cooldown,
            ),
        );
    }
}
#[derive(Component, Clone)]
pub struct BossAttackFlagComp {
    pub attack_picked: BossAttackType,
}
#[derive(Component, Clone)]
pub struct OnCooldownFlag;

#[derive(Component, Clone)]
pub struct PickAttackFlag;

#[derive(Component)]
pub struct BossAttackSystem {
    pub weight_array: Vec<i16>,
    pub cooldown_array: Vec<Timer>,
    pub cooldown_between_attacks: Timer,
    pub cooldown_mask: u32,
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
#[derive(PartialEq, Clone)]
#[repr(u8)]
pub enum BossAttackType {
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

fn pick_direction(player_pos: Vec3, boss_pos: Vec3) -> Vec2 {
    let direction = (boss_pos - player_pos).truncate();
    let mut vec_dirs = vec![[0, 0], [0, 1], [0, 2], [0, 3]]; //1st - right 2nd - left 3-up 4 - down
    if direction.x > 0. {
        vec_dirs[1][0] = 20;
        vec_dirs[0][0] = 40;
        if direction.y > 0. {
            vec_dirs[2][0] = 40;
            vec_dirs[3][0] = 20;
        } else {
            vec_dirs[2][0] = 20;
            vec_dirs[3][0] = 40;
        }
    } else {
        vec_dirs[1][0] = 40;
        vec_dirs[0][0] = 20;
        if direction.y > 0. {
            vec_dirs[2][0] = 40;
            vec_dirs[3][0] = 20;
        } else {
            vec_dirs[2][0] = 20;
            vec_dirs[3][0] = 40;
        }
    }

    vec_dirs[0][0] *= rand::thread_rng().gen_range(0..100);
    vec_dirs[1][0] *= rand::thread_rng().gen_range(0..100);
    vec_dirs[2][0] *= rand::thread_rng().gen_range(0..100);
    vec_dirs[3][0] *= rand::thread_rng().gen_range(0..100);

    vec_dirs.sort_unstable_by(|a, b| b[0].cmp(&a[0]));
    return WALL_DIRECTIONS[vec_dirs[0][1]];
}

fn perform_attack(
    mut ev_spawn_projectile: EventWriter<SpawnProjectileEvent>,
    boss_query: Query<
        (Entity, &BossAttackSystem, &BossAttackFlagComp, &Transform),
        Without<BeforeAttackDelayBoss>,
    >,
    player_query: Query<&Transform, With<Player>>,
    mut ev_mob_spawn: EventWriter<MobSpawnEvent>,
    mut commands: Commands,
    //    phase3_query: Query<&ThirdPhase>,
) {
    let Ok((boss_e, _boss_sys, attack_type, boss_position)) = boss_query.get_single() else {
        return;
    };

    let Ok(player_pos) = player_query.get_single() else {
        println!("NO PLAYER?????");
        return;
    };

    let element: ElementType = rand::random();
    let mut amount_attack = 0;

    match attack_type.attack_picked {
        BossAttackType::Wall => {
            println!("wall");
            let to_skip = rand::thread_rng().gen_range((ROOM_SIZE / 2 - 7)..(ROOM_SIZE / 2 + 8));

            let direction = pick_direction(player_pos.translation, boss_position.translation);

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
                    radius: 1.0,
                    speed: 75.0,
                    damage: 20,
                    element,
                    is_friendly: false,
                });
            }
        }

        BossAttackType::Radial => {
            println!("radial");
            let amount_attack = rand::thread_rng().gen_range(8..16);
            let radius = rand::thread_rng().gen_range(500..800);

            let offset = 2.0 * PI / (amount_attack as f32);

            let to_skip = rand::thread_rng().gen_range(0..amount_attack);

            for i in 0..amount_attack {
                if i == to_skip {
                    continue;
                }

                let direction = -Vec2::from_angle(i as f32 * offset);
                let position = (player_pos.translation.truncate()
                    - direction * (radius as f32) / 10.)
                    .extend(1.0);

                ev_spawn_projectile.send(SpawnProjectileEvent {
                    texture_path: "textures/fireball.png".to_string(),
                    color: element.color(),
                    translation: position,
                    angle: direction.to_angle(),
                    radius: 1.0,
                    speed: 50.0,
                    damage: 20,
                    element,
                    is_friendly: false,
                });
            }
        }
        BossAttackType::Blank => {
            println!("how");
        }

        BossAttackType::Shield => {
            println!("shield");
        }
        BossAttackType::FastPierce => {
            println!("fast pierce");
            amount_attack += 2;
            let angle_disp = PI / (2 + amount_attack) as f32;
            let mut angle = (boss_position.translation - player_pos.translation)
                .truncate()
                .to_angle()
                - angle_disp;
            for i in 0..amount_attack {
                ev_spawn_projectile.send(SpawnProjectileEvent {
                    texture_path: "textures/fireball.png".to_string(),
                    color: element.color(),
                    translation: boss_position.translation,
                    angle: angle,
                    radius: 1.0,
                    speed: 50.0,
                    damage: 20,
                    element,
                    is_friendly: false,
                });
                angle += angle_disp;
            }
        }

        BossAttackType::SpawnAirElemental => {
            println!("air");
            amount_attack += 6;
            let mut position_drift = -64.;
            for _ in 0..amount_attack {
                ev_mob_spawn.send(MobSpawnEvent {
                    mob_type: MobType::AirElemental,
                    pos: Vec2::new(
                        boss_position.translation.x + position_drift,
                        boss_position.translation.y + 32.,
                    ),
                    is_friendly: false,
                    loot: None,
                    owner: Some(boss_e),
                    exp_amount: 0,
                });

                position_drift += 16.;
            }
        }

        BossAttackType::SpawnClayGolem => {
            println!("golem");
            amount_attack += 2;
            let mut position_drift = -64.;
            for _ in 0..amount_attack {
                ev_mob_spawn.send(MobSpawnEvent {
                    mob_type: MobType::ClayGolem,
                    pos: Vec2::new(
                        boss_position.translation.x + position_drift,
                        boss_position.translation.y + 32.,
                    ),
                    is_friendly: false,
                    loot: None,
                    owner: Some(boss_e),
                    exp_amount: 0,
                });

                position_drift += 128.;
            }
        }
        BossAttackType::SpawnEarthElemental => {
            println!("earth");
        }
        BossAttackType::SpawnWaterElemental => {
            println!("water");
            amount_attack += 3;
            let mut position_drift = -64.;
            for _ in 0..amount_attack {
                ev_mob_spawn.send(MobSpawnEvent {
                    mob_type: MobType::WaterElemental,
                    pos: Vec2::new(
                        boss_position.translation.x + position_drift,
                        boss_position.translation.y + 32.,
                    ),
                    is_friendly: false,
                    loot: None,
                    owner: Some(boss_e),
                    exp_amount: 0,
                });

                position_drift += 64.;
            }
        }
        BossAttackType::ProjectilePattern => {
            println!("pattern");
        }
        BossAttackType::MegaStan => {
            println!("megastan");
        }
        BossAttackType::SpawnFireElemental => {
            println!("fire");
            amount_attack += 4;
            let radius = 64.;
            let mut angle = PI / 4.;

            for _ in 0..amount_attack {
                ev_mob_spawn.send(MobSpawnEvent {
                    mob_type: MobType::FireElemental,
                    pos: Vec2::new(
                        player_pos.translation.x + radius / angle.cos(),
                        player_pos.translation.y + radius * angle.tan(),
                    ),
                    is_friendly: false,
                    loot: None,
                    owner: Some(boss_e),
                    exp_amount: 0,
                });
                angle += PI / 2.;
            }
        }
        _ => {
            println!("what");
        }
    }

    commands.entity(boss_e).insert(Done::Success);
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
    let Ok((boss_e, mut attack_system, summon_list, boss_hp, boss_transform)) =
        boss_query.get_single_mut()
    else {
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

        if attack_system.cooldown_mask & (1 << i) == 0 {
            attack_system.weight_array[i] = -10000;
            continue;
        }
        let attack_flag;
        let mut mob_spawn = MobType::Mossling;

        match BossAttackType::try_from(i).unwrap() {
            BossAttackType::SpawnEarthElemental => {
                base_weight += (phase == 3) as i16 * i16::MIN;

                mob_spawn = MobType::EarthElemental;
                attack_flag = BossAttackFlag::SpawnSpells;
            }
            BossAttackType::SpawnAirElemental => {
                base_weight += (phase != 1) as i16 * i16::MIN;

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
                base_weight += (phase == 1) as i16 * i16::MIN; //add bonus when player near walls
                attack_flag = BossAttackFlag::ProjectileSpells;
            }
            BossAttackType::MegaStan => {
                base_weight += (phase <= 2) as i16 * i16::MIN;
                attack_flag = BossAttackFlag::ProjectileSpells; //dd bonus when player far away from walls
            }
            BossAttackType::FastPierce => {
                base_weight += (phase == 1) as i16 * i16::MIN;
                attack_flag = BossAttackFlag::ProjectileSpells; // add bonus when player far from boss
            }

            BossAttackType::ProjectilePattern => {
                base_weight += (phase != 3) as i16 * i16::MIN; //add bonus when player far away from walls
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

pub fn tick_cooldown_boss(
    mut commands: Commands,
    mut attack_timers: Query<(Entity, &mut BossAttackSystem), With<OnCooldownFlag>>,
    time: Res<Time>,
) {
    //add on cooldown state, so we don't tick timer during attack
    let Ok((boss_e, mut attack_system)) = attack_timers.get_single_mut() else {
        return;
    };

    attack_system.cooldown_between_attacks.tick(time.delta());

    if attack_system.cooldown_between_attacks.just_finished() {
        commands.entity(boss_e).insert(Done::Success);
    }
}

pub fn tick_every_spell_cooldown(mut attack_timers: Query<&mut BossAttackSystem>, time: Res<Time>) {
    let Ok(mut attack_system) = attack_timers.get_single_mut() else {
        return;
    };

    for i in 0..attack_system.cooldown_array.len() {
        if attack_system.cooldown_mask & (1 << i) != 1 {
            attack_system.cooldown_array[i].tick(time.delta());

            if attack_system.cooldown_array[i].just_finished() {
                attack_system.cooldown_mask |= 1 << i;
                println!("flags: {:#018b}", attack_system.cooldown_mask);
            }
        }
    }
}

pub fn cast_blank(
    mut spawn_blank_ev: EventWriter<SpawnBlankEvent>,
    mut boss_query: Query<(&mut BossAttackSystem, &Transform)>,
) {
    let Ok((mut attack_system, pos)) = boss_query.get_single_mut() else {
        println!("no attack system to cast shield");
        return;
    };

    if attack_system.weight_array[BossAttackType::Blank as usize] > 2500 {
        attack_system.cooldown_mask ^= 1 << BossAttackType::Blank as usize;
        spawn_blank_ev.send(SpawnBlankEvent {
            range: 100.,
            position: pos.translation,
            speed: 10.,
            side: false,
        });
    }
}

pub fn cast_shield(
    mut boss_query: Query<(Entity, &mut BossAttackSystem)>,
    mut cast_shield: EventWriter<SpawnShieldEvent>,
) {
    let Ok((boss_e, mut attack_system)) = boss_query.get_single_mut() else {
        println!("no attack system to cast shield");
        return;
    };

    if attack_system.weight_array[BossAttackType::Shield as usize] > 2500 {
        attack_system.cooldown_mask ^= 1 << BossAttackType::Shield as usize;
        cast_shield.send(SpawnShieldEvent {
            duration: 4.,
            owner: boss_e,
            is_friendly: false,
            size: 64,
        });
    }
    //when weight overcomes certain value - cast and cooldown
}
//
#[derive(Component, Clone)]
pub struct BeforeAttackDelayBoss {
    timer: Timer,
    check: bool,
}
impl Default for BeforeAttackDelayBoss {
    fn default() -> Self {
        Self {
            timer: Timer::new(Duration::from_millis(450), TimerMode::Repeating),
            check: true,
        }
    }
}
pub fn warn_player_abt_attack(
    time: Res<Time>,
    mut boss_query: Query<(
        Entity,
        &mut BossAttackSystem,
        &BossAttackFlagComp,
        &mut BeforeAttackDelayBoss,
        &Transform,
    )>,
    mut commands: Commands,
    mut ev_spawn_alert: EventWriter<SpawnAlertEvent>,
) {
    let Ok((boss_e, mut boss, attack_flag, mut delay, pos)) = boss_query.get_single_mut() else {
        return;
    };
    delay.timer.tick(time.delta());
    if delay.check {
        ev_spawn_alert.send(SpawnAlertEvent {
            position: pos.translation.truncate().with_y(pos.translation.y + 24.),
            attack_alert: true,
        });
        //spawn marker
        delay.check = false;
    }
    if delay.timer.just_finished() {
        boss.cooldown_mask ^= 1 << attack_flag.attack_picked.clone() as usize;
        commands.entity(boss_e).remove::<BeforeAttackDelayBoss>();
    }
}
pub fn pick_attack_to_perform_koldun(
    In(entity): In<Entity>,
    attack_system: Query<&BossAttackSystem>,
) -> Option<Option<BossAttackType>> {
    let Ok(attack_system) = attack_system.get(entity) else {
        println!("No attacks system?");
        return None;
    };
    let mut pick_1 = 0;
    let mut pick_2 = -1;

    let mut largest_value = attack_system.weight_array[0];

    for i in 0..attack_system.weight_array.len() {
        if attack_system.weight_array[i] > largest_value {
            largest_value = attack_system.weight_array[i];

            pick_2 = pick_1;
            pick_1 = i as i16;
        }
    }

    let chance_to_pick = rand::thread_rng().gen_range(0.0..1.0);

    if chance_to_pick >= 0.65 && pick_2 > 0 {
        return Some(Some(BossAttackType::try_from(pick_2 as usize).unwrap()));
    }

    if largest_value < 0 {
        println!("ERROR VALUE");
        return None;
    }

    return Some(Some(BossAttackType::try_from(pick_1 as usize).unwrap()));
    //pick with random attack including weights, like idk, use coeff or smth
}
