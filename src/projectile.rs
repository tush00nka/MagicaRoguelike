use bevy::prelude::*;

pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, move_projectile);
    }
}

#[derive(Component)]
pub struct Projectile {
    pub direction: Vec2,
    pub speed: f32,
}

fn move_projectile(
    mut projectile_query: Query<(&mut Transform, &Projectile)>, 
    time: Res<Time>,
) {
    for (mut projectile_transform, projectile) in projectile_query.iter_mut() {
        projectile_transform.translation += Vec3::new(projectile.direction.x, projectile.direction.y, 0.0) * projectile.speed * time.delta_seconds();
    }
}