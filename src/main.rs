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

mod hit_player;
use hit_player::HitPlayerPlugin;

mod mouse_position;
use mouse_position::MousePositionPlugin;

mod wand;
use wand::WandPlugin;

mod elements;
use elements::ElementsPlugin;

mod hub_location;
use hub_location::HubPlugin;


mod projectile;
use projectile::ProjectilePlugin;

mod experience;
use experience::ExperiencePlugin;

mod exp_orb;
use exp_orb::ExpOrbPlugin;

mod exp_tank;
use exp_tank::ExpTankPlugin;

mod health;

mod health_tank;
use health_tank::HealthTankPlugin;

mod pathfinding;
use pathfinding::PathfindingPlugin;

mod mob;
use mob::MobPlugin;

mod shield_spell;
use shield_spell::ShieldSpellPlugin;

mod game_over;
use game_over::GameOverPlugin;

mod invincibility;
use invincibility::InvincibilityPlugin;

mod stun;
use stun::StunPlugin;

mod animation;
mod utils;

mod chapter;
use chapter::ChapterPlugin;

mod item;
use item::ItemPlugin;

mod ui;
use ui::{
    ElementsUIPlugin,
    ExperienceUIPlugin,
    HealthUIPlugin,
    MainMenuPlugin,
    ItemUIPlugin,
};

mod loot;
use loot::LootPlugin;

mod items;
use items::ItemEffectsPlugin;

mod pause;
use pause::PausePlugin;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    MainMenu,
    InGame,
    Settings,
    Loading,
    GameOver,
    Hub,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum TimeState {
    #[default]
    Unpaused,
    Paused,
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
        .insert_resource(ClearColor(Color::srgb(69./255., 35./255., 13./255.)))
        .add_plugins(DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(RenderPlugin {
                render_creation: wpgu_settings.into(),
                synchronous_pipeline_compilation: false,
            }))
        .add_plugins(PhysicsPlugins::default())
        .init_state::<GameState>()
        .init_state::<TimeState>()
        .add_plugins(MainMenuPlugin)
        .add_plugins(MousePositionPlugin)
        .add_plugins(GameMapPlugin)
        .add_plugins(CameraPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(WandPlugin)
        .add_plugins((ElementsPlugin, ElementsUIPlugin))
        .add_plugins(ShieldSpellPlugin)
        .add_plugins(ProjectilePlugin)
        .add_plugins((ExperiencePlugin, ExperienceUIPlugin, ExpOrbPlugin, ExpTankPlugin))
        .add_plugins((HealthTankPlugin, HealthUIPlugin))
        .add_plugins(PathfindingPlugin)
        .add_plugins(MobPlugin)
        .add_plugins(GameOverPlugin)
        .add_plugins(LevelCompletionPlugin)
        .add_plugins(HitPlayerPlugin)
        .add_plugins(HubPlugin)
        .add_plugins((InvincibilityPlugin, StunPlugin))
        .add_plugins(ChapterPlugin)
        .add_plugins((ItemPlugin, ItemUIPlugin, ItemEffectsPlugin))
        .add_plugins(PausePlugin)
        .add_plugins(LootPlugin)
        .run();
}
