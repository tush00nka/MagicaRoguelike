use avian2d::prelude::*;
use bevy::prelude::*;
use core::f32;
use rand::Rng;
use seldom_state::prelude::*;

use crate::mobs::air_elemental_movement;
use crate::{
    alert::SpawnAlertEvent,
    blank_spell::Blank,
    friend::Friend,
    gamemap::{Wall, ROOM_SIZE},
    mobs::mob::*,
    obstacles::Corpse,
    pathfinding::{Pathfinder, EnemyRush, FriendRush},
    player::Player,
    shield_spell::Shield,
    stun::Stun,
    GameState,
};

pub struct MobMovementPlugin;

impl Plugin for MobMovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, teleport_mobs.run_if(in_state(GameState::InGame)))
            .add_systems(
                FixedUpdate,
                (
                    move_mobs::<FriendRush>,
                    move_mobs::<EnemyRush>,
                    runaway_mob,
                    phasing_mob::<Enemy, Friend>,
                    phasing_mob::<Friend, Enemy>,
                    air_elemental_movement::<Friend>,
                    air_elemental_movement::<Enemy>,
                )
                    .run_if(in_state(GameState::InGame)),
            );

        app.add_systems(
            FixedUpdate,
            (
                idle::<Enemy, Friend>,
                pursue::<Enemy, Friend>,
                idle_static::<Enemy, Friend>,

                idle::<Friend, Enemy>,
                pursue::<Friend, Enemy>,
                idle_static::<Friend, Enemy>,
                update_weights,
            ),
        );
    }
}

//телепорт в заранее найденное место для телепортации
fn teleport_mobs(
    // mut commands: Commands,
    mut mob_query: Query<
        (Entity, &mut Transform, &mut Teleport),
        (Without<Stun>, With<TeleportFlag>),
    >,
    mut commands: Commands,
) {
    for (mob, mut transform, mut tp) in mob_query.iter_mut() {
        if tp.place_to_teleport.len() > 0 {
            transform.translation = Vec3::new(
                tp.place_to_teleport[0].0 as f32 * ROOM_SIZE as f32,
                tp.place_to_teleport[0].1 as f32 * ROOM_SIZE as f32,
                1.0,
            );
            tp.place_to_teleport.remove(0);
            commands.entity(mob).insert(Done::Success);
        }
    }
}

//система для мобов, которые убегают от таргета
fn runaway_mob(
    mut mob_query: Query<
        (&mut LinearVelocity, &Transform, &mut Pathfinder),
        (
            Without<Stun>,
            Without<Teleport>,
            Without<RaisingFlag>,
            With<RunawayRush>,
        ),
    >,
    mut player_query: Query<&Transform, (With<Player>, Without<Mob>)>,
    time: Res<Time>,
) {
    if let Ok(player) = player_query.get_single_mut() {
        let player_pos = player.translation.truncate();
        for (mut linvel, transform, pathfinder) in mob_query.iter_mut() {
            let direction = Vec2::new(
                transform.translation.x - player_pos.x,
                transform.translation.y - player_pos.y,
            )
            .normalize();
            linvel.0 = direction * pathfinder.speed * time.delta_seconds();
        }
    }
}

