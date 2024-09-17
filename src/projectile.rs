use core::f32;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::gamemap::Wall;

pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (move_projectile, hit_walls));
    }
}

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
    projectile_query: Query<(&Transform, Entity), (With<Projectile>, Without<Wall>)>,
    wall_query: Query<&Transform, (With<Wall>, Without<Projectile>)>,
) {
    for (projectile_transform, projectile_entity) in projectile_query.iter() {
        let mut closest: Vec3 = Vec3::INFINITY;
        for wall_transform in wall_query.iter() {
            if wall_transform.translation.distance(projectile_transform.translation) < closest.distance(projectile_transform.translation) {
                closest = wall_transform.translation;
            }
        }

        if closest.distance(projectile_transform.translation) <= 24.0 {
            commands.entity(projectile_entity).despawn();
        }
    }
}