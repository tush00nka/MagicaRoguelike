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
use mobs::{BossBehavoiurPlugin, MobAnimationPlugin, MobMovementPlugin, MobPlugin, MobSpawnPlugin};

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

mod friend;
use friend::FriendPlugin;

mod chapter;
use chapter::ChapterPlugin;

mod item;
use item::ItemPlugin;

mod boss_room;
use boss_room::BossRoomPlugin;

mod ui;
use ui::{
    ElementsUIPlugin, ExperienceUIPlugin, HealthUIPlugin, ItemUIPlugin, LoadingScreenUIPlugin,
    MainMenuPlugin, PauseUIPlguin, DebugConsolePlugin
};

mod loot;
use loot::LootPlugin;

mod items;
use items::ItemEffectsPlugin;

use seldom_state::prelude::*;

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

mod audio;
use audio::AudioPlugin;

mod save;
use save::SavePlugin;

use utils::LifetimePlugin;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    MainMenu,
    InGame,
    Loading,
    GameOver,
    Hub,
    LoadingBoss,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, SubStates)]
#[source(GameState = GameState::MainMenu)]
pub enum MainMenuState {
    #[default]
    Main,
    Settings,
    AlmanachSelection,
    ViewSpells,
    ViewItems,
    ViewMobs,
}

#[derive(PhysicsLayer)]
pub enum GameLayer {
    Player,
    Enemy,
    Projectile,
    Wall,
    Interactable,
    Shield,
    Friend,
}

fn main() {
    let mut wpgu_settings = WgpuSettings::default();
    wpgu_settings
        .features
        .set(WgpuFeatures::VERTEX_WRITABLE_STORAGE, true);

    App::new()
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
        .add_sub_state::<MainMenuState>()
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
            StateMachinePlugin,
            MobPlugin,
            MobAnimationPlugin,
            MobSpawnPlugin,
            MobMovementPlugin,
            BossBehavoiurPlugin,
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
        .add_plugins(FriendPlugin)
        .add_plugins(ParticlesPlguin)
        .add_plugins(AudioPlugin)
        .add_plugins(SavePlugin)
        .add_plugins(LifetimePlugin)
        .add_plugins(DebugConsolePlugin)
        .run();
}
