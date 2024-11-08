//all things about mobs and their spawn/behaviour
///add mobs with kinematic body type
#[allow(unused)]
use crate::{
    animation::AnimationConfig,
    elements::{ElementResistance, ElementType},
    exp_orb::SpawnExpOrbEvent,
    experience::PlayerExperience,
    gamemap::Map,
    health::{Health, Hit},
    level_completion::{PortalEvent, PortalManager},
    mobs::{
        MeleeMobBundle, SearchAndPursue, Idle,
        FlipEntity, Mob, MobDeathEvent, MobLoot, MobType, PhysicalBundle, RotationEntity,
        STATIC_MOBS,
    },
    obstacles::{Corpse, CorpseSpawnEvent},
    player::Player,
    projectile::{Friendly, Projectile, SpawnProjectileEvent},
    stun::Stun,
    GameLayer, GameState,
};

use avian2d::prelude::*;
use bevy::prelude::*;

pub struct FriendPlugin;

impl Plugin for FriendPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (friend_damage_mob, damage_friends).run_if(in_state(GameState::InGame)),
        );
    }
}

#[derive(Component, Default)]
pub struct Friend;
///maybe add contact damage or add some melee attacks?


///спавн именно особых энтити, не поднятие дохлых, дохлых поднимать можно через mob_spawn

fn friend_damage_mob(
    mut friend_query: Query<
        (&CollidingEntities, &mut Health, &Mob),
        (With<Friend>, Without<Player>),
    >,
    // если у нас моб берётся как референс, можно не писать With<Mob>, он и так будет с ним
    mut mob_query: Query<(Entity, &mut Health, &Mob), Without<Friend>>, 
) {
    for (friend_e, mut health_f, mob_f) in friend_query.iter_mut() {
        for (mob_e, mut health_m, mob_m) in mob_query.iter_mut() {
            if friend_e.contains(&mob_e) {
                health_f.hit_queue.push(Hit {
                    damage: mob_m.damage as i32,
                    element: Some(ElementType::Earth),
                    direction: Vec3::ZERO,
                });

                health_m.hit_queue.push(Hit {
                    damage: mob_f.damage as i32,
                    element: Some(ElementType::Earth),
                    direction: Vec3::ZERO,
                });
            }
        }
    }
}

pub fn damage_friends(
    mut commands: Commands,
    mut ev_corpse: EventWriter<CorpseSpawnEvent>,
    mut mob_query: Query<
        (Entity, &mut Health, &mut Mob, &Transform, &MobType),
        With<Friend>,
    >,
    mut mob_map: ResMut<Map>,
) {
    for (entity, mut health, _mob, transform, mob_type) in mob_query.iter_mut() {
        if !health.hit_queue.is_empty() {
            let hit = health.hit_queue.remove(0);

            // наносим урон
            health.damage(hit.damage);

            // кидаем стан
            commands.entity(entity).insert(Stun::new(0.5));
            // шлём ивент смерти
            if health.current <= 0 {
                // деспавним сразу
                commands.entity(entity).despawn_recursive();
                /*
                                // события "поcле смерти"
                                ev_death.send(MobDeathEvent {
                                    orbs: loot.orbs,
                                    pos: transform.translation,
                                    dir: hit.direction,
                                });
                */
                // спавним труп на месте смерти моба
                ev_corpse.send(CorpseSpawnEvent {
                    mob_type: mob_type.clone(),
                    pos: transform.translation.with_z(0.05),
                });

                for i in STATIC_MOBS {
                    if mob_type == i {
                        let mob_pos = (
                            (transform.translation.x.floor() / 32.).floor() as u16,
                            (transform.translation.y.floor() / 32.).floor() as u16,
                        );

                        mob_map
                            .map
                            .get_mut(&(mob_pos.0, mob_pos.1))
                            .unwrap()
                            .mob_count -= 1;

                        break;
                    }
                }
            }
        }
    }
}
