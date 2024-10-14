use bevy::prelude::*;

pub struct LoadingScreenUIPlugin;
use crate::{animation::AnimationConfig, chapter::ChapterManager, GameState};
impl Plugin for LoadingScreenUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (load_loading_screen,despawn_mobs_loading_screen.after(load_loading_screen)))
            .add_systems(
                OnEnter(GameState::Loading),
                (display_loading_screen, animate_mobs_ui),
            )
            .add_systems(OnExit(GameState::Loading), despawn_mobs_loading_screen);
    }
}
#[derive(Component)]
struct OnlyUIMobs;
#[derive(Component)]
struct LoadingScreen;

// Spawns the necessary components for the loading screen.
fn load_loading_screen(mut commands: Commands) {
    let text_style = TextStyle {
        font_size: 80.0,
        ..default()
    };
    // Spawn the UI that will make up the loading screen.
    commands
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
                ..default()
            },
            LoadingScreen,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_sections([TextSection::new(
                "Loading...",
                text_style.clone(),
            )]));
        });
}

fn display_loading_screen(
    mut loading_screen: Query<&mut Visibility, With<LoadingScreen>>,
    chapter_manager: Res<ChapterManager>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    *loading_screen.get_single_mut().unwrap() = Visibility::Visible;
    let mut texture_path: Vec<&str> = vec![];
    let mut frame_count: Vec<u32> = vec![];
    let mut fps: Vec<u8> = vec![];

    match chapter_manager.get_current_chapter() {
        1 => {
            frame_count.push(4);
            fps.push(12);
            texture_path.push("textures/mobs/mossling.png");

            frame_count.push(2);
            fps.push(3);
            texture_path.push("textures/mobs/fire_mage.png");

            frame_count.push(2);
            fps.push(3);
            texture_path.push("textures/mobs/water_mage.png");
        }
        2 => {}
        3 => {}
        _ => {}
    }
    for i in 0..texture_path.len() {
        let texture = asset_server.load(texture_path[i]);

        let layout = TextureAtlasLayout::from_grid(UVec2::splat(16), frame_count[i], 1, None, None);
        let texture_atlas_layout = texture_atlas_layouts.add(layout);
        //setup animation cfg
        let animation_config = AnimationConfig::new(0, frame_count[i] as usize - 1, fps[i]);
        //spawn mob with texture
        let mob = commands
            .spawn(SpriteBundle {
                texture,
                transform: Transform::from_xyz(300. * (i + 1) as f32, 300., 1.0),
                ..default()
            })
            .id();

        commands
            .entity(mob) //todo: change it that we could test mobs without animations
            .insert(TextureAtlas {
                layout: texture_atlas_layout.clone(),
                index: animation_config.first_sprite_index,
            })
            .insert(animation_config)
            .insert(OnlyUIMobs);
    }
}

fn despawn_mobs_loading_screen(
    mut loading_screen: Query<&mut Visibility, With<LoadingScreen>>,
    mut query: Query<Entity, With<OnlyUIMobs>>,
    mut commands: Commands,
) {
    *loading_screen.get_single_mut().unwrap() = Visibility::Hidden;
    for mob in &mut query{
        commands.entity(mob).despawn();
    }
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
/*
match mob_type {
    MobType::Mossling => {
        frame_count = 4;
        fps = 12;
        texture_path = "textures/mobs/mossling.png";
    }
    MobType::FireMage => {
        texture_path = "textures/mobs/fire_mage.png";
        frame_count = 2;
        fps = 3;
    }
    MobType::WaterMage => {
        frame_count = 2;
        fps = 3;
        texture_path = "textures/mobs/water_mage.png";
    }
}
//get texture and layout
let texture = asset_server.load(texture_path);

let layout =
    TextureAtlasLayout::from_grid(UVec2::splat(16), frame_count, 1, None, None);
let texture_atlas_layout = texture_atlas_layouts.add(layout);
//setup animation cfg
let animation_config = AnimationConfig::new(0, frame_count as usize - 1, fps);
//spawn mob with texture
let mob = commands
    .spawn(SpriteBundle {
        texture,
        transform: Transform::from_xyz(
            (i as i32 * ROOM_SIZE) as f32,
            (j as i32 * ROOM_SIZE) as f32,
            1.0,
        ),
        ..default()
    })
    .id();

commands
.entity(mob) //todo: change it that we could test mobs without animations
.insert(TextureAtlas {
    layout: texture_atlas_layout.clone(),
    index: animation_config.first_sprite_index,
})
.insert(animation_config);
 */
