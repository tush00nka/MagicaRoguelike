use bevy::prelude::*;

mod bacon;
use bacon::BaconPlugin;

mod speed_potion;
use speed_potion::SpeedPotionPlugin;

mod heart;
use heart::HeartPlugin;

mod amulet;
use amulet::AmuletPlugin;

pub struct ItemEffectsPlugin;
impl Plugin for ItemEffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            BaconPlugin,
            SpeedPotionPlugin,
            HeartPlugin,
            AmuletPlugin,
        ));
    }
}