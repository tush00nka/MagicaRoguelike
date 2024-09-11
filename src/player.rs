use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Startup, spawn_player)
        .add_systems(FixedUpdate, move_player);
    }
}

#[derive(Component, Clone, Copy)]
struct Player {
    pub speed: f32,
}

fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(SpriteBundle {
        texture: asset_server.load("textures/player_placeholder.png"),
        transform: Transform::from_xyz(0.0, 0.0, 1.0),
        ..default()
    })
    .insert(RigidBody::KinematicPositionBased)
    .insert(Player { speed: 100.0 });
}

fn move_player(
    mut player_query: Query<(&mut Transform, &Player)>,
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if let Ok((mut player_transform, &player)) = player_query.get_single_mut() {
        let mut direction = Vec2::new(0.0, 0.0);

        if keyboard.pressed(KeyCode::KeyA) {
            direction.x = -1.0;
        }
        else if keyboard.pressed(KeyCode::KeyD) {
            direction.x = 1.0;
        }

        if keyboard.pressed(KeyCode::KeyS) {
            direction.y = -1.0;
        }
        else if keyboard.pressed(KeyCode::KeyW) {
            direction.y = 1.0;
        }

        player_transform.translation += Vec3::new(direction.x, direction.y, 0.0).normalize_or_zero() * player.speed * time.delta_seconds();
    }
}