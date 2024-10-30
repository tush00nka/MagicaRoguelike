use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    gamemap::ROOM_SIZE, mobs::mob::*, pathfinding::Pathfinder, player::Player, stun::Stun,
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
        (Without<Stun>, Without<Teleport>, Without<Raising>),
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
