use avian2d::prelude::{Physics, PhysicsTime};
use bevy::prelude::*;

use crate::GameState;

pub struct PausePlugin;

impl Plugin for PausePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, pause_unpause
                .run_if(in_state(GameState::InGame)))
            .add_systems(Update, pause_unpause
                .run_if(in_state(GameState::Hub)));
    }
}

fn pause_unpause(
    mut virtual_time: ResMut<Time<Virtual>>,
    mut physics_time: ResMut<Time<Physics>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        if virtual_time.is_paused()
        || physics_time.is_paused() {
            virtual_time.unpause();
            physics_time.unpause();
        }
        else {
            virtual_time.pause();
            physics_time.pause();
        }
    }
}