//система для мобов, которые двигаются сквозь стены
fn phasing_mob<Side: Component, Target: Component>(
    mut side_query: Query<
        (
            Entity,
            &mut LinearVelocity,
            &Transform,
            &Phasing,
            &mut AttackComponent,
        ),
        (
            Without<Stun>,
            Without<Teleport>,
            Without<RaisingFlag>,
            With<PhasingFlag>,
            With<Side>,
            Without<Target>,
        ),
    >,
    mut ev_spawn_alert: EventWriter<SpawnAlertEvent>,
    target_query: Query<(Entity, &Transform), (With<Target>, Without<Side>)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (phasing_e, mut linvel, transform, pathfinder, mut attack) in side_query.iter_mut() {
        if target_query.iter().len() <= 0 {
            return;
        }

        let sorted_targets: Vec<(Entity, &Transform)> = target_query
            .iter()
            .sort_by::<&Transform>(|item1, item2| {
                item1
                    .translation
                    .distance(transform.translation)
                    .total_cmp(&item2.translation.distance(transform.translation))
            })
            .collect();

        let (target_e, target_transform) = sorted_targets[0];

        let direction = Vec2::new(
            target_transform.translation.x - transform.translation.x,
            target_transform.translation.y - transform.translation.y,
        )
        .normalize();

        linvel.0 = direction * pathfinder.speed * time.delta_seconds();

        if target_transform.translation.distance(transform.translation) < attack.range
            && !attack.attacked
        //melee range idk
        {
            commands.entity(phasing_e).insert(Done::Success);
            attack.target = Some(target_e);
            attack.attacked = true;
            linvel.0 = Vec2::ZERO;

            ev_spawn_alert.send(SpawnAlertEvent {
                position: transform
                    .translation
                    .truncate()
                    .with_y(transform.translation.y + 16.),
                attack_alert: true,
            });
        }
    }
}

//система для ходьбы мобов в соответствии с pathfinding-ом
fn move_mobs<SideRush: Component>(
    mut mob_query: Query<
        (&mut LinearVelocity, &Transform, &mut Pathfinder),
        (
            Without<Stun>,
            Without<Teleport>,
            Without<RaisingFlag>,
            Without<Phasing>,
            Or<(With<SideRush>, Without<SearchAndPursue>)>,
        ),
    >,
    time: Res<Time>,
) {
    for (mut linvel, transform, mut pathfinder) in mob_query.iter_mut() {
        if pathfinder.path.len() > 0 {
            let direction = Vec2::new(
                pathfinder.path[0].0 as f32 * 32. - transform.translation.x,
                pathfinder.path[0].1 as f32 * 32. - transform.translation.y,
            )
            .normalize();

            linvel.0 = direction * pathfinder.speed * time.delta_seconds();

            if transform.translation.truncate().distance(Vec2::new(
                pathfinder.path[0].0 as f32 * 32.,
                pathfinder.path[0].1 as f32 * 32.,
            )) <= 4.
            {
                pathfinder.path.remove(0);
            }
        }
    }
}

//система для отдыха статических мобов, когда видят врага - атакуют
fn idle_static<Who: Component, Target: Component>(
    mut commands: Commands,
    spatial_query: SpatialQuery,
    mut mob_query: Query<
        (Entity, &Transform, &mut SearchAndPursue, &mut AttackComponent),
        (With<IdleStatic>, With<Who>, Without<Target>),
    >,
    target_query: Query<(Entity, &Transform), With<Target>>,
    mut ev_spawn_alert: EventWriter<SpawnAlertEvent>,
    ignore_query: Query<Entity, Or<(With<Corpse>, With<Shield>, With<Blank>, With<Who>)>>,
) {
    for (mob_e, mob_transform, mut mob, mut attack_range) in mob_query.iter_mut() {
        if target_query.iter().len() <= 0 {
            return;
        }

        let sorted_targets: Vec<(Entity, &Transform)> = target_query
            .iter()
            .sort_by::<&Transform>(|item1, item2| {
                item1
                    .translation
                    .distance(mob_transform.translation)
                    .total_cmp(&item2.translation.distance(mob_transform.translation))
            })
            .collect();

        let (target_e, target_transform) = sorted_targets[0];

        let dir = (target_transform.translation - mob_transform.translation)
            .truncate()
            .normalize();

        let Some(first_hit) = spatial_query.cast_ray_predicate(
            mob_transform.translation.truncate(),
            Dir2::new_unchecked(dir),
            512.,
            true,
            SpatialQueryFilter::default().with_excluded_entities(&ignore_query),
            &|entity| entity != mob_e,
        ) else {
            continue;
        };
        if first_hit.entity == target_e {
            mob.last_target_dir = dir;
        }else{
            commands.entity(mob_e).insert(Done::Failure);
            continue;
        }
        if target_transform
            .translation
            .distance(mob_transform.translation)
            < attack_range.range
            && !attack_range.attacked
        //melee range idk
        {
            commands.entity(mob_e).insert(Done::Success);
            attack_range.attacked = true;
            mob.last_target_dir = Vec2::ZERO;
            attack_range.target = Some(target_e);
            
            ev_spawn_alert.send(SpawnAlertEvent {
                position: mob_transform
                    .translation
                    .truncate()
                    .with_y(mob_transform.translation.y + 16.),
                attack_alert: true,
            });
        }
    }
}

