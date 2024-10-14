use avian2d::prelude::LinearVelocity;
use bevy::prelude::*;

pub fn despawn_all_with<C: Component>(query: Query<Entity, With<C>>, mut commands: Commands) {
    for e in query.iter() {
        commands.entity(e).despawn_recursive();
    }
}

pub fn clear_velocity_for<C: Component>(
    mut query: Query<&mut LinearVelocity, With<C>>,
) {
    for mut linvel in query.iter_mut() {
        linvel.0 = Vec2::ZERO;
    }
}