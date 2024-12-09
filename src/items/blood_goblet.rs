//Кубок с кровью - дает регенерацию здоровья, но теперь заклинания тратят здоровье (зависит также от кол-ва копий)
use bevy::prelude::*;

use crate::{
    health::Health,
    item::{
        ItemPickedUpEvent,
        ItemType
    }, 
    player::{
        Player,
        PlayerStats
    }
};

pub struct BloodGobletPlugin;

impl Plugin for BloodGobletPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (regen_health, apply_effect));
    }
}

#[derive(Component)]
pub struct RegenHealth {
    timer: Timer,
}

fn apply_effect(
    mut commands: Commands,
    mut ev_item_picked_up: EventReader<ItemPickedUpEvent>,
    mut player_stats: ResMut<PlayerStats>,
    player_query: Query<Entity, With<Player>>, 
) {
    for ev in ev_item_picked_up.read() {
        if ev.item_type == ItemType::BloodGoblet {
            player_stats.health_regen += 1;
            player_stats.spell_cast_hp_fee += 5;

            if let Ok(entity) = player_query.get_single() {
                commands.entity(entity).insert(RegenHealth {
                    timer: Timer::from_seconds(0.5, TimerMode::Repeating),
                });
            }
        }   
    }
}

fn regen_health(
    player_stats: Res<PlayerStats>,
    mut player_query: Query<(&mut Health, &mut RegenHealth), With<Player>>,
    time: Res<Time>,
) {
    let Ok((mut player_health, mut regen)) = player_query.get_single_mut() else {
        return;
    };

    regen.timer.tick(time.delta());

    if regen.timer.just_finished() {
        player_health.heal(player_stats.health_regen);
    }
}