use avian2d::{prelude::PhysicsLayer, PhysicsPlugins};
use bevy::{
    prelude::*,
    render::{
        settings::{WgpuFeatures, WgpuSettings},
        RenderPlugin,
    },
};

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

mod mobs;
use mobs::{MobAnimationPlugin, MobMovementPlugin, MobPlugin, MobSpawnPlugin};

mod shield_spell;
use shield_spell::ShieldSpellPlugin;

mod black_hole;
use black_hole::BlackHolePlugin;

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

mod boss_room;
use boss_room::BossRoomPlugin;

mod ui;
use ui::{
    ElementsUIPlugin, ExperienceUIPlugin, HealthUIPlugin, ItemUIPlugin, LoadingScreenUIPlugin,
    MainMenuPlugin, PauseUIPlguin,
};

mod loot;
use loot::LootPlugin;

mod items;
use items::ItemEffectsPlugin;

mod obstacles;
use obstacles::ObstaclePlugin;

mod pause;
use pause::PausePlugin;

mod alert;
use alert::AlertPlugin;

mod blank_spell;
use blank_spell::BlankSpellPlugin;

mod particles;
use particles::ParticlesPlguin;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    MainMenu,
    InGame,
    Settings,
    Loading,
    GameOver,
    Hub,
    LoadingBoss,
}

#[derive(PhysicsLayer)]
pub enum GameLayer {
    Player,
    Enemy,
    Projectile,
    Wall,
    Interactable,
    Shield,
}

fn main() {
    let mut wpgu_settings = WgpuSettings::default();
    wpgu_settings
        .features
        .set(WgpuFeatures::VERTEX_WRITABLE_STORAGE, true);

    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(RenderPlugin {
                    render_creation: wpgu_settings.into(),
                    synchronous_pipeline_compilation: false,
                }),
        )
        .add_plugins(PhysicsPlugins::default())
        .init_state::<GameState>()
        .add_plugins(MainMenuPlugin)
        .add_plugins(MousePositionPlugin)
        .add_plugins(GameMapPlugin)
        .add_plugins(CameraPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(WandPlugin)
        .add_plugins((ElementsPlugin, ElementsUIPlugin))
        .add_plugins((ShieldSpellPlugin, BlackHolePlugin, BlankSpellPlugin))
        .add_plugins(ProjectilePlugin)
        .add_plugins((
            ExperiencePlugin,
            ExperienceUIPlugin,
            ExpOrbPlugin,
            ExpTankPlugin,
        ))
        .add_plugins((HealthTankPlugin, HealthUIPlugin))
        .add_plugins(PathfindingPlugin)
        .add_plugins((
            MobPlugin,
            MobAnimationPlugin,
            MobSpawnPlugin,
            MobMovementPlugin,
            AlertPlugin,
        ))
        .add_plugins(GameOverPlugin)
        .add_plugins(LevelCompletionPlugin)
        .add_plugins(HitPlayerPlugin)
        .add_plugins(HubPlugin)
        .add_plugins((InvincibilityPlugin, StunPlugin))
        .add_plugins(ChapterPlugin)
        .add_plugins((ItemPlugin, ItemUIPlugin, ItemEffectsPlugin))
        .add_plugins(LootPlugin)
        .add_plugins((PausePlugin, PauseUIPlguin))
        .add_plugins(LoadingScreenUIPlugin)
        .add_plugins(ObstaclePlugin)
        .add_plugins(BossRoomPlugin)
        .add_plugins(ParticlesPlguin)
        .run();
}
