use avian2d::prelude::*;
use bevy::prelude::*;
use rand::Rng;

use crate::{
    gamemap::{Wall, ROOM_SIZE}, mobs::mob::*, pathfinding::Pathfinder, player::Player, stun::Stun,
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
        
        app.add_systems(FixedUpdate, (idle, pursue_player, update_weights));
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
        (Without<Stun>, Without<Teleport>, Without<Raising>, Without<SearchAndPursue>),
    >,
    time: Res<Time>,
) {
    for (mut linvel, transform, mut pathfinder) in mob_query.iter_mut() {
        if pathfinder.path.len() > 0 {
            //let mob_tile_pos = Vec2::new(((transform.translation.x - (ROOM_SIZE / 2) as f32) / ROOM_SIZE as f32).floor(), (transform.translation.y - (ROOM_SIZE / 2) as f32) / ROOM_SIZE as f32).floor();
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

fn idle(
    mut commands: Commands,
    mut mob_query: Query<(Entity, &Transform, &mut SearchAndPursue, &mut RayCaster, &RayHits), With<Idle>>,
    player_query: Query<(Entity, &Transform), With<Player>>,
    time: Res<Time>,
) {
    let Ok((player_e, player_transform)) = player_query.get_single() else {
        return;
    };

    for (mob_e, mob_transform, mut mob, mut ray, hits) in mob_query.iter_mut() {

        mob.wander_timer.tick(time.delta());

        if mob.wander_timer.elapsed_secs() == mob.wander_timer.remaining_secs() {
            let directions: Vec<Vec2> = mob.rays.iter().map(|ray| ray.direction).collect();
            let direction = directions[rand::thread_rng().gen_range(0..directions.len())];

            commands.entity(mob_e).insert(LinearVelocity(direction * mob.speed * time.delta_seconds()));
        }
        
        if mob.wander_timer.just_finished() {
            commands.entity(mob_e).insert(LinearVelocity::ZERO);
        }

        ray.direction = Dir2::new_unchecked((player_transform.translation - mob_transform.translation).truncate().normalize());
        let hits_data = hits.iter_sorted().collect::<Vec<RayHitData>>();
        if player_transform.translation.distance(mob_transform.translation) <= mob.pursue_radius {
            if !hits_data.is_empty() && hits_data[0].entity == player_e {
                commands.entity(mob_e).remove::<Idle>();
                commands.entity(mob_e).insert(PursuePlayer);
                mob.search_time.reset();
            }
        }
    }
}

fn pursue_player(
    mut commands: Commands,
    mut mob_query: Query<(Entity, &mut LinearVelocity, &Transform, &mut SearchAndPursue, &mut RayCaster, &RayHits), With<PursuePlayer>>,
    player_query: Query<(Entity, &Transform), With<crate::player::Player>>,
    time: Res<Time>,
) {
    let Ok((player_e, player_transform)) = player_query.get_single() else {
        return;
    };

    for (mob_e, mut linvel, mob_transform, mut mob, mut ray, hits) in mob_query.iter_mut() {
        let direction = (player_transform.translation - mob_transform.translation).truncate().normalize();
        ray.direction = Dir2::new_unchecked(direction);
       
        let hits_data = hits.iter_sorted().collect::<Vec<RayHitData>>();
    
        if !hits_data.is_empty() {
            if hits_data[0].entity == player_e {
                mob.last_player_dir = direction;
            }
        }
    
        let mut desire_direction = Vec2::ZERO;
    
        for ray in mob.rays.iter() {
            desire_direction += ray.direction * ray.weight;
        }
    
        linvel.0 = (mob.last_player_dir + desire_direction) * mob.speed * time.delta_seconds();

        mob.search_time.tick(time.delta());

        if player_transform.translation.distance(mob_transform.translation) > mob.pursue_radius
        || mob.search_time.just_finished() {
            commands.entity(mob_e).remove::<PursuePlayer>();
            commands.entity(mob_e).insert(Idle);
            linvel.0 = Vec2::ZERO;
            mob.last_player_dir = Vec2::ZERO;
        }
    }
}

fn update_weights(
    spatial_query: SpatialQuery,
    mut mob_query: Query<(&Transform, &mut SearchAndPursue), With<PursuePlayer>>,
    avoid_query: Query<Entity, With<Wall>>,
) {
    for (mob_transform, mut mob) in mob_query.iter_mut() {
        for i in 0..16 {
            mob.rays[i].weight = 0.0;
    
            let offset = std::f32::consts::PI/8.0;
            let Some(first_hit) = spatial_query.cast_ray_predicate(
                mob_transform.translation.truncate(),
                Dir2::new_unchecked(Vec2::from_angle(i as f32 * offset)),
                128.,
                true,
                SpatialQueryFilter::default(),
                &|entity| {
                    avoid_query.contains(entity)
                })
            else {
                continue;
            };
    
            if first_hit.time_of_impact < 24. {
                mob.rays[i].weight = -1. / first_hit.time_of_impact;
            }
        }
    };
}