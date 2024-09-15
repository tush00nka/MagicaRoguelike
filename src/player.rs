use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::gamemap::ROOM_SIZE;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Startup, spawn_player)
        .add_systems(FixedUpdate, move_player);
    }
}

#[derive(Component, Clone, Copy)]
pub struct Player {
    pub speed: f32,
}

fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let player = commands.spawn(SpriteBundle {
        texture: asset_server.load("textures/player_placeholder.png"),
        transform: Transform::from_xyz((ROOM_SIZE * 16) as f32, (ROOM_SIZE * 16) as f32, 1.0),
        ..default()
    }).id();

    commands.entity(player)
        .insert(RigidBody::Dynamic)
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(Collider::cuboid(8.0, 8.0))
        .insert(Velocity {
            linvel: Vec2::new(10.0, 0.0),
            angvel: 0.0,
        })
        .insert(Sleeping::disabled())
        .insert(Ccd::enabled())
        .insert(Player { speed: 10000.0 });
}

fn move_player(
    mut player_query: Query<(&mut Velocity, &Player)>,
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if let Ok((mut player_velocity, &player)) = player_query.get_single_mut() {
        let mut direction = Vec2::new(0.0, 0.0);

        if keyboard.pressed(KeyCode::KeyA) {
            direction.x -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyD) {
            direction.x += 1.0;
        }

        if keyboard.pressed(KeyCode::KeyS) {
            direction.y -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyW) {
            direction.y += 1.0;
        };

        player_velocity.linvel = direction.normalize_or_zero() * player.speed * time.delta_seconds();
    }
}
