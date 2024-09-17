use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

mod player;
use player::PlayerPlugin;

mod camera;
use camera::CameraPlugin;

mod gamemap;
use gamemap::GameMapPlugin;

mod mouse_position;
use mouse_position::MousePositionPlugin;

mod wand;
use wand::WandPlugin;

mod elements;
use elements::ElementsPlugin;

mod elements_ui;
use elements_ui::ElementsUiPlugin;

mod projectile;
use projectile::ProjectilePlugin;

mod pathfinding;
use pathfinding::PathfindingPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(MousePositionPlugin)
        .add_plugins(GameMapPlugin)
        .add_plugins(CameraPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(WandPlugin)
        .add_plugins(ElementsPlugin)
        .add_plugins(ElementsUiPlugin)
        .add_plugins(ProjectilePlugin)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .run();
}
