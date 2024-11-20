use avian2d::prelude::*;
use bevy::prelude::*;
use seldom_state::prelude::*;
use crate::{animation::AnimationConfig, mobs::mob::*, player::Player, stun::Stun, GameState,pathfinding::Pathfinder};

pub struct MobAnimationPlugin;

impl Plugin for MobAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (animate_mobs, rotate_mobs, mob_flip, animate_mob_attack).run_if(in_state(GameState::InGame)),
        );
    }
}
///flag for multistate animation
#[derive(Component)]
pub struct MultistateAnimationFlag;
fn animate_mobs(
    time: Res<Time>,
    mut query: Query<(&mut AnimationConfig, &mut TextureAtlas), (With<Mob>, Without<Stun>, (With<SearchAndPursue>, Without<MultistateAnimationFlag>))>,
    mut multistate_query: Query<(&mut AnimationConfig, &mut TextureAtlas, &LinearVelocity), (With<SearchAndPursue>, With<MultistateAnimationFlag>)>,
    mut pathfinder_query: Query<(&mut AnimationConfig, &mut TextureAtlas), (With<Mob>, Without<Stun>, (With<Pathfinder>, Without<MultistateAnimationFlag>, Without<SearchAndPursue>))>,
    
) {
    fn animate(config: &mut AnimationConfig, atlas: &mut TextureAtlas, time: &Time) {
        // we track how long the current sprite has been displayed for
        config.frame_timer.tick(time.delta());

        // If it has been displayed for the user-defined amount of time (fps)...
        if config.frame_timer.just_finished() {
            if atlas.index == config.last_sprite_index {
                // ...and it IS the last frame, then we move back to the first frame and stop.
                atlas.index = config.first_sprite_index;
            } else {
                // ...and it is NOT the last frame, then we move to the next frame...
                atlas.index += 1;
                // ...and reset the frame timer to start counting all over again
                config.frame_timer = AnimationConfig::timer_from_fps(config.fps);
            }
        }
    }

    for (mut config, mut atlas) in query.iter_mut() {
        animate(&mut config, &mut atlas, &time);
    }

    for (mut config, mut atlas) in pathfinder_query.iter_mut() {
        animate(&mut config, &mut atlas, &time);
    }

    for (mut config, mut atlas, linvel) in multistate_query.iter_mut() {
        if linvel.0 != Vec2::ZERO {
            animate(&mut config, &mut atlas, &time);
        }
    }   
}

fn rotate_mobs(
    player_query: Query<&Transform, (With<Player>, Without<RotationEntity>)>,
    mut rotation_query: Query<
        (&GlobalTransform, &mut Transform),
        (With<RotationEntity>, Without<Player>, Without<Stun>),
    >,
    time: Res<Time>,
) {
    for (global_transform, mut rotation_en) in &mut rotation_query {
        if let Ok(player_transform) = player_query.get_single() {
            let translation = global_transform.translation();
            let diff = (player_transform.translation - translation).truncate().normalize_or_zero();

            let angle = diff.to_angle();

            rotation_en.rotation = rotation_en
            .rotation
            .lerp(Quat::from_rotation_z(angle), 12.0 * time.delta_seconds());
        }
    }
}
fn mob_flip(
    mut mob_query: Query <(&mut Transform,&LinearVelocity),  (With<FlipEntity>, With<Pathfinder>)>,
    time: Res<Time>,
) {
    for (mut transform,lin_vel) in mob_query.iter_mut() {
        if lin_vel.x > 0. {
            transform.scale.x = transform
                .scale
                .x
                .lerp(-1.0, 10.0 * time.delta_seconds());
        } else {
            transform.scale.x = transform
                .scale
                .x
                .lerp(1.0, 10.0 * time.delta_seconds());
        }
    }
}

fn animate_mob_attack(
    time: Res<Time>,
    mut attack_query: Query<(Entity, &mut AnimationConfig, &mut TextureAtlas, &Parent), With<Attack>>,
    mut commands: Commands,
    mob_query: Query<Entity, (With<AttackFlag>, With<Mob>)>
    //add parent_query to get and animate parent?
) {
    fn animate(config: &mut AnimationConfig, atlas: &mut TextureAtlas, time: &Time) -> bool {
        // we track how long the current sprite has been displayed for
        config.frame_timer.tick(time.delta());
        let mut last_frame_check: bool = false;
        // If it has been displayed for the user-defined amount of time (fps)...
        if config.frame_timer.just_finished() {
            if atlas.index == config.last_sprite_index {
                // ...and it IS the last frame, then we move back to the first frame and stop.
                last_frame_check = true;
            } else {
                // ...and it is NOT the last frame, then we move to the next frame...
                atlas.index += 1;
                // ...and reset the frame timer to start counting all over again
                config.frame_timer = AnimationConfig::timer_from_fps(config.fps);
            }
        }
        return last_frame_check;
    }

    for (attack_e,mut config, mut atlas,parent) in attack_query.iter_mut() {
        let last_frame: bool = animate(&mut config, &mut atlas, &time);
        if last_frame{
            commands.entity(attack_e).despawn();
            let mob_e = mob_query.get(**parent);
            match mob_e{
                Ok(ent) => {commands.entity(ent).insert(Done::Success);},
                _=> {},
            }
        }
    }
}