//система отдыха для подвижных врагов, когда видят врага - преследуют
fn idle<Who: Component, Target: Component>(
    mut commands: Commands,
    spatial_query: SpatialQuery,
    mut mob_query: Query<
        (Entity, &Transform, &mut SearchAndPursue),
        (With<Idle>, With<Who>, Without<Target>),
    >,
    target_query: Query<(Entity, &Transform), With<Target>>,
    mut ev_spawn_alert: EventWriter<SpawnAlertEvent>,
    ignore_query: Query<Entity, Or<(With<Corpse>, With<Shield>, With<Blank>, With<Who>)>>,
    time: Res<Time>,
) {
    for (mob_e, mob_transform, mut mob) in mob_query.iter_mut() {
        // --- Гуляем ---

        mob.wander_timer.tick(time.delta());

        // получаем вектор, корректирующий направление моба,
        // суммируя произведение вектора направления на его вес
        let ray_sum_dir: Vec2 = mob.rays.iter().map(|ray| ray.direction * ray.weight).sum();

        if mob.wander_timer.elapsed_secs() == mob.wander_timer.remaining_secs() {
            let directions: Vec<Vec2> = mob.rays.iter().map(|ray| ray.direction).collect();
            let direction = directions[rand::thread_rng().gen_range(0..directions.len())];

            commands.entity(mob_e).insert(LinearVelocity(
                (direction + ray_sum_dir) * mob.speed * time.delta_seconds(),
            ));
        }

        if mob.wander_timer.just_finished() {
            commands.entity(mob_e).insert(LinearVelocity::ZERO);
        }

        // --- Детектим цель ---
        if target_query.iter().len() <= 0 {
            commands.entity(mob_e).insert(Done::Success);
            return;
        }

        let sorted_targets: Vec<(Entity, &Transform)> = target_query
            .iter()
            .sort_by::<&Transform>(|item1, item2| {
                item1
                    .translation
                    .distance(mob_transform.translation)
                    .total_cmp(&item2.translation.distance(mob_transform.translation))
            })
            .collect();

        let (target_e, target_transform) = sorted_targets[0];

        let dir = (target_transform.translation - mob_transform.translation)
            .truncate()
            .normalize();

        let Some(first_hit) = spatial_query.cast_ray_predicate(
            mob_transform.translation.truncate(),
            Dir2::new_unchecked(dir),
            512.,
            true,
            SpatialQueryFilter::default().with_excluded_entities(&ignore_query),
            &|entity| entity != mob_e,
        ) else {
            commands.entity(mob_e).insert(Done::Success);
            continue;
        };

        if target_transform
            .translation
            .distance(mob_transform.translation)
            <= mob.pursue_radius
        {
            if first_hit.entity == target_e {
                commands.entity(mob_e).insert(Done::Success);
                mob.search_time.reset();

                ev_spawn_alert.send(SpawnAlertEvent {
                    position: mob_transform
                        .translation
                        .truncate()
                        .with_y(mob_transform.translation.y + 16.),
                    attack_alert: false,
                });
            }else{
                commands.entity(mob_e).insert(Done::Failure);
            }
        }
    }
}

