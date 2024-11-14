use std::time::Duration;

use bevy::prelude::*;
use avian2d::prelude::*;

use crate::{friend::Friend, player::Player, GameLayer};

pub struct ShieldSpellPlugin;

impl Plugin for ShieldSpellPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_event::<SpawnShieldEvent>()
        .add_systems(Update, (spawn_shield, animate_shield, despawn_shield));
    }
}

#[derive(Component)]
pub struct Shield {
    pub effect_timer: Timer,
    pub blink_timer: Timer,
}

#[derive(Component)]
pub struct ShieldAnimation {
    pub speed: f32
}

#[derive(Event)]
pub struct SpawnShieldEvent {
    pub duration: f32,
}

fn spawn_shield(
    mut commands: Commands,
    mut ev_spawn_shield: EventReader<SpawnShieldEvent>,
    player_query: Query<(Entity, &Transform), With<Player>>,
    asset_server: Res<AssetServer>
) {
    for ev in ev_spawn_shield.read() {
        if let Ok((player_e, player_transform)) = player_query.get_single() {
            let shield_e = commands.spawn(SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(2.0, 2.0, 2.0),
                    ..default()
                },
                texture: asset_server.load("textures/shield.png"),
                transform: Transform {
                    scale: Vec3::splat(0.1),
                    translation: player_transform.translation,
                    ..default()
                },
                ..default()
            })
            .insert(Shield {
                effect_timer: Timer::new(Duration::from_secs_f32(ev.duration), TimerMode::Once),
                blink_timer: Timer::new(Duration::from_secs_f32(0.1), TimerMode::Repeating)
            })
            .insert(ShieldAnimation { speed: 25.0 })
            .insert(RigidBody::Dynamic)
            .insert(GravityScale(0.0))
            .insert(Collider::circle(16.0))
            .insert(CollisionLayers::new(GameLayer::Shield, [GameLayer::Enemy, GameLayer::Projectile]))
            .insert(Friend).id();

            commands.spawn(FixedJoint::new(player_e, shield_e));
        }
    }
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
    mut shield_query: Query<(Entity, &mut Shield, &mut Sprite)>,
    time: Res<Time>
) {
    for (entity, mut shield, mut sprite) in shield_query.iter_mut() {
        shield.effect_timer.tick(time.delta());
        
        if shield.effect_timer.fraction_remaining() <= 0.25 {
            shield.blink_timer.tick(time.delta());
        }

        if shield.blink_timer.just_finished() {
            match sprite.color.alpha() {
                0.0 => sprite.color.set_alpha(1.0),
                1.0 => sprite.color.set_alpha(0.0),
                _ => {}
            }
        }

        if shield.effect_timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}