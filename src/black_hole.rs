use avian2d::prelude::ExternalForce;
use bevy::prelude::*;

use crate::{mobs::Mob, utils::pulsate};

pub struct BlackHolePlugin;

impl Plugin for BlackHolePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnBlackHoleEvent>();

        app.add_systems(
            Update,
            (
                spawn_black_hole,
                move_black_hole,
                pulsate::<BlackHole>,
                pull_mobs,
                despawn_black_hole_on_timer,
            ),
        );
    }
}

#[derive(Event)]
pub struct SpawnBlackHoleEvent {
    pub spawn_pos: Vec3,
    pub target_pos: Vec3,
    pub lifetime: f32,
    pub strength: f32,
}

#[derive(Component)]
pub struct BlackHole {
    target_pos: Vec3,
    timer: Timer,
    strength: f32,
}

fn spawn_black_hole(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut ev_spawn_black_hole: EventReader<SpawnBlackHoleEvent>,
) {
    for ev in ev_spawn_black_hole.read() {
        commands
            .spawn(SpriteBundle {
                texture: asset_server.load("textures/black_hole.png"),
                transform: Transform::from_translation(ev.spawn_pos),
                ..default()
            })
            .insert(BlackHole {
                target_pos: ev.target_pos,
                timer: Timer::from_seconds(ev.lifetime, TimerMode::Once),
                strength: ev.strength,
            });
    }
}

fn move_black_hole(mut black_hole_query: Query<(&mut Transform, &BlackHole)>, time: Res<Time>) {
    for (mut transform, black_hole) in black_hole_query.iter_mut() {
        transform.translation = transform.translation.lerp(
            black_hole.target_pos,
            50. / transform.translation.distance(black_hole.target_pos) * time.delta_seconds(),
        );
    }
}

fn pull_mobs(
    mut black_hole_query: Query<(&Transform, &BlackHole)>,
    mut mob_query: Query<(&mut ExternalForce, &Transform), With<Mob>>,
) {
    for (hole_tf, hole) in black_hole_query.iter_mut() {
        for (mut force, mob_tf) in mob_query.iter_mut() {
            if hole_tf.translation.distance(mob_tf.translation) <= 64. {
                let direction = (hole_tf.translation - mob_tf.translation)
                    .truncate()
                    .normalize_or_zero();
                force.apply_force(direction * hole.strength).with_persistence(false);
            }
        }
    }
}

fn despawn_black_hole_on_timer(
    mut commands: Commands,
    mut black_hole_query: Query<(Entity, &mut BlackHole)>,
    mut mob_query: Query<&mut ExternalForce, With<Mob>>,
    time: Res<Time>,
) {
    for (entity, mut black_hole) in black_hole_query.iter_mut() {
        black_hole.timer.tick(time.delta());

        if black_hole.timer.just_finished() {

            for mut external_force in mob_query.iter_mut() {
                external_force.clear();
            }

            commands.entity(entity).despawn();
        }
    }
}