//система преследования, если таргет в радиусе атаки и атака не в кд - юнит атакует
fn pursue<Who: Component, Target: Component>(
    mut commands: Commands,
    spatial_query: SpatialQuery,
    mut mob_query: Query<
        (
            Entity,
            &mut LinearVelocity,
            &Transform,
            &mut SearchAndPursue,
            &mut AttackComponent,
        ),
        (With<Pursue>, With<Who>, Without<Target>),
    >,
    target_query: Query<(Entity, &Transform), With<Target>>,
    ignore_query: Query<Entity, Or<(With<Corpse>, With<Shield>, With<Blank>, With<Enemy>)>>,
    mut ev_spawn_alert: EventWriter<SpawnAlertEvent>,
    time: Res<Time>,
) {
    for (mob_e, mut linvel, mob_transform, mut mob, mut attack_range) in mob_query.iter_mut() {
        if target_query.iter().len() <= 0 {
            commands.entity(mob_e).insert(Done::Failure);
            return;
        }

        let sorted_targets: Vec<(Entity, &Transform)> = target_query
            .iter()
            .sort_by::<&Transform>(|item1, item2| {
                item1
                    .translation
                    .distance(mob_transform.translation)
                    .total_cmp(&item2.translation.distance(mob_transform.translation))
            })
            .collect();

        let (target_e, target_transform) = sorted_targets[0];
        let direction = (target_transform.translation - mob_transform.translation)
            .truncate()
            .normalize();

        let ray_sum_dir: Vec2 = mob.rays.iter().map(|ray| ray.direction * ray.weight).sum();

        let Some(first_hit) = spatial_query.cast_ray_predicate(
            mob_transform.translation.truncate(),
            Dir2::new_unchecked(direction),
            512.,
            true,
            SpatialQueryFilter::default().with_excluded_entities(&ignore_query),
            &|entity| entity != mob_e,
        ) else {
            commands.entity(mob_e).insert(Done::Failure);
            continue;
        };

        if first_hit.entity == target_e {
            mob.last_target_dir = direction;
        }
            
        linvel.0 = (mob.last_target_dir + ray_sum_dir) * mob.speed * time.delta_seconds();

        mob.search_time.tick(time.delta());

        if target_transform
            .translation
            .distance(mob_transform.translation)
            > mob.pursue_radius
            || mob.search_time.just_finished()
        {
            commands.entity(mob_e).insert(Done::Failure);

            linvel.0 = Vec2::ZERO;
            mob.last_target_dir = Vec2::ZERO;
        } else if target_transform
            .translation
            .distance(mob_transform.translation)
            < attack_range.range
            && !attack_range.attacked
        //melee range idk
        {
            commands.entity(mob_e).insert(Done::Success);
            attack_range.attacked = true;
            attack_range.target = Some(target_e);
            linvel.0 = Vec2::ZERO;
            mob.last_target_dir = Vec2::ZERO;

            ev_spawn_alert.send(SpawnAlertEvent {
                position: mob_transform
                    .translation
                    .truncate()
                    .with_y(mob_transform.translation.y + 16.),
                attack_alert: true,
            });
        }
    }
}

//система для обновления весов юнитов, которые двигаются на рейкастинге
fn update_weights(
    spatial_query: SpatialQuery,
    mut mob_query: Query<(&Transform, &mut SearchAndPursue)>,
    avoid_query: Query<Entity, With<Wall>>,
) {
    for (mob_transform, mut mob) in mob_query.iter_mut() {
        for i in 0..16 {
            mob.rays[i].weight = 0.0;

            let offset = std::f32::consts::PI / 8.0;
            let Some(first_hit) = spatial_query.cast_ray_predicate(
                mob_transform.translation.truncate(),
                Dir2::new_unchecked(Vec2::from_angle(i as f32 * offset)),
                128.,
                true,
                SpatialQueryFilter::default(),
                &|entity| avoid_query.contains(entity),
            ) else {
                continue;
            };

            if first_hit.time_of_impact < 24. {
                mob.rays[i].weight = -1. / first_hit.time_of_impact;
            }
        }
    }
}
