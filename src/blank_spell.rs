use bevy::prelude::*;
use avian2d::prelude::*;

use crate::{player::Player, GameLayer};

pub struct BlankSpellPlugin;

impl Plugin for BlankSpellPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_event::<SpawnBlankEvent>()
        .add_systems(Update, (spawn_blank, animate_blank, despawn_blank));
    }
}

#[derive(Component)]
pub struct Blank {
    pub range: f32,
    pub speed: f32,
}

#[derive(Component)]
pub struct BlankFader {
    pub speed: f32,
}

#[derive(Event)]
pub struct SpawnBlankEvent {
    pub range: f32,
    pub speed: f32,
}

fn spawn_blank(
    mut commands: Commands,
    mut ev_spawn_shield: EventReader<SpawnBlankEvent>,
    player_query: Query<&Transform, With<Player>>,
    asset_server: Res<AssetServer>
) {
    for ev in ev_spawn_shield.read() {
        if let Ok(player_transform) = player_query.get_single() {
            commands.spawn(SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(1.0, 2.0, 1.0),
                    ..default()
                },
                texture: asset_server.load("textures/blank_spell.png"),
                transform: Transform {
                    scale: Vec3::splat(0.1),
                    translation: player_transform.translation,
                    ..default()
                },
                ..default()
            })
            .insert(Blank {
                range: ev.range,
                speed: ev.speed,
            })
            .insert(RigidBody::Static)
            .insert(GravityScale(0.0))
            .insert(Collider::circle(16.0))
            .insert(Sensor)
            .insert(CollisionLayers::new(GameLayer::Shield, [GameLayer::Projectile]));
        }
    }
}

fn animate_blank(
    mut commands: Commands,
    mut animation_query: Query<(Entity, &Blank, &mut Transform), Without<BlankFader>>,
    time: Res<Time>,
) {
    for (entity, blank, mut transform) in animation_query.iter_mut() {
        transform.scale = transform.scale.lerp(Vec3::splat(blank.range), blank.speed * time.delta_seconds());
        if transform.scale.distance(Vec3::splat(blank.range)) <= 2. {
            commands.entity(entity).insert(BlankFader { speed: 10.0 });
        }
    }
}

fn despawn_blank(
    mut commands: Commands,
    mut shield_query: Query<(Entity, &BlankFader, &mut Sprite), With<Blank>>,
    time: Res<Time>
) {
    for (entity, fader, mut sprite) in shield_query.iter_mut() {
        let alpha = sprite.color.alpha();
        sprite.color.set_alpha(alpha.lerp(0.0, fader.speed * time.delta_seconds()));

        if sprite.color.alpha() <= 0.1 {
            commands.entity(entity).despawn();
        }
    }
}