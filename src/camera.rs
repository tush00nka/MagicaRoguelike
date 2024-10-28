use bevy::{core_pipeline::bloom::BloomSettings, prelude::{Camera, *}};

use crate::{
    gamemap::{ROOM_SIZE, TILE_SIZE}, player::Player, GameState, TimeState
};
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera);
        app.add_systems(OnExit(GameState::Hub), reset_player_camera);
        app.add_systems(OnExit(GameState::InGame), reset_player_camera);
        app.add_systems(Update, sync_player_camera
            .run_if(in_state(GameState::InGame))
            .run_if(in_state(TimeState::Unpaused)));
        app.add_systems(Update, sync_player_camera
            .run_if(in_state(GameState::Hub))
            .run_if(in_state(TimeState::Unpaused)));
    }
}

const CAM_LERP: f32 = 8.0;


#[derive(Component)]
struct PlayerCamera {
    follow_x: bool,
    follow_y: bool,
}

impl Default for PlayerCamera {
    fn default() -> Self {
        Self {
            follow_x: true,
            follow_y: true,
        }
    }
}

impl PlayerCamera {
    fn reset(&mut self) {
        self.follow_x = true;
        self.follow_y = true;
    }
}

fn spawn_camera(
    mut commands: Commands,
) {
    commands.spawn((Camera2dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },
            // tonemapping: Tonemapping::TonyMcMapface,
            transform: Transform::from_xyz(TILE_SIZE*(ROOM_SIZE/2) as f32, TILE_SIZE*(ROOM_SIZE/2) as f32, 10.0),
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
        PlayerCamera::default()
    )); 
}

fn reset_player_camera(
    mut camera_query: Query<(&mut Transform, &mut PlayerCamera)>,
) {
    let Ok((mut camera_transform, mut player_cam)) = camera_query.get_single_mut() else {
        return;
    };

    camera_transform.translation = Vec3::splat(TILE_SIZE*(ROOM_SIZE/2) as f32).with_z(10.);
    player_cam.reset();
}

fn sync_player_camera(
    player_query: Query<&Transform, (With<Player>, Without<PlayerCamera>)>,
    mut camera_query: Query<(&mut Transform, &PlayerCamera), Without<Player>>,
    time: Res<Time>,
) {
    let Ok((mut camera_transform, camera)) = camera_query.get_single_mut() else {
        return;
    };

    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    let mut direction: Vec3 = camera_transform.translation;

    if camera.follow_x {
        direction.x = player_transform.translation.x;
    }
    
    if camera.follow_y {
        direction.y = player_transform.translation.y;
    }


    camera_transform.translation = camera_transform
        .translation
        .lerp(direction, time.delta_seconds() * CAM_LERP); // сглаживание с помощью линейной интерполяции
}

#[allow(unused)]
fn player_camera_border(
    window_query: Query<&Window, With<bevy::window::PrimaryWindow>>,
    player_query: Query<&GlobalTransform, With<Player>>,
    mut camera_query: Query<(&Camera, &mut PlayerCamera)>,
) {
    let Ok(player_global_transform) = player_query.get_single() else {
        return;
    };

    let Ok((camera, mut player_cam)) = camera_query.get_single_mut() else {
        return;
    };

    let Ok(window) = window_query.get_single() else {
        return;
    };

    let tr = camera.viewport_to_world(player_global_transform, window.size())
        .map(|ray| ray.origin.truncate())
        .unwrap();

    let bl = camera.viewport_to_world(player_global_transform, Vec2::ZERO)
        .map(|ray| ray.origin.truncate())
        .unwrap();

    if tr.x > TILE_SIZE * (ROOM_SIZE as f32)
    || bl.x < 0. {
        player_cam.follow_x = false;
    }
    else {
        player_cam.follow_x = true;
    }

    if tr.y > TILE_SIZE * (ROOM_SIZE as f32)
    || bl.y < 0. {
        player_cam.follow_y = false;
    }
    else {
        player_cam.follow_y = true;
    } 
}