use bevy::prelude::*;
use std::time::Duration;

pub struct LoadingScreenUIPlugin;
use crate::{animation::AnimationConfig, GameState};
impl Plugin for LoadingScreenUIPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LoadingTimer::default())
            .add_systems(Startup, load_loading_screen)
            .add_systems(OnEnter(GameState::Loading), display_loading_screen)
            .add_systems(OnExit(GameState::Loading), despawn_mobs_loading_screen)
            .add_systems(
                Update,
                animate_mobs_ui.run_if(in_state(GameState::Loading))
            );
    }
}
//For debug purposes only
#[derive(Resource)]
pub struct LoadingTimer {
    timer: Timer,
}
impl Default for LoadingTimer {
    fn default() -> Self {
        let mut timer = Self {
            timer: Timer::new(Duration::from_millis(3000), TimerMode::Repeating),
        };
        timer.timer.pause();
        return timer;
    }
}
#[derive(Component)]
struct OnlyUIMobs;
#[derive(Component)]
struct LoadingScreen;

// Spawns the necessary components for the loading screen.
fn load_loading_screen(
//    mut loading_screen: Query<(Entity, &mut Visibility), With<LoadingScreen>>,
//    chapter_manager: Res<ChapterManager>,
    mut commands: Commands,
//    mut loading_timer: ResMut<LoadingTimer>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    windows: Query<&mut Window>,
) {

    let mut texture_path: Vec<&str> = vec![];
    let mut frame_count: Vec<u32> = vec![];
    let mut fps: Vec<u8> = vec![];
    //TODO: NEED TO CHOOSE DUE TO CHAPTER?
    frame_count.push(4);
    fps.push(12);
    texture_path.push("textures/mobs/mossling.png");

    frame_count.push(2);
    fps.push(3);
    texture_path.push("textures/mobs/fire_mage.png");

    frame_count.push(2);
    fps.push(3);
    texture_path.push("textures/mobs/water_mage.png");

    // Spawn the UI that will make up the loading screen.
    let parent = commands
        .spawn((
            NodeBundle {
                background_color: BackgroundColor(Color::BLACK),
                style: Style {
                    height: Val::Percent(100.0),
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                visibility: Visibility::Hidden,
                ..default()
            },
            LoadingScreen,
        ))
        .id();

    for i in 0..texture_path.len() {
        let layout = TextureAtlasLayout::from_grid(UVec2::splat(16), frame_count[i], 1, None, None);
        let texture_atlas_layout = texture_atlas_layouts.add(layout);
        //setup animation cfg
        let animation_config = AnimationConfig::new(0, frame_count[i] as usize - 1, fps[i]);
        let child = commands
            .spawn(ImageBundle {
                image: UiImage::new(asset_server.load(texture_path[i])),
                style: Style {
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    height: Val::Px((windows.single().resolution.width() * 0.15).floor()),
                    width: Val::Px((windows.single().resolution.width() * 0.15).floor()),
                    ..default()
                },
                ..default()
            })
            .insert(TextureAtlas {
                layout: texture_atlas_layout.clone(),
                index: animation_config.first_sprite_index,
            })
            .insert(animation_config)
            .insert(OnlyUIMobs)
            .id();
        commands.entity(parent).push_children(&[child]);
    }
}

fn display_loading_screen(
    mut loading_screen: Query<&mut Visibility, With<LoadingScreen>>,
//    chapter_manager: Res<ChapterManager>,
//    mut commands: Commands,
//    mut loading_timer: ResMut<LoadingTimer>,
//    asset_server: Res<AssetServer>,
//    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    *loading_screen.get_single_mut().unwrap() = Visibility::Visible;
}
fn despawn_mobs_loading_screen(
    mut loading_screen: Query<&mut Visibility, With<LoadingScreen>>,
//    mut query: Query<Entity, With<OnlyUIMobs>>,
//    mut commands: Commands,
//    mut loading_timer: ResMut<LoadingTimer>,
//    mut game_state: ResMut<NextState<GameState>>,
) {//todo: need to change mobs when chapter changes
    *loading_screen.get_single_mut().unwrap() = Visibility::Hidden;
}

fn animate_mobs_ui(
    time: Res<Time>,
    mut query: Query<(&mut AnimationConfig, &mut TextureAtlas), With<OnlyUIMobs>>,
) {
    for (mut config, mut atlas) in &mut query {
        // we track how long the current sprite has been displayed for
        config.frame_timer.tick(time.delta());

        // If it has been displayed for the user-defined amount of time (fps)...
        if config.frame_timer.just_finished() {
            if atlas.index == config.last_sprite_index {
                // ...and it IS the last frame, then we move back to the first frame and stop.
                atlas.index = config.first_sprite_index;
            } else {
                // ...and it is NOT the last frame, then we move to the next frame...
                atlas.index += 1;
                // ...and reset the frame timer to start counting all over again
                config.frame_timer = AnimationConfig::timer_from_fps(config.fps);
            }
        }
    }
}
