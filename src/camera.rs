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
        app.add_event::<CameraShakeEvent>();
        app.add_systems(Startup, spawn_camera);
        app.add_systems(OnExit(GameState::Hub), reset_player_camera);
        app.add_systems(OnExit(GameState::InGame), reset_player_camera);
        app.add_systems(Update, (sync_player_camera, init_shake_player_camera, shake_player_camera, y_sort)
            .run_if(in_state(GameState::InGame)
            .or_else(in_state(GameState::Hub))));

        // app.add_systems(Update, fit_canvas);
    }
}

const CAM_LERP: f32 = 8.0;

#[derive(Component)]
pub struct InGameCamera;

#[derive(Component)]
struct Shake;

#[derive(Event)]
pub struct CameraShakeEvent;

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

fn init_shake_player_camera (
    mut commands: Commands,
    mut ev_shake: EventReader<CameraShakeEvent>,
    mut camera_query: Query<(Entity, &mut OrthographicProjection), With<InGameCamera>>,
) {
    for _ev in ev_shake.read() {
        let Ok((entity, mut projection)) = camera_query.get_single_mut() else {
            return;
        };

        projection.scale = 0.4;
        commands.entity(entity).insert(Shake);
    }
}

fn shake_player_camera(
    mut commands: Commands,
    mut camera_query: Query<(Entity, &mut OrthographicProjection), (With<InGameCamera>, With<Shake>)>,
    time: Res<Time>,
) {
    let Ok((entity, mut projection)) = camera_query.get_single_mut() else {
        return;
    };

    projection.scale = projection.scale.lerp(0.5, 10.0 * time.delta_seconds());

    if projection.scale >= 0.5 {
        commands.entity(entity).remove::<Shake>();
    }
}

/// Component to sort entities by their y position.
/// Takes in a base value usually the sprite default Z with possibly a height offset.
/// this value could be tweaked to implement virtual Z for jumping
#[derive(Component)]
pub struct YSort(pub f32);

/// Applies the y-sorting to the entities Z position.
pub fn y_sort(mut query: Query<(&mut Transform, &YSort)>) {
    for (mut transform, ysort) in query.iter_mut() {
        transform.translation.z = 0.01 * (ysort.0 - transform.translation.y);
    }
}