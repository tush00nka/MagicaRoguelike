use crate::{mob::PortalPosition, GameState};
use crate::GameLayer;
use avian2d::prelude::*;
use bevy::prelude::*;

pub struct LevelCompletionPlugin;

impl Plugin for LevelCompletionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PortalEvent>()
            .add_systems(Update, spawn_portal.run_if(in_state(GameState::InGame)))
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
        let portal = commands.spawn(SpriteBundle {
            texture: asset_server.load("textures/black_hole.png"),
            transform: Transform::from_xyz(ev.pos.x, ev.pos.y, ev.pos.z),
            ..default()
        }).id();
        commands
            .entity(portal)
            .insert(RigidBody::Dynamic)
            .insert(GravityScale(0.0))
            .insert(LockedAxes::ROTATION_LOCKED)
            .insert(Collider::circle(6.0))
            .insert(CollisionLayers::new(
                GameLayer::Portal,
                [
                    GameLayer::Player,
                ],
            )).insert(Portal);
    }
}