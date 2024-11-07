use core::f32;
use std::f32::consts::PI;

use bevy::prelude::*;
use avian2d::prelude::*;
use rand::Rng;

use crate::{
    blank_spell::Blank, elements::ElementType, gamemap::Wall, particles::SpawnParticlesEvent, shield_spell::Shield, GameLayer
};

pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<SpawnProjectileEvent>()
            .add_systems(Update, (spawn_projectile, move_projectile, hit_walls, hit_shield));
    }
}

#[derive(Component)]
pub struct Hostile;

#[derive(Component)]
pub struct Friendly; //tags for projs

#[allow(dead_code)]
#[derive(Component)]
pub struct Projectile {
    pub direction: Vec2,
    pub speed: f32,
    pub damage: u32,
    pub element: ElementType,
}

#[derive(Bundle)]
pub struct ProjectileBundle {
    pub sprite: SpriteBundle,
    pub projectile: Projectile,
    pub collider: Collider,
    pub collision_layers: CollisionLayers,
    pub sensor: Sensor,
}

impl Default for ProjectileBundle {
    fn default() -> Self {
        Self {
            sprite: SpriteBundle::default(),
            projectile: Projectile {
                direction: Vec2::X,
                speed: 100.0,
                damage: 100,
                element: ElementType::Air,
            },
            collider: Collider::circle(8.0),
            collision_layers: CollisionLayers::new(GameLayer::Projectile, [GameLayer::Enemy, GameLayer::Player, GameLayer::Wall]),
            sensor: Sensor
        }
    }
}

#[derive(Event)]
pub struct SpawnProjectileEvent {
    pub texture_path: String,
    pub color: Color,
    pub translation: Vec3,
    pub angle: f32,
    pub radius: f32,
    pub speed: f32,
    pub damage: u32,
    pub element: ElementType,
    pub is_friendly: bool,
}

fn spawn_projectile(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut ev_projectile_spawn: EventReader<SpawnProjectileEvent>,
) {
    for ev in ev_projectile_spawn.read() {
        let mut projectile = commands.spawn(ProjectileBundle {
            sprite: SpriteBundle {
                transform: Transform {
                    translation: ev.translation,
                    rotation: Quat::from_rotation_z(ev.angle),
                    ..default()
                },
                texture: asset_server.load(ev.texture_path.clone()),
                sprite: Sprite {
                    color: ev.color,
                    ..default()
                },
                ..default()
            },

            projectile: Projectile {
                direction: Vec2::from_angle(ev.angle),
                speed: ev.speed,
                damage: ev.damage,
                element: ev.element,
            },
            collider: Collider::circle(ev.radius),
            ..default()
        });
        
        if ev.is_friendly{ //check which flag to add
            projectile.insert(Friendly);
        }else{
            projectile
                .insert(Hostile)
                .insert(CollisionLayers::new(
                    GameLayer::Projectile,
                    [
                        GameLayer::Enemy,
                        GameLayer::Player,
                        GameLayer::Wall,
                        GameLayer::Shield
                    ]
                ));
        }
    }
}

fn move_projectile(
    mut projectile_query: Query<(&mut Transform, &Projectile)>, 
    time: Res<Time>,
) {
    for (mut projectile_transform, projectile) in projectile_query.iter_mut() {
        projectile_transform.translation += Vec3::new(projectile.direction.x, projectile.direction.y, 0.0) * projectile.speed * time.delta_seconds();
        projectile_transform.rotation = Quat::from_rotation_z(projectile.direction.to_angle());
    }
}

fn hit_shield(
    mut commands: Commands,
    projectile_query: Query<(Entity, &CollidingEntities, &Projectile, &Transform), With<Hostile>>,
    shield_query: Query<Entity, Or<(With<Shield>, With<Blank>)>>,
    mut ev_spawn_particles: EventWriter<SpawnParticlesEvent>,
) {
    for (proj_e, colliding_e, projectile, transform) in projectile_query.iter() {
        for shield_e in shield_query.iter() {
            if colliding_e.contains(&shield_e) {
                commands.entity(proj_e).despawn();

                ev_spawn_particles.send(SpawnParticlesEvent {
                    pattern: crate::particles::ParticlePattern::Burst {
                        direction: -projectile.direction,
                        distance: rand::thread_rng().gen_range(8.0..12.0),
                        spread: PI/3.,
                    },
                    position: transform.translation,
                    amount: 3,
                    color: projectile.element.color(),
                    speed: 10.,
                    rotate: false,
                });
            }
        }
    }
}

fn hit_walls(
    mut commands: Commands,
    projectile_query: Query<(Entity, &CollidingEntities, &Projectile, &Transform)>,
    wall_query: Query<Entity, With<Wall>>,
    mut ev_spawn_particles: EventWriter<SpawnParticlesEvent>,
) {
    for (proj_e, colliding_e, projectile, transform) in projectile_query.iter() {
        for wall_e in wall_query.iter() {
            if colliding_e.contains(&wall_e) {
                commands.entity(proj_e).despawn();

                ev_spawn_particles.send(SpawnParticlesEvent {
                    pattern: crate::particles::ParticlePattern::Burst {
                        direction: -projectile.direction,
                        distance: rand::thread_rng().gen_range(8.0..12.0),
                        spread: PI/3.,
                    },
                    position: transform.translation,
                    amount: 3,
                    color: projectile.element.color(),
                    speed: 10.,
                    rotate: false,
                });
            }
        }
    }
}