use core::f32;
use std::cmp::Ordering;

use avian2d::prelude::*;
use bevy::prelude::*;
use rand::Rng;

use crate::{
    alert::SpawnAlertEvent,
    blank_spell::Blank,
    friend::Friend,
    gamemap::{Wall, ROOM_SIZE},
    mobs::mob::*,
    obstacles::Corpse,
    pathfinding::Pathfinder,
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
                (move_mobs, runaway_mob).run_if(in_state(GameState::InGame)),
            );

        app.add_systems(FixedUpdate, (
            idle::<Enemy, Friend>, 
            pursue::<Enemy, Friend>,
            idle::<Friend, Enemy>, 
            pursue::<Friend, Enemy>,
            update_weights
        ));
    }
}

fn teleport_mobs(mut mob_query: Query<(&mut Transform, &mut Teleport), Without<Stun>>) {
    for (mut transform, mut mob) in mob_query.iter_mut() {
        if mob.place_to_teleport.len() > 0 {
            transform.translation = Vec3::new(
                mob.place_to_teleport[0].0 as f32 * ROOM_SIZE as f32,
                mob.place_to_teleport[0].1 as f32 * ROOM_SIZE as f32,
                1.0,
            );
            mob.place_to_teleport.remove(0);
        }
    }
}

fn runaway_mob(
    mut mob_query: Query<
        (&mut LinearVelocity, &Transform, &mut Pathfinder),
        (
            Without<Stun>,
            Without<Teleport>,
            Without<Raising>,
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

fn move_mobs(
    mut mob_query: Query<
        (&mut LinearVelocity, &Transform, &mut Pathfinder),
        (
            Without<Stun>,
            Without<Teleport>,
            Without<Raising>,
            Without<SearchAndPursue>,
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

fn idle<Who: Component, Target: Component>(
    mut commands: Commands,
    spatial_query: SpatialQuery,
    mut mob_query: Query<(Entity, &Transform, &mut SearchAndPursue), (With<Idle>, With<Who>, Without<Target>)>,
    target_query: Query<(Entity, &Transform), With<Target>>,
    mut ev_spawn_alert: EventWriter<SpawnAlertEvent>,
    ignore_query: Query<Entity, Or<(With<Corpse>, With<Shield>, With<Blank>)>>,
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
            return;
        }
        let sorted_targets: Vec<(Entity, &Transform)> = target_query.iter()
            .sort_by::<&Transform>(|item1, item2| { 
                if item1.translation.distance(mob_transform.translation) 
                < item2.translation.distance(mob_transform.translation) {
                    return Ordering::Less;
                }
                else if item1.translation.distance(mob_transform.translation) 
                > item2.translation.distance(mob_transform.translation){
                    return Ordering::Greater;
                }

                return Ordering::Equal
        }).collect();

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
            &|entity| {
                entity != mob_e
            } )
        else {
            continue;
        };

        // это что за хуйня, никита?? ты зачем на 100 строк разбил выражение
        if target_transform.translation.distance(mob_transform.translation) <= mob.pursue_radius
        {
            if first_hit.entity == target_e {
                commands.entity(mob_e).remove::<Idle>();
                commands.entity(mob_e).insert(Pursue);
                mob.search_time.reset();

                ev_spawn_alert.send(SpawnAlertEvent {
                    position: mob_transform
                        .translation
                        .truncate()
                        .with_y(mob_transform.translation.y + 16.),
                });
            }
        }
    }
}

fn pursue<Who: Component, Target: Component>(
    mut commands: Commands,
    spatial_query: SpatialQuery,
    mut mob_query: Query<
        (
            Entity,
            &mut LinearVelocity,
            &Transform,
            &mut SearchAndPursue,
        ),
        (With<Pursue>, With<Who>, Without<Target>),
    >,
    target_query: Query<(Entity, &Transform), With<Target>>,
    ignore_query: Query<Entity, Or<(With<Corpse>, With<Shield>, With<Blank>)>>,
    time: Res<Time>,
) {
    for (mob_e, mut linvel, mob_transform, mut mob) in mob_query.iter_mut() {

        if target_query.iter().len() <= 0 {
            return;
        }

        let sorted_targets: Vec<(Entity, &Transform)> = target_query.iter()
            .sort_by::<&Transform>(|item1, item2| { 
                if item1.translation.distance(mob_transform.translation) 
                < item2.translation.distance(mob_transform.translation) {
                    return Ordering::Less;
                }
                else if item1.translation.distance(mob_transform.translation) 
                > item2.translation.distance(mob_transform.translation){
                    return Ordering::Greater;
                }

                return Ordering::Equal
        }).collect();

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
            &|entity| {
                entity != mob_e
            } )
        else {
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
            commands.entity(mob_e).remove::<Pursue>();
            commands.entity(mob_e).insert(Idle);
            linvel.0 = Vec2::ZERO;
            mob.last_target_dir = Vec2::ZERO;
        }
    }
}
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
