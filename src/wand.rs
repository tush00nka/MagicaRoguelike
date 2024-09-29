use bevy::prelude::*;

use crate::mouse_position::MouseCoords;
use crate::player::Player;
use crate::GameState;

pub struct WandPlugin;

impl Plugin for WandPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::InGame), spawn_wand)
            .add_systems(FixedUpdate, move_rotate_wand.run_if(in_state(GameState::InGame)))
            .add_systems(FixedUpdate, move_rotate_wand.run_if(in_state(GameState::Hub)));
    }
}

#[derive(Component)]
pub struct Wand;

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
    time : Res<Time>,
) {
    if let Ok(mut wand_transform) = wand_query.get_single_mut() {
        if let Ok(player_transform) = player_query.get_single_mut() {
            // двигаем за игроком
            if player_transform.translation.truncate().distance(mouse_position.0) > 8.0 &&
               wand_transform.translation.truncate().distance(mouse_position.0) > 2.0 {
                let wand_dir = (mouse_position.0 - wand_transform.translation.truncate()).normalize_or_zero() * 16.0;
                let wand_pos = player_transform.translation + Vec3::new(wand_dir.x, wand_dir.y, 1.0);
                wand_transform.translation = wand_transform.translation.lerp(wand_pos, 12.0 * time.delta_seconds());
            }
            // крутим (АААААА, ЛИНАЛ)
            let diff = Vec3::new(mouse_position.0.x, mouse_position.0.y, wand_transform.translation.z) - wand_transform.translation;
            let angle = diff.y.atan2(diff.x);
            wand_transform.rotation = wand_transform.rotation.lerp(Quat::from_rotation_z(angle), 12.0 * time.delta_seconds());
        }
    }
}

