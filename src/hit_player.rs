//all systems that can damage player should be there

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    elements::ElementResistance,
    health::{Health, Hit},
    invincibility::Invincibility,
    mobs::Mob,
    player::{Player, PlayerDeathEvent},
    projectile::{Hostile, Projectile},
    GameState,
};

pub struct HitPlayerPlugin;

impl Plugin for HitPlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (hit_player, proj_hit_player, damage_player)
                .run_if(in_state(GameState::InGame)));
    }
}
//damage by collision with mob
fn hit_player(
    mob_query: Query<(Entity, &Mob), Without<Player>>,
    mut player_query: Query<(&CollidingEntities, &mut Health), (With<Player>, Without<Invincibility>)>,
) {
    let Ok((colliding_e, mut health)) = player_query.get_single_mut() else {
        return;
    };

    for (mob_e, mob) in mob_query.iter() {
        if colliding_e.contains(&mob_e) {
            health.hit_queue.push( Hit {
                damage: mob.damage,
                element: None,
                direction: Vec3::ZERO,
            });
        }
    }

}
//damage by projectiles
fn proj_hit_player(
    //todo: change that we could add resistance mechanic
    mut commands: Commands,
    projectile_query: Query<(Entity, &Projectile), With<Hostile>>,
    mut player_query: Query<(&CollidingEntities, &mut Health), (With<Player>, Without<Invincibility>)>,
) {
    let Ok((colliding_e, mut health)) = player_query.get_single_mut() else {
        return;
    };

    for (proj_e, projectile) in projectile_query.iter() {
        if colliding_e.contains(&proj_e) {
            health.hit_queue.push( Hit {
                damage: projectile.damage as i32,
                element: Some(projectile.element),
                direction: Vec3::ZERO,
            });

            commands.entity(proj_e).despawn();
        }
    }

}

fn damage_player(
    mut commands: Commands,
    mut ev_death: EventWriter<PlayerDeathEvent>,
    mut player_query: Query<(Entity, &mut Health, &Player, &ElementResistance), (With<Player>, Without<Invincibility>)>,
) {
    let Ok((player_e, mut health, player, resistance)) = player_query.get_single_mut() else {
        return;
    };

    if !health.hit_queue.is_empty() {
        //i-frames
        commands
        .entity(player_e)
        .insert(Invincibility::new(player.invincibility_time));

        let hit = health.hit_queue.remove(0);
        health.hit_queue.clear();

        // считаем сопротивление
        let mut damage = hit.damage;
        resistance.calculate_for(&mut damage, hit.element);

        // наносим урон
        health.damage(damage);

        // шлём ивент смерти
        if health.current <= 0 {
            // события "поле смерти"
            ev_death.send(PlayerDeathEvent (player_e));
        }
    }
}
