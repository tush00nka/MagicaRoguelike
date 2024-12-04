use crate::{blank_spell::SpawnBlankEvent, mobs::{OnDeathEffect, OnDeathEffectEvent, OnHitEffect, OnHitEffectEvent, PickupItemQueue, ProjectileType}};
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
        FlipEntity, Idle, MeleeMobBundle, Mob, MobDeathEvent, MobLoot, MobType, PhysicalBundle,
        RotationEntity, SearchAndPursue, STATIC_MOBS,
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
    mut mob_query: Query<(Entity, &mut Health, &mut Mob, &Transform, &MobType), With<Friend>>,
    mut mob_map: ResMut<Map>,

    mut blank_spawn_ev: EventWriter<SpawnBlankEvent>,

    on_hit_query: Query<&OnHitEffect>,
    on_death_effect: Query<&OnDeathEffect>,

    mut on_hit_event: EventWriter<OnHitEffectEvent>,
    mut on_death_event: EventWriter<OnDeathEffectEvent>,

    mut thief_query: Query<&mut PickupItemQueue>,
) {
    for (entity, mut health, _mob, transform, mob_type) in mob_query.iter_mut() {
        if !health.hit_queue.is_empty() {
            let hit = health.hit_queue.remove(0);

            // наносим урон
            health.damage(hit.damage);

            if on_hit_query.contains(entity) {
                let mut vec_objects = vec![];
                let on_hit_eff;

                match on_hit_query.get(entity).unwrap() {
                    OnHitEffect::DropItemFromBag => {
                        on_hit_eff = OnHitEffect::DropItemFromBag;
                        let mut temp_bag = thief_query.get_mut(entity).unwrap();

                        for i in temp_bag.item_queue.clone() {
                            match i {
                                None => break,
                                Some(item) => match item.item_name {
                                    Some(name) => {
                                        vec_objects.push(item.item_type as i32);
                                        vec_objects.push(name as i32);
                                    }
                                    None => {
                                        vec_objects.push(item.item_type as i32);
                                    }
                                },
                            }
                        }

                        temp_bag.empty_queue()
                    }
                }
                on_hit_event.send(OnHitEffectEvent {
                    pos: transform.translation,
                    dir: hit.direction,
                    vec_of_objects: vec_objects,
                    on_hit_effect_type: on_hit_eff,
                    is_friendly: false,
                });
            }

            // кидаем стан
            commands.entity(entity).insert(Stun::new(0.5));
            // шлём ивент смерти
            if health.current <= 0 {
            
                if on_death_effect.contains(entity) {
                    let vec_objects ;
                    let on_death_eff;

                    match on_death_effect.get(entity).unwrap() {
                        OnDeathEffect::CircleAttack => {
                            on_death_eff = OnDeathEffect::CircleAttack;
                            vec_objects = vec![ProjectileType::Gatling as i32; 16];
                        }
                    }
                    on_death_event.send(OnDeathEffectEvent {
                        pos: transform.translation,
                        dir: hit.direction,
                        vec_of_objects: vec_objects,
                        on_death_effect_type: on_death_eff,
                        is_friendly: false,
                    });
                }
            
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

                if *mob_type == MobType::AirElemental {
                    blank_spawn_ev.send(SpawnBlankEvent {
                        range: 8.,
                        position: transform.translation,
                        speed: 10.,
                        is_friendly: true,
                    });
                }

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
