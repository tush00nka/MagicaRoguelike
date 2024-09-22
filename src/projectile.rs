use core::f32;

use bevy::prelude::*;
use avian2d::prelude::*;

use crate::gamemap::Wall;

pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (move_projectile, hit_walls));
    }
}

#[allow(unused)]
#[derive(Component)]
pub struct Projectile {
    pub direction: Vec2,
    pub speed: f32,
    pub damage: i32,
    pub is_friendly: bool
}

#[derive(Bundle)]
pub struct ProjectileBundle {
    pub sprite: SpriteBundle,
    pub projectile: Projectile,
    pub collider: Collider,
    pub sensor: Sensor,
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
    }
}