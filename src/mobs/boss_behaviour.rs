use std::f32::consts::PI;
use std::time::Duration;

use bevy::prelude::*;
use rand::{thread_rng, Rng};
use seldom_state::trigger::Done;

use std::convert::TryFrom;

use crate::alert::SpawnAlertEvent;
use crate::blank_spell::SpawnBlankEvent;
use crate::gamemap::Map;
use crate::health::Health;
use crate::projectile::{Friendly, Projectile, Trajectory};
use crate::shield_spell::SpawnShieldEvent;
use crate::{
    elements::ElementType,
    gamemap::{ROOM_SIZE, TILE_SIZE},
    player::Player,
    projectile::SpawnProjectileEvent,
};

use super::{Mob, SummonUnit};
use super::{MobSpawnEvent, MobType};
use super::{PhaseManager, SummonQueue};

pub struct BossBehavoiurPlugin;

impl Plugin for BossBehavoiurPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                check_is_summon_alive,
                tick_cooldown_boss,
                recalculate_weights,
                cast_shield,
                cast_blank,
                warn_player_abt_attack,
                perform_attack,
                tick_every_spell_cooldown,
                switch_phase,
                projectiles_check,
                cast_out_of_order,
            ),
        );
    }
}

pub const STATIC_ANGLE_POINTS: &[(i32, i32)] = &[
    (ROOM_SIZE / 2 - 6, ROOM_SIZE / 2 - 6),
    (ROOM_SIZE / 2 + 6, ROOM_SIZE / 2 - 6),
    (ROOM_SIZE / 2 - 6, ROOM_SIZE / 2 + 6),
    (ROOM_SIZE / 2 + 6, ROOM_SIZE / 2 + 6),
];

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

#[derive(Component)]
pub struct OutOfOrderAttackQueue {
    queue: Vec<BossAttackType>,
    timer: Timer,
}

impl Default for OutOfOrderAttackQueue {
    fn default() -> Self {
        Self {
            queue: vec![],
            timer: Timer::new(Duration::from_millis(1000), TimerMode::Repeating),
        }
    }
}

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

