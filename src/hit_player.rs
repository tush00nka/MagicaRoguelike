//all systems that can damage player should be there
use crate::{
    health::Health,
    invincibility::Invincibility,
    mob::Mob,
    player::{Player, PlayerDeathEvent},
    projectile::{Hostile, Projectile},
    GameState,
};
use avian2d::prelude::*;
use bevy::prelude::*;
pub struct HitPlayerPlugin;

impl Plugin for HitPlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (hit_player, proj_hit_player).run_if(in_state(GameState::InGame)),
        );
    }
}
//damage by collision with mob
fn hit_player(
    mut commands: Commands,
    mut collision_event_reader: EventReader<Collision>,
    mob_query: Query<(Entity, &Mob), Without<Player>>,
    mut player_query: Query<(Entity, &mut Health, &Player), Without<Invincibility>>,
    mut ev_death: EventWriter<PlayerDeathEvent>,
) {
    for Collision(contacts) in collision_event_reader.read() {
        let mut mob_e = Entity::PLACEHOLDER;

        if mob_query.contains(contacts.entity1) && player_query.contains(contacts.entity2) {
            mob_e = contacts.entity1;
        } else if mob_query.contains(contacts.entity2) && player_query.contains(contacts.entity1) {
            mob_e = contacts.entity2;
        }

        if let Ok((player_e, mut health, player)) = player_query.get_single_mut() {
            for (mob_cadidate_e, mob) in mob_query.iter() {
                if mob_cadidate_e == mob_e {
                    health.damage(mob.damage);
                    commands
                        .entity(player_e)
                        .insert(Invincibility::new(player.invincibility_time));
                        //inv frames
                    if health.current <= 0 {
                        ev_death.send(PlayerDeathEvent(player_e));
                    }
                }
            }
        }
    }
}

//damage by projectiles
fn proj_hit_player(
    mut commands: Commands,
    mut collision_event_reader: EventReader<Collision>,
    projectile_query: Query<(Entity, &Projectile), With<Hostile>>,
    mut player_query: Query<(Entity, &mut Health, &Player), Without<Invincibility>>,
    mut ev_death: EventWriter<PlayerDeathEvent>,
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

        if let Ok((player_e, mut health, player)) = player_query.get_single_mut() {
            for (proj_cand_e, proj) in projectile_query.iter() {
                if proj_cand_e == projectile_e {
                    health.damage(proj.damage as i32);
          
                    commands
                        .entity(player_e)
                        .insert(Invincibility::new(player.invincibility_time));
          
                    if health.current <= 0 {
                        ev_death.send(PlayerDeathEvent(player_e));
                    }
                    commands.get_entity(proj_cand_e).unwrap().despawn();
                }
            }
        }
    }
}
