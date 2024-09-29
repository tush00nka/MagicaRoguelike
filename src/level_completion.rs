use crate::player::Player;
use crate::GameLayer;
use crate::{mob::PortalPosition, GameState};
use avian2d::prelude::*;
use bevy::prelude::*;

pub struct LevelCompletionPlugin;

impl Plugin for LevelCompletionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PortalEvent>()
        .add_systems(Update, spawn_portal.run_if(in_state(GameState::InGame)))
        .add_systems(Update, spawn_portal.run_if(in_state(GameState::Hub)))
            .add_systems(Update, collision_portal.run_if(in_state(GameState::InGame)))
            .add_systems(Update, collision_portal.run_if(in_state(GameState::Hub)))
            .add_systems(
                OnEnter(GameState::Hub),
                despawn_all_with::<crate::exp_tank::ExpTank>,
            )
            .add_systems(
                OnEnter(GameState::Hub),
                despawn_all_with::<crate::health::HealthTank>,
            )
            .add_systems(
                OnEnter(GameState::Hub),
                despawn_all_with::<crate::gamemap::Floor>,
            )
            .add_systems(
                OnEnter(GameState::Hub),
                despawn_all_with::<crate::gamemap::Wall>,
            )
            .add_systems(
                OnEnter(GameState::Hub),
                despawn_all_with::<crate::exp_orb::ExpOrb>,
            )
            .add_systems(
                OnExit(GameState::Hub),
                despawn_all_with::<crate::gamemap::Wall>,
            )
            .add_systems(
                OnExit(GameState::Hub),
                despawn_all_with::<crate::gamemap::Floor>,
            )
            .add_systems(
                OnExit(GameState::Hub),
                despawn_all_with::<Player>,
            )
            .add_systems(
                OnExit(GameState::Hub),
                despawn_all_with::<crate::wand::Wand>,
            )//need to delete and despawn: levelgen, exp particles, portal in hub, maybe something else, need to check
            .add_systems(OnEnter(GameState::Hub), despawn_all_with::<Portal>)
            .add_systems(OnExit(GameState::Hub), despawn_all_with::<Portal>)
            .insert_resource(PortalPosition::default());
    }
}

#[derive(Event)]
pub struct PortalEvent {
    pub pos: Vec3,
}
#[derive(Component)]
struct Portal;

fn spawn_portal(
    mut commands: Commands,

    mut ev_portal: EventReader<PortalEvent>,
    asset_server: Res<AssetServer>,
) {
    for ev in ev_portal.read() {
        let portal = commands
            .spawn(SpriteBundle {
                texture: asset_server.load("textures/black_hole.png"),
                transform: Transform::from_xyz(ev.pos.x, ev.pos.y, ev.pos.z),
                ..default()
            })
            .id();
        commands
            .entity(portal)
            .insert(RigidBody::Static)
            .insert(GravityScale(0.0))
            .insert(LockedAxes::ROTATION_LOCKED)
            .insert(Collider::circle(6.0))
            .insert(CollisionLayers::new(
                GameLayer::Interactable,
                [GameLayer::Player],
            ))
            .insert(Portal);
    }
}

fn collision_portal(
    mut collision_event_reader: EventReader<Collision>,
    player_query: Query<(&Transform, Entity), With<Player>>,
    portal_query: Query<(&Transform, Entity), With<Portal>>,
    mut game_state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
    mut amount_mobs: ResMut<PortalPosition>
) {
    for Collision(contacts) in collision_event_reader.read() {
        if player_query.contains(contacts.entity2) && portal_query.contains(contacts.entity1)
            || player_query.contains(contacts.entity1) && portal_query.contains(contacts.entity2)
        {
            match (current_state.get()) {
                GameState::InGame => {
                    game_state.set(GameState::Hub);
                }
                GameState::Hub =>{
                    game_state.set(GameState::Loading);
                    amount_mobs.check = false;
                }
                _ => {}
            }
        }
    }
}
fn despawn_all_with<C: Component>(query: Query<Entity, With<C>>, mut commands: Commands) {
    for e in query.iter() {
        commands.entity(e).despawn();
    }
}
