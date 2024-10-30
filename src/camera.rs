use bevy::{
    core_pipeline::bloom::BloomSettings,
    prelude::*,
};

use crate::{
    gamemap::{ROOM_SIZE, TILE_SIZE}, mouse_position::MouseCoords, player::Player, GameState
};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Msaa::Off);
        app.add_systems(Startup, spawn_camera);
        app.add_systems(OnExit(GameState::Hub), reset_player_camera);
        app.add_systems(OnExit(GameState::InGame), reset_player_camera);
        app.add_systems(Update, sync_player_camera
            .run_if(in_state(GameState::InGame)));
        app.add_systems(Update, sync_player_camera
            .run_if(in_state(GameState::Hub)));

        // app.add_systems(Update, fit_canvas);
    }
}

const CAM_LERP: f32 = 8.0;

#[derive(Component)]
pub struct InGameCamera;


fn spawn_camera(
    mut commands: Commands,
) {
    commands.spawn((Camera2dBundle {
            camera: Camera {
                hdr: true,
                order: -1,
                // target: RenderTarget::Image(image_handle.clone()),
                ..default()
            },
            projection: OrthographicProjection {
                scale: 0.5,
                ..default()
            },
            // tonemapping: Tonemapping::TonyMcMapface,
            transform: Transform::from_xyz(TILE_SIZE*(ROOM_SIZE/2) as f32, TILE_SIZE*(ROOM_SIZE/2) as f32, 10.0),
            ..default()
        },
        BloomSettings::default(),
        InGameCamera,
    )); 
}

fn reset_player_camera(
    mut camera_query: Query<&mut Transform, With<InGameCamera>>,
) {
    let Ok(mut camera_transform) = camera_query.get_single_mut() else {
        return;
    };

    camera_transform.translation = Vec3::splat(TILE_SIZE*(ROOM_SIZE/2) as f32).with_z(10.);
}

fn sync_player_camera(
    player_query: Query<&Transform, (With<Player>, Without<InGameCamera>)>,
    mut camera_query: Query<&mut Transform, (With<InGameCamera>, Without<Player>)>,
    time: Res<Time>,
    mouse_coords: Res<MouseCoords>,
) {
    let Ok(mut camera_transform) = camera_query.get_single_mut() else {
        return;
    };

    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    let direction: Vec3 = (mouse_coords.0.extend(0.0) - player_transform.translation.with_z(0.0)).normalize_or_zero();
    let target: Vec3 = player_transform.translation.with_z(10.0) + direction * 16.0;

    camera_transform.translation = camera_transform
        .translation
        .lerp(target, time.delta_seconds() * CAM_LERP); // сглаживание с помощью линейной интерполяции
}