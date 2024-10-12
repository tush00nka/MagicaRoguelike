use core::f32;

use bevy::prelude::*;
use avian2d::prelude::*;

use crate::{elements::ElementType, gamemap::Wall, GameLayer};

pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<SpawnProjectileEvent>()
            .add_systems(Update, (spawn_projectile, move_projectile, hit_walls));
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
        let projectile = commands.spawn(ProjectileBundle {
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
        }).id();
        
        if ev.is_friendly{ //check which flag to add
            commands
                .entity(projectile)
                .insert(Friendly);
        }else{
            commands
                .entity(projectile)
                .insert(Hostile);
        }
    }
}

fn move_projectile(
    mut projectile_query: Query<(&mut Transform, &Projectile)>, 
    time: Res<Time>,
) {
    for (mut projectile_transform, projectile) in projectile_query.iter_mut() {
        projectile_transform.translation += Vec3::new(projectile.direction.x, projectile.direction.y, 0.0) * projectile.speed * time.delta_seconds();
    }
}

fn hit_walls(
    mut commands: Commands,
    mut collision_event_reader: EventReader<Collision>,
    projectile_query: Query<Entity, With<Projectile>>,
    wall_query: Query<Entity, With<Wall>>,

) {
    for Collision(contacts) in collision_event_reader.read() {
        if projectile_query.contains(contacts.entity2) && wall_query.contains(contacts.entity1) {
            commands.get_entity(contacts.entity2).unwrap().despawn();
        }

        if projectile_query.contains(contacts.entity1) && wall_query.contains(contacts.entity2) {
            commands.get_entity(contacts.entity1).unwrap().despawn();
        }
    }
}