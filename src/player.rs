use avian2d::prelude::*;
use bevy::prelude::*;

use crate::camera::YSort;
use crate::elements::{ElementResistance, ElementType};
use crate::friend::Friend;
use crate::health::*;
use crate::invincibility::Invincibility;
use crate::item::ItemPickupAnimation;
use crate::items::lizard_tail::DeathAvoidPopupEvent;
use crate::level_completion::PortalManager;
use crate::mobs::{HitList, MobType, SummonQueue, SummonUnit};
use crate::mouse_position::MouseCoords;
use crate::GameLayer;
use crate::{gamemap::ROOM_SIZE, GameState};

use crate::animation::AnimationConfig;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerDeathEvent>()
            .init_resource::<PlayerStats>()
            .add_systems(OnExit(GameState::MainMenu), spawn_player)
            .add_systems(OnExit(GameState::Hub), reset_player_position)
            .add_systems(OnExit(GameState::InGame), reset_player_position)
            .add_systems(
                Update,
                (
                    animate_player,
                    flip_towards_mouse,
                    debug_take_damage,
                    player_death,
                ),
            )
            .add_systems(FixedUpdate, move_player);
    }
}

#[derive(Event)]
pub struct PlayerDeathEvent(pub Entity);

#[derive(Resource)]
pub struct PlayerStats {
    pub speed: f32,
    pub damage: u32,
    pub invincibility_time: f32,
    pub projectile_deflect_chance: f32,
    pub vampirism: i32,
    pub health_regen: i32,
    pub spell_cast_hp_fee: i32,
    pub blind_rage_bonus: u32,
    pub element_damage_percent: [f32; 5],
}

impl PlayerStats {
    pub fn get_bonused_damage(&self, element: ElementType) -> u32 {
        return (self.damage as f32 * (1.0 + self.element_damage_percent[element as usize])).round()
            as u32
            + self.blind_rage_bonus;
    }
}

impl Default for PlayerStats {
    fn default() -> Self {
        Self {
            speed: 8000.,
            damage: 20,
            invincibility_time: 1.0,
            projectile_deflect_chance: 0.0,
            vampirism: 0,
            health_regen: 0,
            spell_cast_hp_fee: 0,
            blind_rage_bonus: 0,
            element_damage_percent: [0., 0., 0., 0., 0.],
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct Player;

fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    commands.init_resource::<PlayerStats>();

    let texture = asset_server.load("textures/player_walk_mantle.png");

    let layout = TextureAtlasLayout::from_grid(UVec2::splat(24), 8, 2, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    let animation_config = AnimationConfig::new(0, 7, 24);

    let player = commands
        .spawn((
            SpriteBundle {
                texture: texture.clone(),
                transform: Transform::from_xyz(
                    (ROOM_SIZE * 16) as f32,
                    (ROOM_SIZE * 16) as f32,
                    1.0,
                ),
                ..default()
            },
            TextureAtlas {
                layout: texture_atlas_layout.clone(),
                index: animation_config.first_sprite_index,
            },
            animation_config,
            YSort(9.0),
        ))
        .id();

    commands
        .entity(player)
        .insert(RigidBody::Dynamic)
        .insert(GravityScale(0.0))
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(Collider::circle(6.0))
        .insert(CollisionLayers::new(
            GameLayer::Player,
            [
                GameLayer::Wall,
                GameLayer::Interactable,
                GameLayer::Projectile,
                GameLayer::Enemy,
            ],
        ))
        .insert(LinearVelocity::ZERO)
        .insert(Player)
        .insert(Health {
            max: 100,
            current: 100,
            extra_lives: 0,
            hit_queue: vec![],
        })
        .insert(ElementResistance {
            elements: vec![],
            resistance_percent: vec![0, 0, 0, 0, 0],
        })
        .insert(SummonQueue {
            queue: vec![
                SummonUnit {
                    entity: None,
                    mob_type: MobType::Mossling
                };
                5
            ],
            amount_of_mobs: 0,
            max_amount: 5,
        })
        .insert(Friend)
        .insert(HitList::default());
}

fn move_player(
    player_stats: Res<PlayerStats>,
    mut player_query: Query<&mut LinearVelocity, With<Player>>,
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if let Ok(mut player_velocity) = player_query.get_single_mut() {
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

        player_velocity.0 =
            direction.normalize_or_zero() * player_stats.speed * time.delta_seconds();
    }
}

fn reset_player_position(mut player_query: Query<&mut Transform, With<Player>>) {
    if let Ok(mut transform) = player_query.get_single_mut() {
        transform.translation = Vec3::new((ROOM_SIZE * 16) as f32, (ROOM_SIZE * 16) as f32, 1.0);
    }
}

fn animate_player(
    time: Res<Time>,
    mut query: Query<
        (&mut AnimationConfig, &mut TextureAtlas, &LinearVelocity),
        (With<Player>, Without<ItemPickupAnimation>),
    >,
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
        } else {
            if atlas.index != config.first_sprite_index {
                atlas.index = config.first_sprite_index;
            }
        }
    }
}

fn flip_towards_mouse(
    mut player_query: Query<(&mut Sprite, &Transform), (With<Player>, Without<ItemPickupAnimation>)>,
    mouse_coords: Res<MouseCoords>,
) {
    if let Ok((mut sprite, player_transform)) = player_query.get_single_mut() {
        if player_transform.translation.x - mouse_coords.0.x > 0. {
            sprite.flip_x = true;
        } else {
            sprite.flip_x = false;
        }
    }
}

fn player_death(
    mut commands: Commands,
    mut ev_player_death: EventReader<PlayerDeathEvent>,
    mut ev_death_popup: EventWriter<DeathAvoidPopupEvent>,
    mut player_query: Query<&mut Health, With<Player>>,
    mut game_state: ResMut<NextState<GameState>>,
    mut portal_manager: ResMut<PortalManager>,
) {
    for ev in ev_player_death.read() {
        if let Ok(mut health) = player_query.get_single_mut() {
            if health.extra_lives > 0 {
                health.extra_lives -= 1;

                let heal_amount = health.max;
                health.heal(heal_amount);

                ev_death_popup.send(DeathAvoidPopupEvent);

                return;
            }

            portal_manager.set_mob(0);

            commands.entity(ev.0).despawn();
            game_state.set(GameState::GameOver);
        }
    }
}

fn debug_take_damage(
    mut commands: Commands,
    mut ev_death: EventWriter<PlayerDeathEvent>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut health_query: Query<(&mut Health, Entity), (With<Player>, Without<Invincibility>)>,
    player_stats: Res<PlayerStats>,
) {
    if keyboard.just_pressed(KeyCode::KeyZ) {
        if let Ok((mut health, ent)) = health_query.get_single_mut() {
            health.damage(25);
            commands
                .entity(ent)
                .insert(Invincibility::new(player_stats.invincibility_time));

            if health.current <= 0 {
                ev_death.send(PlayerDeathEvent(ent));
            }
        }
    }
}
