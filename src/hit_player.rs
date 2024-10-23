//all systems that can damage player should be there
use crate::{
    elements::ElementResistance, health::{Health, Hit}, invincibility::Invincibility, mob::Mob, player::{Player, PlayerDeathEvent}, projectile::{Hostile, Projectile}, GameState, TimeState
};
use avian2d::prelude::*;
use bevy::prelude::*;
pub struct HitPlayerPlugin;

impl Plugin for HitPlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, (hit_player, proj_hit_player, damage_player)
                .run_if(in_state(GameState::InGame))
                .run_if(in_state(TimeState::Unpaused)));
    }
}
//damage by collision with mob
fn hit_player(  //todo: change that we could add resistance mechanic
    mut collision_event_reader: EventReader<Collision>,
    mob_query: Query<(Entity, &Mob), Without<Player>>,
    mut player_query: Query<(Entity, &mut Health, &Player), Without<Invincibility>>,
) {
    for Collision(contacts) in collision_event_reader.read() {
        let mut mob_e = Entity::PLACEHOLDER;

        if mob_query.contains(contacts.entity1) && player_query.contains(contacts.entity2) {
            mob_e = contacts.entity1;
        } else if mob_query.contains(contacts.entity2) && player_query.contains(contacts.entity1) {
            mob_e = contacts.entity2;
        }

        if let Ok((_player_e, mut health, _player)) = player_query.get_single_mut() {
            for (mob_cadidate_e, mob) in mob_query.iter() {
                if mob_cadidate_e == mob_e {
                    health.hit_queue.push(Hit {
                        damage: mob.damage,
                        element: None,
                        direction: Vec3::ZERO,
                    });
                }
            }
        }
    }
}

//damage by projectiles
fn proj_hit_player( //todo: change that we could add resistance mechanic
    mut commands: Commands,
    mut collision_event_reader: EventReader<Collision>,
    projectile_query: Query<(Entity, &Projectile), With<Hostile>>,
    mut player_query: Query<(Entity, &mut Health, &Player), Without<Invincibility>>,
) {
    for Collision(contacts) in collision_event_reader.read() {
        let mut projectile_e = Entity::PLACEHOLDER;

        if projectile_query.contains(contacts.entity1) && player_query.contains(contacts.entity2) {
            projectile_e = contacts.entity1;
        } else if projectile_query.contains(contacts.entity2)
            && player_query.contains(contacts.entity1)
        {
            projectile_e = contacts.entity2;
        }

        if let Ok((_player_e, mut health, _player)) = player_query.get_single_mut() {
            for (proj_cand_e, proj) in projectile_query.iter() {
                if proj_cand_e == projectile_e {
                    health.hit_queue.push( Hit {
                        damage: proj.damage as i32,
                        element: Some(proj.element),
                        direction: Vec3::ZERO,
                    });
                    commands.get_entity(proj_cand_e).unwrap().despawn();
                }
            }
        }
    }
}

fn damage_player(
    mut commands: Commands,
    mut ev_death: EventWriter<PlayerDeathEvent>,
    mut player_query: Query<(Entity, &mut Health, &Player, &ElementResistance), With<Player>>,
) {
    for (player_e, mut health, player, resistance) in player_query.iter_mut() {
        if !health.hit_queue.is_empty() {
            let hit = health.hit_queue.remove(0);

            // считаем сопротивление
            let mut damage = hit.damage;
            resistance.calculate_for(&mut damage, hit.element);

            // наносим урон
            health.damage(damage);

            //i-frames
            commands
            .entity(player_e)
            .insert(Invincibility::new(player.invincibility_time));


            // шлём ивент смерти
            if health.current <= 0 {
                // события "поле смерти"
                ev_death.send(PlayerDeathEvent (player_e));
            }
        }
    }
}