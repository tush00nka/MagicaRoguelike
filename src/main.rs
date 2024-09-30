use avian2d::{prelude::PhysicsLayer, PhysicsPlugins};
use bevy::{prelude::*, render::{settings::{WgpuFeatures, WgpuSettings}, RenderPlugin}};

mod player;
use player::PlayerPlugin;

mod level_completion;
use level_completion::LevelCompletionPlugin;

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

mod hub_location;
use hub_location::HubPlugin;
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

mod health_ui;
use health_ui::HealthUIPlugin;

mod main_menu;
use main_menu::MainMenuPlugin;

mod pathfinding;
use pathfinding::PathfindingPlugin;

mod mob;
use mob::MobPlugin;

mod shield_spell;
use shield_spell::ShieldSpellPlugin;

mod game_over;
use game_over::GameOverPlugin;

mod animation;
mod utils;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    MainMenu,
    InGame,
    Settings,
    Loading,
    //SpellSelection,
    GameOver,
    Hub,
}

#[derive(PhysicsLayer)]
pub enum GameLayer {
    Player,
    Enemy,
    Projectile,
    Wall,
    Interactable,
    Shield
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
        .add_plugins(MainMenuPlugin)
        .add_plugins(MousePositionPlugin)
        .add_plugins(GameMapPlugin)
        .add_plugins(CameraPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(WandPlugin)
        .add_plugins((ElementsPlugin, ElementsUiPlugin))
        .add_plugins(ShieldSpellPlugin)
        .add_plugins(ProjectilePlugin)
        .add_plugins((ExperiencePlugin, ExpOrbPlugin, ExpTankPlugin))
        .add_plugins((HealthPlugin, HealthUIPlugin))
        .add_plugins(PathfindingPlugin)
        .add_plugins(MobPlugin)
        .add_plugins(GameOverPlugin)
        .add_plugins(LevelCompletionPlugin)
        .add_plugins(HubPlugin)
        .run();
}
