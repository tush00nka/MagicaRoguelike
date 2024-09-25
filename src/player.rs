use bevy::{ecs::entity, prelude::*};
use avian2d::prelude::*;

use crate::{gamemap::ROOM_SIZE, GameState};
use crate::health::Health;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(OnEnter(GameState::InGame), spawn_player)
        .add_systems(FixedUpdate, move_player.run_if(in_state(GameState::InGame)));
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
        .insert(GravityScale(0.0))
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(Collider::circle(8.0))
        .insert(LinearVelocity::ZERO)
        .insert(Player { speed: 10000.0 })
        .insert(Health{max: 100, current: 50});
}

fn move_player(
    mut player_query: Query<(&mut LinearVelocity, &Player)>,
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if let Ok((mut player_velocity, &player)) = player_query.get_single_mut() {
        let mut direction = Vec2::new(0.0, 0.0);

        keyboard.get_pressed().for_each(|key| {
            match key {
                KeyCode::KeyA => { direction.x = -1.0 }
                KeyCode::KeyD => { direction.x = 1.0 }
                KeyCode::KeyS => { direction.y = -1.0 }
                KeyCode::KeyW => { direction.y = 1.0 }
                _ => {}
            }
        });

        player_velocity.0 = direction.normalize_or_zero() * player.speed * time.delta_seconds();
    }
}
