use bevy::prelude::*;

pub struct InvincibilityPlugin;

impl Plugin for InvincibilityPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_invincibility);
    }
}

#[derive(Component)]
pub struct Invincibility {
    effect_timer: Timer,
    blink_timer: Timer,
}

impl Invincibility {
    pub fn new(duration: f32) -> Self {
        Self {
            effect_timer: Timer::from_seconds(duration, TimerMode::Once),
            blink_timer: Timer::from_seconds(0.1, TimerMode::Repeating),
        }
    }
}

impl Default for Invincibility {
    fn default() -> Self {
        Self {
            effect_timer: Timer::from_seconds(1., TimerMode::Once),
            blink_timer: Timer::from_seconds(0.1, TimerMode::Repeating),
        }
    }
}

fn handle_invincibility(
    mut commands: Commands,
    mut inv_query: Query<(&mut Invincibility, Entity, &mut Sprite)>,
    time: Res<Time>,
) {
    for (mut invincibility, entity, mut sprite) in inv_query.iter_mut() {
        invincibility.effect_timer.tick(time.delta());
        invincibility.blink_timer.tick(time.delta());

        if invincibility.blink_timer.just_finished() {
            match sprite.color.alpha() {
                0.0 => sprite.color.set_alpha(1.0),
                1.0 => sprite.color.set_alpha(0.0),
                _ => {}
            }
        }

        if invincibility.effect_timer.just_finished() {
            sprite.color.set_alpha(1.0);
            commands.entity(entity).remove::<Invincibility>();
        }
    }
}