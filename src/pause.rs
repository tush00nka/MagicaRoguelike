use bevy::prelude::*;

use crate::{
    GameState,
    TimeState
};

pub struct PausePlugin;

impl Plugin for PausePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, pause
                .run_if(in_state(TimeState::Unpaused))
                .run_if(in_state(GameState::InGame)))
            .add_systems(Update, pause
                .run_if(in_state(TimeState::Unpaused))
                .run_if(in_state(GameState::Hub)))
            .add_systems(Update, unpause
                .run_if(in_state(TimeState::Paused))
                .run_if(in_state(GameState::InGame)))
            .add_systems(Update, unpause
                .run_if(in_state(TimeState::Paused))
                .run_if(in_state(GameState::Hub)));
    }
}

fn pause(
    mut time_state: ResMut<NextState<TimeState>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        time_state.set(TimeState::Paused);
    }
}

fn unpause(
    mut time_state: ResMut<NextState<TimeState>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        time_state.set(TimeState::Unpaused);
    }
}
