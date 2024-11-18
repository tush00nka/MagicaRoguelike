use bevy::prelude::*;

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayAudioEvent>();
    
        app.add_systems(Update, play_sfx);
    }
}

#[derive(Event)]
pub struct PlayAudioEvent {
    filename: String
}

impl PlayAudioEvent {
    pub fn from_file(filename: &str) -> Self {
        Self {
            filename: filename.to_string()
        }
    }
}

fn play_sfx(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut ev_play_audio: EventReader<PlayAudioEvent>,
) {
    for ev in ev_play_audio.read() {
        commands.spawn(AudioBundle {
            source: asset_server.load(format!("audio/{}", ev.filename)),
            settings: PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Despawn,
                ..default()
            }
        });
    }
}