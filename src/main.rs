use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

mod player;
use player::PlayerPlugin;

mod camera;
use camera::CameraPlugin;

mod gamemap;
use gamemap::GameMapPlugin;


fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(GameMapPlugin)
        .add_plugins(CameraPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .run();
}
