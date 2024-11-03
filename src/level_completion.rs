use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{player::Player, utils::*, GameLayer, GameState, chapter::ChapterManager,};

pub struct LevelCompletionPlugin;

impl Plugin for LevelCompletionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PortalEvent>()
            .add_systems(Update, (spawn_portal, rotate_portal, pulsate::<Portal>))
            .add_systems(Update, collision_portal.run_if(in_state(GameState::InGame)))
            .add_systems(Update, collision_portal.run_if(in_state(GameState::Hub)))
            .add_systems(
                OnEnter(GameState::Hub),
                (
                    despawn_all_with::<crate::exp_tank::ExpTank>,
                    despawn_all_with::<crate::health_tank::HealthTank>,
                    despawn_all_with::<crate::gamemap::Floor>,
                    despawn_all_with::<crate::gamemap::Wall>,
                    despawn_all_with::<crate::exp_orb::ExpOrb>,
                    despawn_all_with::<crate::projectile::Projectile>,
                    despawn_all_with::<crate::shield_spell::Shield>,
                    despawn_all_with::<crate::black_hole::BlackHole>,
                    despawn_all_with::<crate::item::Item>,
                    despawn_all_with::<crate::obstacles::Obstacle>,
                    despawn_all_with::<Portal>,
                ),
            )
            .add_systems(
                OnExit(GameState::Hub),
                (
                    despawn_all_with::<crate::gamemap::Wall>,
                    despawn_all_with::<crate::gamemap::Floor>,
                    despawn_all_with::<crate::wand::Wand>,
                    despawn_all_with::<crate::projectile::Projectile>,
                    despawn_all_with::<crate::shield_spell::Shield>,
                    despawn_all_with::<crate::black_hole::BlackHole>,
                    despawn_all_with::<crate::item::Item>,
                    despawn_all_with::<crate::obstacles::Obstacle>,
                    despawn_all_with::<Portal>,
                ),
            )
            .insert_resource(PortalManager::default());
    }
}

#[derive(Resource)]
pub struct PortalManager {
    position: Vec3,
    pub mobs: u32, //maybe change to i32, if there would be some bugs with despawn, portal may not spawn, i suppose?
}
impl Default for PortalManager {
    fn default() -> PortalManager {
        PortalManager {
            position: Vec3 {
                x: 0.,
                y: 0.,
                z: 0.,
            },
            mobs: 0,
        }
    }
}
impl PortalManager {
    pub fn get_pos(&self) -> Vec3 {
        self.position
    }

    pub fn set_pos(&mut self, pos: Vec3) {
        self.position = pos;
    }

    pub fn pop_mob(&mut self) {
        self.mobs -= 1;
    }

    pub fn no_mobs_on_level(&self) -> bool {
        self.mobs <= 0
    }

    pub fn push_mob(&mut self) {
        self.mobs += 1;
    }
}

#[derive(Event)]
pub struct PortalEvent {
    pub pos: Vec3,
}
#[derive(Component)]
pub struct Portal;

fn spawn_portal(
    mut commands: Commands,
    mut ev_portal: EventReader<PortalEvent>,
    asset_server: Res<AssetServer>,
) {
    for ev in ev_portal.read() {
        let portal = commands
            .spawn(SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(2.0, 2.0, 2.0),
                    ..default()
                },
                texture: asset_server.load("textures/portal.png"),
                transform: Transform::from_xyz(ev.pos.x, ev.pos.y, ev.pos.z),
                ..default()
            })
            .id();
        commands
            .entity(portal)
            .insert(RigidBody::Static)
            .insert(Collider::circle(6.0))
            .insert(Sensor)
            .insert(CollisionLayers::new(
                GameLayer::Interactable,
                [GameLayer::Player],
            ))
            .insert(Portal);
    }
}

fn rotate_portal(mut portal_query: Query<&mut Transform, With<Portal>>, time: Res<Time>) {
    for mut transform in portal_query.iter_mut() {
        transform.rotate_z(time.delta_seconds());
    }
}

fn collision_portal(
    mut collision_event_reader: EventReader<Collision>,
    player_query: Query<(&Transform, Entity), (With<Player>, Without<Portal>)>,
    portal_query: Query<(&Transform, Entity), (With<Portal>, Without<Player>)>,
    mut game_state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
    chapter_manager: Res<ChapterManager>,
) {
    for Collision(contacts) in collision_event_reader.read() {
        if player_query.contains(contacts.entity2) && portal_query.contains(contacts.entity1)
            || player_query.contains(contacts.entity1) && portal_query.contains(contacts.entity2)
        {
            match current_state.get() {
                GameState::InGame => {
                    game_state.set(GameState::Hub);
                }
                GameState::Hub => {
                    if chapter_manager.get_current_chapter() == 3 {
                        game_state.set(GameState::LoadingBoss);
                    } else {
                        game_state.set(GameState::Loading);
                    }
                }
                _ => {}
            }
        }
    }
}
