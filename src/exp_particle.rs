use bevy::prelude::*;

use crate::{experience::{ExpGained, PlayerExperience}, mouse_position::MouseCoords, player::Player};

pub struct ExpParticlePlugin;

impl Plugin for ExpParticlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (debug_particles, move_particles));
    }
}

#[derive(Component)]
struct ExpParticle {
    exp: u32,
}

fn debug_particles(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mouse_coords: Res<MouseCoords>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::KeyX) {
        commands.spawn(SpriteBundle {
            texture: asset_server.load("textures/exp_particle.png"),
            transform: Transform::from_xyz(mouse_coords.0.x, mouse_coords.0.y, 2.0),
            ..default()
        }).insert(ExpParticle { exp: 5 });
    }
}   

fn move_particles(
    mut commands: Commands,
    mut particles_query: Query<(&mut Transform, &ExpParticle, Entity), Without<Player>>,
    mut player_experience: ResMut<PlayerExperience>,  
    mut ev_exp_gained: EventWriter<ExpGained>,
    player_query: Query<&Transform, With<Player>>,
    time: Res<Time>,
) {
    if let Ok(player_transform) = player_query.get_single() {

        for (mut particle_transform, particle, particle_e) in particles_query.iter_mut() {

            if particle_transform.translation.distance(player_transform.translation) <= 48.0 {

                let direction = Vec3::new(player_transform.translation.x, player_transform.translation.y, particle_transform.translation.z);
                particle_transform.translation = particle_transform.translation.lerp(direction, time.delta_seconds() * 20.0);

                if particle_transform.translation.distance(player_transform.translation) <= 4.0 {
                    player_experience.give(particle.exp);
                    ev_exp_gained.send(ExpGained);
                    commands.entity(particle_e).despawn();
                }
            }
        }
    }

}
