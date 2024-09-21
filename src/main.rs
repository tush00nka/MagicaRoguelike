use avian2d::PhysicsPlugins;
use bevy::{prelude::*, render::{settings::{WgpuFeatures, WgpuSettings}, RenderPlugin}};
use bevy_hanabi::HanabiPlugin;

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

mod health;
use health::HealthPlugin;

fn main() {

    let mut wpgu_settings = WgpuSettings::default();
    wpgu_settings.features.set(WgpuFeatures::VERTEX_WRITABLE_STORAGE, true,);

    App::new()
        .add_plugins(DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(RenderPlugin {
                render_creation: wpgu_settings.into(),
                synchronous_pipeline_compilation: false,
            }))
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(HanabiPlugin)
        .add_plugins(MousePositionPlugin)
        .add_plugins(GameMapPlugin)
        .add_plugins(CameraPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(WandPlugin)
        .add_plugins(ElementsPlugin)
        .add_plugins(ElementsUiPlugin)
        .add_plugins(ProjectilePlugin)
        .add_plugins(HealthPlugin)
        .run();
}
