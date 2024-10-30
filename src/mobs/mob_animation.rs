use bevy::prelude::*;

use crate::{animation::AnimationConfig, mobs::mob::*, player::Player, stun::Stun, GameState};

pub struct MobAnimationPlugin;

impl Plugin for MobAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (animate_mobs, rotate_mobs, mob_flip).run_if(in_state(GameState::InGame)),
        );
    }
}
fn animate_mobs(
    time: Res<Time>,
    mut query: Query<(&mut AnimationConfig, &mut TextureAtlas), (With<Mob>, Without<Stun>)>,
) {
    for (mut config, mut atlas) in &mut query {
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
}

fn rotate_mobs(
    player_query: Query<&Transform, (With<Player>, Without<RotationEntity>)>,
    mut rotation_query: Query<
        (&GlobalTransform, &mut Transform),
        (With<RotationEntity>, Without<Player>, Without<Stun>),
    >,
    time: Res<Time>,
) {
    for (global_rotation, mut rotation_en) in &mut rotation_query {
        if let Ok(player_transform) = player_query.get_single() {
            let translation = global_rotation.translation();
            let diff = Vec3::new(
                player_transform.translation.x,
                player_transform.translation.y,
                translation.z,
            ) - translation;
            let angle = diff.y.atan2(diff.x);
            rotation_en.rotation = rotation_en
                .rotation
                .lerp(Quat::from_rotation_z(angle), 12.0 * time.delta_seconds());
        }
    }
}
fn mob_flip() {}
