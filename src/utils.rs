use bevy::prelude::*;
use rand::{
    distributions::WeightedIndex,
    prelude::Distribution,
    thread_rng
};

pub fn despawn_all_with<C: Component>(query: Query<Entity, With<C>>, mut commands: Commands) {
    for e in query.iter() {
        commands.entity(e).despawn_recursive();
    }
}

pub fn pulsate<C: Component>(mut portal_query: Query<&mut Transform, With<C>>, time: Res<Time>) {
    for mut transform in &mut portal_query {
        let mut xy = time.elapsed_seconds().sin() * time.elapsed_seconds().sin();
        if xy <= 0.5 && xy + 0.05 <= 1.  {
            xy = 1. - xy + 0.05;
        }
        transform.scale = Vec3::new(xy, xy, 1.0 );
    }
}

pub fn get_random_index_with_weight(weights: Vec<usize>) -> usize {
    let distribution = WeightedIndex::new(&weights).unwrap();
    let mut rng = thread_rng();
    
    distribution.sample(&mut rng)
}

pub struct LifetimePlugin;

impl Plugin for LifetimePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, despawn_on_lifetime);
    }
}

/// Use this to automatically destroy entities over time
#[derive(Component)]
pub struct Lifetime(Timer);

impl Lifetime {
    pub fn new(duration: f32) -> Self {
        Self(Timer::from_seconds(duration, TimerMode::Once))
    }
}

pub fn despawn_on_lifetime(mut commands: Commands, mut query: Query<(Entity, &mut Lifetime)>, time: Res<Time>) {
    for (entity, mut lifetime) in query.iter_mut() {
        lifetime.0.tick(time.delta());

        if lifetime.0.just_finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}