use bevy::prelude::*;
use avian2d::prelude::*;

use crate::mouse_position::MouseCoords;
use crate::{health, GameLayer};
use crate::{gamemap::ROOM_SIZE, GameState};
use crate::health::*;

use crate::animation::AnimationConfig;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(OnEnter(GameState::InGame), spawn_player)
        .add_systems(FixedUpdate, move_player.run_if(in_state(GameState::InGame)))
        .add_systems(Update, (animate_player, flip_towards_mouse, take_damage).run_if(in_state(GameState::InGame)));
    }
}

#[derive(Event)]
pub struct PlayerDeathEvent;

#[derive(Component, Clone, Copy)]
pub struct Player {
    pub speed: f32,
}

fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture = asset_server.load("textures/player_walk_mantle.png");

    let layout = TextureAtlasLayout::from_grid(UVec2::splat(24), 8, 1, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    let animation_config = AnimationConfig::new(0, 7, 24);

    let player = commands.spawn((
        SpriteBundle {
            texture: texture.clone(),
            transform: Transform::from_xyz((ROOM_SIZE * 16) as f32, (ROOM_SIZE * 16) as f32, 1.0),
            ..default()
        },
        TextureAtlas {
            layout: texture_atlas_layout.clone(),
            index: animation_config.first_sprite_index,
        },
        animation_config
    )).id();

    commands.entity(player)
        .insert(RigidBody::Dynamic)
        .insert(GravityScale(0.0))
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(Collider::circle(8.0))
        .insert(CollisionLayers::new(GameLayer::Player, [GameLayer::Wall, GameLayer::Interactable, GameLayer::Projectile, GameLayer::Enemy]))
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
        let mut direction = Vec2::ZERO;

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
        }

        player_velocity.0 = direction.normalize_or_zero() * player.speed * time.delta_seconds();
    }
}

fn animate_player(
    time: Res<Time>,
    mut query: Query<(&mut AnimationConfig, &mut TextureAtlas, &LinearVelocity), With<Player>>,
) {
    for (mut config, mut atlas, linvel) in &mut query {
        if linvel.0 != Vec2::ZERO {
            // we track how long the current sprite has been displayed for
            config.frame_timer.tick(time.delta());

            // If it has been displayed for the user-defined amount of time (fps)...
            if config.frame_timer.just_finished() {
                if atlas.index == config.last_sprite_index {
                    // ...and it IS the last frame, then we move back to the first frame and stop.
                    atlas.index = config.first_sprite_index;
                } else {
                    // ...and it is NOT the last frame, then we move to the next frame...
                    atlas.index += 1;
                    // ...and reset the frame timer to start counting all over again
                    config.frame_timer = AnimationConfig::timer_from_fps(config.fps);
                }
            }
        }
        else {
            if atlas.index != config.first_sprite_index {
                atlas.index = config.first_sprite_index;
            }
        }
    }
}

fn flip_towards_mouse(
    mut player_query: Query<(&mut Sprite, &Transform), With<Player>>,
    mouse_coords: Res<MouseCoords>,
) {
    if let Ok((mut sprite, player_transform)) = player_query.get_single_mut() {
        if player_transform.translation.x - mouse_coords.0.x > 0. {
            sprite.flip_x = true;
        }
        else {
            sprite.flip_x = false;
        }
    }
}


fn take_damage(
    mut ev_death: EventWriter<DeathEvent>,
    mut ev_hp: EventWriter<PlayerHPChanged>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut health_query: Query<(&mut Health, Entity), With<Player>>
){
    if keyboard.just_pressed(KeyCode::KeyZ) {
        if let Ok((mut health, ent)) = health_query.get_single_mut(){
            health.damage(25);
            ev_hp.send(PlayerHPChanged);
            if health.current <= 0 {
                ev_death.send(DeathEvent(ent));
            }
        }
    }
}
