use bevy::prelude::*;

use crate::player::Player;
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera);
        app.add_systems(FixedUpdate, sync_player_camera);
    }
}

#[derive(Component)]
struct Camera;

fn spawn_camera(
    mut commands: Commands,
) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 10.0),
        projection: OrthographicProjection {
            scale: 0.5,
            ..default()
        },
        ..default()
    }).insert(Camera); 
}

fn sync_player_camera(
    mut player_query: Query<&Transform, (With<Player>, Without<Camera>)>,
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Player>)>,
) {
    if let Ok(mut camera_transform) = camera_query.get_single_mut() {
        if let Ok(player_transform) = player_query.get_single_mut() {
            camera_transform.translation = Vec3::new(player_transform.translation.x, player_transform.translation.y, camera_transform.translation.z);
        }
    }
}