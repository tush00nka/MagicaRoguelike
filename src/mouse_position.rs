use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::camera::InGameCamera;

/// Ресурс, содержащий текущую позицию мыши в мире
#[derive(Resource, Default)]
pub struct MouseCoords(pub Vec2); 

pub struct MousePositionPlugin;

impl Plugin for MousePositionPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(MouseCoords::default())
            .add_systems(Update, update_mouse_coords); // обновляем позицию мыши и храним в ресурсе
    }
}

fn update_mouse_coords(
    mut coords: ResMut<MouseCoords>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<InGameCamera>>,
) {
    let (camera, camera_transform) = camera_query.single();

    let Some(cursor_position) = window_query.single().cursor_position() else {
        return;
    };

    let Some(point) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    coords.0 = point;
}