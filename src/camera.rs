use bevy::{core_pipeline::bloom::BloomSettings, prelude::{Camera, *}};

use crate::{
    player::Player,
    GameState, TimeState
};
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera);
        app.add_systems(Update, sync_player_camera
            .run_if(in_state(GameState::InGame))
            .run_if(in_state(TimeState::Unpaused)));
    }
}

const CAM_LERP: f32 = 8.0;


#[derive(Component)]
struct PlayerCamera;

fn spawn_camera(
    mut commands: Commands,
) {
    commands.spawn((Camera2dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },
            // tonemapping: Tonemapping::TonyMcMapface,
            transform: Transform::from_xyz(0.0, 0.0, 10.0),
            projection: OrthographicProjection {
                scale: 0.6,
                ..default()
            },
            ..default()
        },
        BloomSettings {
            // prefilter_settings: BloomPrefilterSettings {
            //     threshold: 1.0,
            //     ..default()
            // } ,
            ..default()
        },
        PlayerCamera
    )); 
}

fn sync_player_camera(
    player_query: Query<&Transform, (With<Player>, Without<PlayerCamera>)>,
    mut camera_query: Query<&mut Transform, (With<PlayerCamera>, Without<Player>)>,
    time: Res<Time>,
) {
    let Ok(mut camera_transform) = camera_query.get_single_mut() else {
        return;
    };

    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    let direction = Vec3::new(player_transform.translation.x, player_transform.translation.y, camera_transform.translation.z);

    camera_transform.translation = camera_transform
        .translation
        .lerp(direction, time.delta_seconds() * CAM_LERP); // сглаживание с помощью линейной интерполяции
}