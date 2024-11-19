use bevy::prelude::*;

use crate::{animation::AnimationConfig, audio::PlayAudioEvent, player::Player, GameState};

pub struct ExperiencePlugin;

impl Plugin for ExperiencePlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(PlayerExperience::default())
            .add_systems(OnExit(GameState::MainMenu), init_experience)
            .add_systems(Update, (lvl_up_popup, animate_popup))
            .add_event::<ExpGained>();
    }
}

#[derive(Resource)]
pub struct PlayerExperience {
    pub current: u32,
    pub to_lv_up: u32,
    pub lv: u8,
    max_lv: u8,
    popup_flag: bool,
    pub orb_bonus: u32,
}

impl Default for PlayerExperience {
    fn default() -> Self {
        Self {
            current: 0,
            to_lv_up: 100,
            lv: 1,
            max_lv: 9,
            popup_flag: false,
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
            self.popup_flag = true;
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

#[derive(Component)]
struct Popup;

fn lvl_up_popup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut exp: ResMut<PlayerExperience>,
    player_query: Query<Entity, With<Player>>,

    mut ev_play_audio: EventWriter<PlayAudioEvent>,
) {
    let Ok(player_e) = player_query.get_single() else{
        return;
    };

    if exp.popup_flag {
        exp.popup_flag = false;

        ev_play_audio.send(PlayAudioEvent::from_file("lvlup.ogg"));

        let texture = asset_server.load("textures/lvl_up_popup.png");
        let layout = TextureAtlasLayout::from_grid(UVec2::splat(16), 8, 1, None, None);
        let texture_atlas_layout = texture_atlas_layouts.add(layout);

        let config = AnimationConfig::new(0, 7, 8);

        commands.entity(player_e).with_children(|parent| {
            parent.spawn((
                SpriteBundle {
                    texture: texture.clone(),
                    transform: Transform::from_xyz(0.0, 16.0, 0.1),
                    ..default()
                },
                TextureAtlas {
                    layout: texture_atlas_layout.clone(),
                    index: config.first_sprite_index,
                },
                config,
                Popup
            ));
        });
    }
}

fn animate_popup(
    mut commands: Commands,
    mut query: Query<(Entity, &mut TextureAtlas, &mut AnimationConfig), With<Popup>>,
    time: Res<Time>,
) {
    for (entity, mut atlas, mut config) in query.iter_mut() {
        config.frame_timer.tick(time.delta());

        if config.frame_timer.just_finished() {
            if atlas.index == config.last_sprite_index {
                commands.entity(entity).despawn();

            }
            else {
                atlas.index += 1;
                config.frame_timer = AnimationConfig::timer_from_fps(config.fps);
            }
        }
    }
}