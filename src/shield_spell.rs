use avian2d::prelude::{FixedJoint, LinearVelocity};
use bevy::prelude::*;

use crate::{player::Player, GameState};

pub struct ShieldSpellPlugin;

impl Plugin for ShieldSpellPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, (animate_shield, despawn_shield).run_if(in_state(GameState::InGame)));
    }
}

#[derive(Component)]
pub struct Shield {
    pub timer: Timer,
}

#[derive(Component)]
pub struct ShieldAnimation {
    pub speed: f32
}

fn animate_shield(
    mut commands: Commands,
    mut animation_query: Query<(Entity, &mut Transform, &ShieldAnimation)>,
    time: Res<Time>,
) {
    for (entity, mut transform, animation) in animation_query.iter_mut() {
        transform.scale = transform.scale.lerp(Vec3::ONE, animation.speed * time.delta_seconds());
        if transform.scale == Vec3::ONE {
            commands.entity(entity).remove::<ShieldAnimation>();
        }
    }
}

fn despawn_shield(
    mut commands: Commands,
    mut shield_query: Query<(Entity, &mut Shield)>,
    time: Res<Time>
) {
    for (entity, mut shield) in shield_query.iter_mut() {
        shield.timer.tick(time.delta());
        
        if shield.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}