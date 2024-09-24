use avian2d::PhysicsPlugins;
use bevy::{prelude::*, render::{settings::{WgpuFeatures, WgpuSettings}, RenderPlugin}};

#[allow(unused)]
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

mod experience;
use experience::ExperiencePlugin;

mod exp_orb;
use exp_orb::ExpOrbPlugin;

mod exp_tank;
use exp_tank::ExpTankPlugin;

mod health;
use health::HealthPlugin;

mod main_menu;
use main_menu::MainMenuPlugin;

mod pathfinding;
#[allow(unused)]
use pathfinding::PathfindingPlugin;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    MainMenu,
    InGame,
    Settings,
    Loading
}

fn main() {

    let mut wpgu_settings = WgpuSettings::default();
    wpgu_settings.features.set(WgpuFeatures::VERTEX_WRITABLE_STORAGE, true,);

    App::new()
        .insert_resource(ClearColor(Color::hsl(24.0, 0.68, 0.16)))
        .add_plugins(DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(RenderPlugin {
                render_creation: wpgu_settings.into(),
                synchronous_pipeline_compilation: false,
            }))
        .add_plugins(PhysicsPlugins::default())
        .init_state::<GameState>()
        // .add_plugins(HanabiPlugin)
        .add_plugins(MainMenuPlugin)
        .add_plugins(MousePositionPlugin)
        .add_plugins(GameMapPlugin)
        .add_plugins(CameraPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(WandPlugin)
        .add_plugins((ElementsPlugin, ElementsUiPlugin))
        .add_plugins(ProjectilePlugin)
        .add_plugins((ExperiencePlugin, ExpOrbPlugin, ExpTankPlugin))
        .add_plugins(HealthPlugin)
        .add_plugins(PathfindingPlugin)
        .run();
}
