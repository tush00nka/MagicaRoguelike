use bevy::prelude::*;

use crate::animation::AnimationConfig;

pub struct AlertPlugin;

impl Plugin for AlertPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnAlertEvent>();
        app.add_systems(Update, (spawn_alert, animate_alert));
    }
}

#[derive(Event)]
pub struct SpawnAlertEvent{
    pub position: Vec2,
}

#[derive(Component)]
struct AlertPopup;

fn spawn_alert(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut ev_spawn_alert: EventReader<SpawnAlertEvent>, 
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    for ev in ev_spawn_alert.read() {
        let texture = asset_server.load("textures/alert.png");

        let layout = TextureAtlasLayout::from_grid(UVec2::splat(16), 8, 1, None, None);
        let texture_atlas_layout = texture_atlas_layouts.add(layout);

        let animation_config = AnimationConfig::new(0, 7, 24);

        commands.spawn((
            SpriteBundle {
                texture: texture.clone(),
                transform: Transform::from_xyz(ev.position.x, ev.position.y, 5.),
                ..default()
            },
            TextureAtlas {
                layout: texture_atlas_layout.clone(),
                index: animation_config.first_sprite_index,
            },
            animation_config,
            AlertPopup,
        ));
    }
}

fn animate_alert(
    mut commands: Commands,
    mut alert_query: Query<(Entity, &mut AnimationConfig, &mut TextureAtlas), With<AlertPopup>>,
    time: Res<Time>,
) {
    for (alert_e, mut config, mut atlas) in alert_query.iter_mut() {
        config.frame_timer.tick(time.delta());

        if config.frame_timer.just_finished() {
            if atlas.index == config.last_sprite_index {
                commands.entity(alert_e).despawn();
            } else {
                atlas.index += 1;
                config.frame_timer = AnimationConfig::timer_from_fps(config.fps);
            }
        }
    }
}