fn switch_phase(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &Health,
        &mut PhaseManager,
        &MobType,
        &mut SummonQueue,
    )>,
) {
    let Ok((boss_e, health, mut phase_manager, boss_type, mut summons)) = query.get_single_mut()
    else {
        return;
    };
    if phase_manager.current_phase >= phase_manager.max_phase {
        return;
    }
    if health.current
        <= (health.max as f32
            * phase_manager.phase_change_hp_multiplier[phase_manager.current_phase as usize - 1])
            as i32
    {
        if *boss_type == MobType::Koldun {
            if phase_manager.current_phase == 1 {
                summons.resize(6);
                summons.queue.resize(
                    6,
                    SummonUnit {
                        entity: None,
                        mob_type: MobType::Mossling,
                    },
                );
                summons.amount_of_mobs = 5;
            }
        }
        commands.entity(boss_e).insert(OutOfOrderAttackQueue {
            queue: vec![
                BossAttackType::Blank,
                BossAttackType::Blank,
                BossAttackType::Blank,
            ],
            ..default()
        });
        phase_manager.current_phase += 1;
    }
}
fn cast_out_of_order(
    mut boss_query: Query<(Entity, &Transform, &mut OutOfOrderAttackQueue)>,
    time: Res<Time>,
    mut spawn_blank_ev: EventWriter<SpawnBlankEvent>,
    mut commands: Commands,
) {
    for (boss_e, pos, mut attack_queue) in boss_query.iter_mut() {
        if attack_queue.queue.len() == 0 {
            commands.entity(boss_e).remove::<OutOfOrderAttackQueue>();
            return;
        }
        attack_queue.timer.tick(time.delta());
        if attack_queue.timer.just_finished() {
            match attack_queue.queue[attack_queue.queue.len() - 1] {
                BossAttackType::Blank => {
                    spawn_blank_ev.send(SpawnBlankEvent {
                        range: 100.,
                        position: pos.translation,
                        speed: 20.,
                        is_friendly: false,
                    });
                }
                _ => {
                    println!("other")
                }
            };
            attack_queue.queue.pop();
        }
    }
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

fn get_wall_pos(direction: Vec2, i: i32) -> Vec3 {
    match direction {
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
    }
}

fn perform_attack(
    mut ev_spawn_projectile: EventWriter<SpawnProjectileEvent>,
    boss_query: Query<
        (
            Entity,
            &BossAttackSystem,
            &BossAttackFlagComp,
            &Transform,
            &PhaseManager,
        ),
        Without<BeforeAttackDelayBoss>,
    >,
    player_query: Query<&Transform, With<Player>>,
    mut ev_mob_spawn: EventWriter<MobSpawnEvent>,
    mut commands: Commands,
) {
    let Ok((boss_e, _boss_sys, attack_type, boss_position, phase_manager)) =
        boss_query.get_single()
    else {
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
            let mut second_direction =
                pick_direction(player_pos.translation, boss_position.translation);

            while second_direction == direction {
                second_direction =
                    pick_direction(player_pos.translation, boss_position.translation);
            }

            let second_to_skip =
                rand::thread_rng().gen_range((ROOM_SIZE / 2 - 7)..(ROOM_SIZE / 2 + 8));

            for i in (ROOM_SIZE / 2 - 7)..(ROOM_SIZE / 2 + 8) {
                if i == to_skip
                    || (phase_manager.current_phase == 3 && (i == to_skip - 1 || i == to_skip + 1))
                {
                    continue;
                }

                let position = get_wall_pos(direction, i);

                ev_spawn_projectile.send(SpawnProjectileEvent {
                    texture_path: "textures/earthquake.png".to_string(),
                    color: element.color(),
                    translation: position,
                    angle: direction.to_angle(),
                    trajectory: Trajectory::Straight,
                    collider_radius: 8.0,
                    speed: 75.0,
                    damage: 20,
                    element,
                    is_friendly: false,
                });

                if phase_manager.current_phase == 3 {
                    if i == second_to_skip {
                        continue;
                    }

                    let second_position = get_wall_pos(second_direction, i);

                    ev_spawn_projectile.send(SpawnProjectileEvent {
                        texture_path: "textures/earthquake.png".to_string(),
                        color: element.color(),
                        translation: second_position,
                        angle: second_direction.to_angle(),
                        trajectory: Trajectory::Straight,
                        collider_radius: 8.0,
                        speed: 75.0,
                        damage: 20,
                        element,
                        is_friendly: false,
                    });
                }
            }
        }

        BossAttackType::Radial => {
            println!("radial");
            let amount_attack = rand::thread_rng().gen_range(8..16);

            let radius = rand::thread_rng().gen_range(500..800);
            let second_radius = rand::thread_rng().gen_range(radius + 500..radius + 800);
            let offset = 2.0 * PI / (amount_attack as f32);

            let amount_skip1 = rand::thread_rng().gen_range(3..5);
            let amount_skip2 = rand::thread_rng().gen_range(1..3);

            let to_skip = vec![rand::thread_rng().gen_range(0..amount_attack); amount_skip1];
            let second_to_skip = vec![rand::thread_rng().gen_range(0..amount_attack); amount_skip2];

            for i in 0..amount_attack {
                if !to_skip.contains(&i) {
                    let direction = -Vec2::from_angle(i as f32 * offset);
                    let position = (player_pos.translation.truncate()
                        - direction * (radius as f32) / 10.)
                        .extend(1.0);

                    ev_spawn_projectile.send(SpawnProjectileEvent {
                        texture_path: "textures/fireball.png".to_string(),
                        color: element.color(),
                        translation: position,
                        angle: direction.to_angle(),
                        trajectory: Trajectory::Straight,
                        collider_radius: 8.0,
                        speed: 42.5,
                        damage: 20,
                        element,
                        is_friendly: false,
                    });
                }

                if phase_manager.current_phase == 3 && !second_to_skip.contains(&i) {
                    let direction = -Vec2::from_angle(i as f32 * offset);
                    let second_position = (player_pos.translation.truncate()
                        - direction * (second_radius as f32) / 10.)
                        .extend(1.0);

                    ev_spawn_projectile.send(SpawnProjectileEvent {
                        texture_path: "textures/fireball.png".to_string(),
                        color: element.color(),
                        translation: second_position,
                        angle: direction.to_angle(),
                        trajectory: Trajectory::Straight,
                        collider_radius: 8.0,
                        speed: 35.0,
                        damage: 30,
                        element,
                        is_friendly: false,
                    });
                }
            }
        }
        BossAttackType::Blank => {
            println!("blank");
        }

        BossAttackType::Shield => {
            println!("shield");
        }
        BossAttackType::FastPierce => {
            println!("fast pierce");
            amount_attack += 2;
            if phase_manager.current_phase == 3 {
                amount_attack += 3;
            }
            let angle_disp = PI / (8 + amount_attack) as f32;
            let mut angle = (player_pos.translation - boss_position.translation)
                .truncate()
                .to_angle()
                - angle_disp * amount_attack as f32 / 2.; //???????
            for _ in 0..amount_attack {
                ev_spawn_projectile.send(SpawnProjectileEvent {
                    texture_path: "textures/fireball.png".to_string(),
                    color: element.color(),
                    translation: boss_position.translation,
                    angle: angle,
                    trajectory: Trajectory::Straight,
                    collider_radius: 8.0,
                    speed: 350.0,
                    damage: 20,
                    element,
                    is_friendly: false,
                });
                angle += angle_disp / 2.; //???????
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
            amount_attack += 4;

            for i in 0..amount_attack {
                let destination_pos = Vec2::new(
                    STATIC_ANGLE_POINTS[i].0 as f32 * 32.,
                    STATIC_ANGLE_POINTS[i].1 as f32 * 32.,
                );
                ev_mob_spawn.send(MobSpawnEvent {
                    mob_type: MobType::EarthElemental,
                    pos: destination_pos,
                    is_friendly: false,
                    owner: Some(boss_e),
                    loot: None,
                    exp_amount: 0,
                });
            }
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
            amount_attack = rand::thread_rng().gen_range(6..12);

            let radius = rand::thread_rng().gen_range(24..48) as f32;
            let angle = PI / 6.;

            for j in 0..7 {
                let offset = (2.0 * PI) / (amount_attack + j) as f32;
                let position = boss_position.translation.truncate() + Vec2::from_angle(angle * (j) as f32) * radius;
            
                for i in 0..(amount_attack + j) {
            
                    let angle = offset * i as f32;

                    ev_spawn_projectile.send(SpawnProjectileEvent {
                        texture_path: "textures/earthquake.png".to_string(),
                        color: element.color(),
                        translation: Vec3::new(position.x, position.y, 0.),
                        angle,
                        collider_radius: 12.,
                        speed: 100.0,
                        damage: 20,
                        element,
                        is_friendly: false,
                        trajectory: crate::projectile::Trajectory::Straight,
                    });
                }

            }
        }
        BossAttackType::MegaStan => {
            let counter_clockwise = rand::thread_rng().gen_bool(0.5);

            amount_attack += 15;
            let offset = PI / 10.0;
            let mut rng = rand::thread_rng();
            for i in 0..amount_attack {
                let dir = (player_pos.translation.x - boss_position.translation.truncate())
                    .normalize_or_zero();
                let angle = dir.y.atan2(dir.x) + rng.gen_range(-offset..offset);

                let radius = 256.;
                let pivot = if counter_clockwise {
                    boss_position.translation.truncate()
                        + Vec2::new(10. * i as f32, 80.)
                        + Vec2::from_angle(angle + PI / 2.) * radius
                } else {
                    boss_position.translation.truncate()
                        - Vec2::new(10. * i as f32, 80.)
                        - Vec2::from_angle(angle + PI / 2.) * radius
                };

                ev_spawn_projectile.send(SpawnProjectileEvent {
                    texture_path: "textures/small_fire.png".to_string(),
                    color: element.color(),
                    translation: boss_position.translation,
                    trajectory: Trajectory::Radial {
                        radius: radius,
                        pivot: pivot,
                        counter_clockwise: counter_clockwise,
                    },
                    angle: angle,
                    collider_radius: 8.,
                    speed: 0.5,
                    damage: 15,
                    element: element,
                    is_friendly: false,
                });
            }
        }
        BossAttackType::SpawnFireElemental => {
            println!("fire");
            amount_attack += 4;
            let radius = 64.;
            let mut angle: f32 = 0.;

            for _ in 0..amount_attack {
                ev_mob_spawn.send(MobSpawnEvent {
                    mob_type: MobType::FireElemental,
                    pos: Vec2::new(
                        player_pos.translation.x + radius * angle.cos(),
                        player_pos.translation.y + radius * angle.sin(),
                    ),
                    is_friendly: false,
                    loot: None,
                    owner: Some(boss_e),
                    exp_amount: 0,
                });
                angle += PI / 2.;
            }
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

//weights depend on:
//Cooldown,
//range to the player
//phase
//boss hp
//base value
//position of player
//is there such mobs
pub fn projectiles_check(
    friendly_projs_query: Query<(&Projectile, &Transform), With<Friendly>>,
    mut big_boss_query: Query<(&Transform, &mut BossAttackSystem)>,
) {
    let Ok((boss_pos, mut weights_system)) = big_boss_query.get_single_mut() else {
        return;
    };
    for (_proj, proj_pos) in friendly_projs_query.iter() {
        weights_system.weight_array[BossAttackType::Blank as usize] += (7500.
            / boss_pos
                .translation
                .truncate()
                .distance(proj_pos.translation.truncate()))
            as i16
            + 1;
        weights_system.weight_array[BossAttackType::Shield as usize] += (7500.
            / boss_pos
                .translation
                .truncate()
                .distance(proj_pos.translation.truncate()))
            as i16
            + 1;
    }
}

pub fn check_is_summon_alive(mob_query: Query<&Mob>, mut summoner_query: Query<&mut SummonQueue>) {
    for mut summon_list in summoner_query.iter_mut() {
        for i in 0..summon_list.queue.len() {
            if summon_list.queue[i].entity.is_some() {
                if !mob_query.contains(summon_list.queue[i].entity.unwrap())
                    && summon_list.queue[i].mob_type != MobType::Mossling
                {
                    summon_list.shift(i);
                    break;
                }
            }
        }
    }
}

pub fn recalculate_weights(
    mut boss_query: Query<(
        &mut BossAttackSystem,
        &mut SummonQueue,
        &Health,
        &Transform,
        &PhaseManager,
    )>,
    player_query: Query<&Transform, With<Player>>,
) {
    let Ok((mut attack_system, summon_list, boss_hp, boss_transform, phase_manager)) =
        boss_query.get_single_mut()
    else {
        return;
    };
    let Ok(player_transform) = player_query.get_single() else {
        println!("Player died! or smth");
        return;
    };
    let phase = phase_manager.current_phase;

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
                base_weight += (phase == 3) as i16 * i16::MIN
                    + 5000
                        * (summon_list
                            .queue
                            .iter()
                            .filter(|summon_unit| summon_unit.mob_type != MobType::Mossling)
                            .count()
                            == 0) as i16;

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
                base_weight += (phase != 3) as i16 * i16::MIN + 5000; //add bonus when player far away from walls
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
                base_weight += (summon_list.queue.len() + 5
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
                        * 50;

                base_weight += ((boss_hp.max - boss_hp.current) / 20) as i16
                    - (summon_list.amount_of_mobs as i32 * (150 * ((phase == 2) as i32 + 50)))
                        as i16;
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
            }
        }
    }
}

pub fn cast_blank(
    mut spawn_blank_ev: EventWriter<SpawnBlankEvent>,
    mut boss_query: Query<(&mut BossAttackSystem, &Transform)>,
) {
    let Ok((mut attack_system, pos)) = boss_query.get_single_mut() else {
        return;
    };

    if attack_system.weight_array[BossAttackType::Blank as usize] > 2500 {
        attack_system.cooldown_mask ^= 1 << BossAttackType::Blank as usize;
        spawn_blank_ev.send(SpawnBlankEvent {
            range: 100.,
            position: pos.translation,
            speed: 20.,
            is_friendly: false,
        });
    }
}

pub fn cast_shield(
    mut boss_query: Query<(Entity, &mut BossAttackSystem)>,
    mut cast_shield: EventWriter<SpawnShieldEvent>,
) {
    let Ok((boss_e, mut attack_system)) = boss_query.get_single_mut() else {
        return;
    };

    if attack_system.weight_array[BossAttackType::Shield as usize] >= 2500 {
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

fn boss_teleport() {}

fn boss_running() {}
