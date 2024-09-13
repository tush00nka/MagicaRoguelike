use bevy::prelude::*;

use crate::mouse_position::MouseCoords;
use crate::player::Player;

pub struct WandPlugin;

impl Plugin for WandPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, spawn_wand)
            .add_systems(Update, move_rotate_wand);
    }
}

#[derive(Component)]
struct Wand;

fn spawn_wand( // спавним палку
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(SpriteBundle {
        texture: asset_server.load("textures/wand.png"),
        ..default()
    }).insert(Wand);
}

fn move_rotate_wand(
    mut wand_query: Query<&mut Transform, (With<Wand>, Without<Player>)>, 
    mut player_query: Query<&Transform, (With<Player>, Without<Wand>)>,
    mouse_position: Res<MouseCoords>,
) {
    if let Ok(mut wand_transform) = wand_query.get_single_mut() {
        if let Ok(player_transform) = player_query.get_single_mut() {
            // двигаем за игроком
            wand_transform.translation = player_transform.translation;
            // крутим (АААААА, ЛИНАЛ)
            let diff = Vec3::new(mouse_position.0.x, mouse_position.0.y, wand_transform.translation.z) - wand_transform.translation;
            let angle = diff.y.atan2(diff.x);
            wand_transform.rotation = Quat::from_axis_angle(Vec3::new(0., 0., 1.), angle)
        }
    }
}

