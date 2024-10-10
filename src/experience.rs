use bevy::prelude::*;

use crate::GameState;

pub struct ExperiencePlugin;

impl Plugin for ExperiencePlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(PlayerExperience::default())
            .add_systems(OnExit(GameState::MainMenu), init_experience)
            .add_event::<ExpGained>();
    }
}

#[derive(Resource)]
pub struct PlayerExperience {
    pub current: u32,
    pub to_lv_up: u32,
    pub lv: u8,
    max_lv: u8,
    pub orb_bonus: u32,
}

impl Default for PlayerExperience {
    fn default() -> Self {
        Self {
            current: 0,
            to_lv_up: 100,
            lv: 1,
            max_lv: 9,
            orb_bonus: 0
        }
    }
}

impl PlayerExperience {
    pub fn give(&mut self, value: u32) {
        if self.current + value >= self.to_lv_up && self.lv < self.max_lv{
            self.lv += 1;
            self.current = self.current + value - self.to_lv_up; 
            self.to_lv_up = (self.to_lv_up as f32 * 1.4) as u32;
        }
        else {
            self.current += value;
        }
    }
}

#[derive(Event)]
pub struct ExpGained;

fn init_experience(
    mut commands: Commands, 
) {
    commands.insert_resource(PlayerExperience::default());
}