//all systems that can damage player should be there

use avian2d::prelude::*;
use bevy::prelude::*;
use rand::Rng;

use crate::{
    camera::CameraShakeEvent, friend::Friend, elements::ElementResistance, health::{Health, Hit}, invincibility::Invincibility, mobs::Mob, player::{Player, PlayerDeathEvent, PlayerStats}, projectile::{Friendly, Hostile, Projectile}, GameState
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
    mob_query: Query<(Entity, &Mob), (Without<Friend>, Without<Player>)>,
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

            return;
        }
    }

}
//damage by projectiles
fn proj_hit_player(
    //todo: change that we could add resistance mechanic
    mut commands: Commands,
    mut projectile_query: Query<(Entity, &mut Projectile), With<Hostile>>,
    mut player_query: Query<(&CollidingEntities, &mut Health), (With<Player>, Without<Invincibility>)>,
    player_stats: Res<PlayerStats>,
) {
    let Ok((colliding_e, mut health)) = player_query.get_single_mut() else {
        return;
    };

    for (proj_e, mut projectile) in projectile_query.iter_mut() {
        if colliding_e.contains(&proj_e) {
            let deflect_check: f32 = rand::thread_rng().gen_range(0.0..1.0);

            if deflect_check <= player_stats.projectile_deflect_chance {
                projectile.direction = -projectile.direction;
                commands.entity(proj_e).remove::<Hostile>();
                commands.entity(proj_e).insert(Friendly);
                return;
            }

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
    mut player_query: Query<(Entity, &mut Health, &ElementResistance), With<Player>>,
    player_stats: Res<PlayerStats>,

    mut ev_shake_camera: EventWriter<CameraShakeEvent>,
) {
    let Ok((player_e, mut health, resistance)) = player_query.get_single_mut() else {
        return;
    };

    if !health.hit_queue.is_empty() {
        let hit = health.hit_queue.remove(0);
        health.hit_queue.clear();

        //i-frames
        commands
        .entity(player_e)
        .insert(Invincibility::new(player_stats.invincibility_time));

        ev_shake_camera.send(CameraShakeEvent);

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
