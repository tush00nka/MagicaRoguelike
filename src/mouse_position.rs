use bevy::prelude::*;
use bevy::window::PrimaryWindow;

#[derive(Resource, Default)]
pub struct MouseCoords(pub Vec2); // ресурс, где всегда будет текущая позиция мыши в мире

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
    camera_query: Query<(&Camera, &GlobalTransform)>,
) {
    if let Ok((camera, camera_transform)) = camera_query.get_single() { // проверяем, есть ли камера
        if let Ok(window) = window_query.get_single() { // есть ли окно
            if let Some(world_position) = window.cursor_position() // берем позицию курсора на экране и конвертируем в позицию в мире
                .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
                .map(|ray| ray.origin.truncate()) {
                    coords.0 = world_position;
                }
        }
    }
}