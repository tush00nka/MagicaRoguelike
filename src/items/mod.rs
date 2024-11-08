use bevy::prelude::*;

mod bacon;
use bacon::BaconPlugin;

mod speed_potion;
use speed_potion::SpeedPotionPlugin;

mod heart;
use heart::HeartPlugin;

mod amulet;
use amulet::AmuletPlugin;

pub mod lizard_tail;
use lizard_tail::LizardTailPlugin;

pub mod resistance_items;
use resistance_items::ResistanceItemsPlugin;

pub mod ghost_in_the_shell;
use ghost_in_the_shell::GhostInTheShellPlugin;

pub mod vampire_tooth;
use vampire_tooth::VampireToothPlugin;

pub mod blood_goblet;
use blood_goblet::BloodGobletPlugin;

pub struct ItemEffectsPlugin;
impl Plugin for ItemEffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            BaconPlugin,
            SpeedPotionPlugin,
            HeartPlugin,
            AmuletPlugin,
            LizardTailPlugin,
            ResistanceItemsPlugin,
            GhostInTheShellPlugin,
            VampireToothPlugin,
            BloodGobletPlugin,
        ));
    }
}