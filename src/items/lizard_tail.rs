use bevy::prelude::*;

use crate::{
    health::Health,
    item::{
        ItemPickedUpEvent,
        ItemType
    },
    player::Player
};

pub struct LizardTailPlugin;

impl Plugin for LizardTailPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<DeathAvoidPopupEvent>()
            .add_systems(Update, (apply_effect, spawn_death_popup, popup_follow_player, popup_scale, popup_despawn));
    }
}

fn apply_effect(
    mut ev_item_picked_up: EventReader<ItemPickedUpEvent>,
    mut player_query: Query<&mut Health, With<Player>>,
) {
    if let Ok(mut health) = player_query.get_single_mut() {
        for ev in ev_item_picked_up.read() {
            if ev.item_type == ItemType::LizardTail {
                println!("Lizard Tail effect applied");
                health.extra_lives += 1;
            }   
        }
    } 
}

#[derive(Component)]
pub struct DeathAvoidPopup {
    timer: Timer,
}

#[derive(Event)]
pub struct DeathAvoidPopupEvent;

fn spawn_death_popup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut ev_player_death: EventReader<DeathAvoidPopupEvent>,
    player_query: Query<&Transform, With<Player>>,
) {
    for _ev in ev_player_death.read() {
        if let Ok(player_transform) = player_query.get_single() {
            commands.spawn(SpriteBundle {
                texture: asset_server.load("textures/items/lizard_tail.png"),
                transform: Transform {
                    translation: player_transform.translation,
                    scale: Vec3::splat(0.1),
                    ..default()
                },
                ..default()
            })
            .insert(DeathAvoidPopup { timer: Timer::from_seconds(3., TimerMode::Once) });
        }
    }
}   

fn popup_follow_player(
    mut popup_query: Query<&mut Transform, (With<DeathAvoidPopup>, Without<Player>)>,
    player_query: Query<&Transform, (With<Player>, Without<DeathAvoidPopup>)>,
) { 
    for mut popup_transform in popup_query.iter_mut() {
        if let Ok(player_transform) = player_query.get_single() {
            popup_transform.translation = player_transform.translation.with_y(player_transform.translation.y + 20.);
        }
    }
}

fn popup_scale(
    mut popup_query: Query<(&mut Transform, &mut Sprite), With<DeathAvoidPopup>>,
    time: Res<Time>,
) {
    for (mut popup_transform, mut sprite) in popup_query.iter_mut() {
        if popup_transform.scale != Vec3::ONE {
            popup_transform.scale = popup_transform.scale.lerp(Vec3::ONE, 3. * time.delta_seconds());

            let current_alpha = sprite.color.alpha();
            sprite.color.set_alpha(current_alpha.lerp(0.0, 2. * time.delta_seconds()));
        }
    }
}

fn popup_despawn(
    mut commands: Commands,
    mut popup_query: Query<(Entity, &mut DeathAvoidPopup)>,
    time: Res<Time>,
) {
    for (entity, mut popup) in popup_query.iter_mut() {
        popup.timer.tick(time.delta());

        if popup.timer.just_finished() {
            commands.entity(entity).despawn();
        }
    } 
}