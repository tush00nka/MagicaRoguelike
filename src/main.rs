use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use player::PlayerPlugin;
use camera::CameraPlugin;

mod player;
mod camera;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(CameraPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .run();
}
