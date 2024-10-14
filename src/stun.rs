use avian2d::prelude::LinearVelocity;
use bevy::prelude::*;

use crate::TimeState;

pub struct StunPlugin;

impl Plugin for StunPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_stun
            .run_if(in_state(TimeState::Unpaused)));
    }
}

#[derive(Component)]
pub struct Stun {
    effect_timer: Timer,
}

impl Stun {
    pub fn new(duration: f32) -> Self {
        Self {
            effect_timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

impl Default for Stun {
    fn default() -> Self {
        Self {
            effect_timer: Timer::from_seconds(0.5, TimerMode::Once),
        }
    }
}

fn handle_stun(
    mut commands: Commands,
    mut inv_query: Query<(&mut Stun, Entity, &mut Sprite, &mut LinearVelocity)>,
    time: Res<Time>,
) {
    for (mut stun, entity, mut sprite, mut linvel) in inv_query.iter_mut() {
        stun.effect_timer.tick(time.delta());

        if !stun.effect_timer.finished() {
            sprite.color = Color::srgb(2., 1., 1.);
            linvel.0 = Vec2::ZERO;
        }

        if stun.effect_timer.just_finished() {
            sprite.color = Color::srgb(1., 1., 1.);
            commands.entity(entity).remove::<Stun>();
        }
    }
}