use bevy::prelude::*;

use crate::GameState;

pub struct ChapterPlugin;

impl Plugin for ChapterPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(ChapterManager::default())
            .add_systems(OnExit(GameState::MainMenu), init_chapter)
            .add_systems(OnExit(GameState::Hub), update_chapter);
    }
}

#[derive(Resource)]
pub struct ChapterManager {
    current_level: u8,
    current_chapter: u8,
    max_chapter: u8,
}

impl Default for ChapterManager {
    fn default() -> Self {
        Self {
            current_level: 1,
            current_chapter: 1,
            max_chapter: 2,
        }
    }
}

impl ChapterManager {
    pub fn get_current_chapter(&self) -> u8 {
        self.current_chapter
    }
}

fn init_chapter(
    mut commands: Commands,
) {
    commands.insert_resource(ChapterManager::default());
    commands.insert_resource(ClearColor(Color::srgb(69./255., 35./255., 13./255.)));
}

fn update_chapter(
    mut commands: Commands,
    mut chapter_manager: ResMut<ChapterManager>,
) {
    chapter_manager.current_level += 1;

    if chapter_manager.current_level > 2 {
        chapter_manager.current_level = 1;
        if chapter_manager.max_chapter > chapter_manager.current_chapter {
            chapter_manager.current_chapter += 1;

            let bg_color: Color;

            match chapter_manager.current_chapter {
                1 => bg_color = Color::srgb(69./255., 35./255., 13./255.),
                2 => bg_color = Color::srgb(31./255., 36./255., 10./255.),
                _ => bg_color = Color::WHITE,
            }

            commands.insert_resource(ClearColor(bg_color));
        }
    }
}