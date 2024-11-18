use avian2d::prelude::*;
use bevy::prelude::*;
use seldom_state::trigger::Done;

use crate::{
    camera::YSort, health::{Health, Hit}, mobs::*, pathfinding::Pathfinder, projectile::{Friendly, Projectile}, stun::Stun, GameLayer, GameState
};

pub struct ObstaclePlugin;

impl Plugin for ObstaclePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CorpseSpawnEvent>().add_systems(
            Update,
            (
                spawn_corpse,
                damage_obstacles::<Obstacle>,
                hit_obstacles::<Obstacle>,
                corpse_collision,
            )
                .run_if(in_state(GameState::InGame)),
        );
    }
}
//event to spawn corpse
//Corpse component for necromancer.
#[derive(Component)]
pub struct Corpse {
    mob_type: MobType,
}

#[derive(Event)]
pub struct CorpseSpawnEvent {
    pub pos: Vec3,
    pub mob_type: MobType,
}

//struct for obstacles, which can be destroyed(post, corpses, smth)
#[derive(Component)]
pub struct Obstacle;

fn corpse_collision(
    mut commands: Commands,
    mut summoner_query: Query<
        (Entity, &Transform, &mut Summoning, &Pathfinder),
        (Without<RaisingFlag>, Without<Stun>),
    >,
    mut corpse_query: Query<(Entity, &Transform, &Corpse), Without<BusyRaising>>,
    mut ev_collision: EventReader<Collision>,
) {
    for Collision(contacts) in ev_collision.read() {
        let mut spawner_e = Entity::PLACEHOLDER;
        let mut corpse_e = Entity::PLACEHOLDER;

        if summoner_query.contains(contacts.entity2) && corpse_query.contains(contacts.entity1) {
            spawner_e = contacts.entity2;
            corpse_e = contacts.entity1;
        } else if summoner_query.contains(contacts.entity1)
            && corpse_query.contains(contacts.entity2)
        {
            spawner_e = contacts.entity1;
            corpse_e = contacts.entity2;
        }
        for (candidate_e, _transform, _summoning, _pathfinder) in summoner_query.iter_mut() {
            if spawner_e == candidate_e {
                for (corpse_candidate_e, transform, corpse) in corpse_query.iter_mut() {
                    if corpse_e == corpse_candidate_e {
                        commands
                            .entity(spawner_e)
                            .insert(Raising {
                                mob_type: corpse.mob_type.clone(),
                                mob_pos: *transform,
                                corpse_id: corpse_e,
                            })
                            .insert(Done::Success);
                        commands.entity(corpse_e).insert(BusyRaising);
                    }
                }
            }
        }
    }
}

fn hit_obstacles<T: Component>(
    //TODO: ADD LOOT DROP FROM OBSTACLES IDK, MAYBE ADD LOOT TO THEM
    mut commands: Commands,
    projectile_query: Query<(Entity, &Projectile, &Transform), With<Friendly>>,
    mut obstacle_query: Query<(&CollidingEntities, &mut Health, &Transform), With<T>>,
) {
    for (colliding_e, mut health, obstacle_transform) in obstacle_query.iter_mut() {
        for (proj_e, projectile, projectile_transform) in projectile_query.iter() {
            if colliding_e.contains(&proj_e) {
                // урон
                let damage = projectile.damage as i32;

                // направление выстрела
                let shot_dir =
                    (obstacle_transform.translation - projectile_transform.translation).normalize();

                // пушим в очередь попадание
                health.hit_queue.push(Hit {
                    damage,
                    element: Some(projectile.element),
                    direction: shot_dir,
                });

                // деспавним снаряд
                commands.entity(proj_e).despawn();
            }
        }
    }
}

fn damage_obstacles<T: Component>(
    mut commands: Commands,
    mut obstacle_query: Query<(Entity, &mut Health), With<T>>,
) {
    for (entity, mut health) in obstacle_query.iter_mut() {
        if !health.hit_queue.is_empty() {
            let hit = health.hit_queue.remove(0);

            // наносим урон
            health.damage(hit.damage);

            if health.current <= 0 {
                // деспавним сразу
                commands.entity(entity).despawn_recursive();
                //TODO: ADD LOOT SPAWN
            }
        }
    }
}

//обработка спавна трупа для некромансера
fn spawn_corpse(
    mut ev_corpse_spawn: EventReader<CorpseSpawnEvent>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    for ev in ev_corpse_spawn.read() {
        let texture_path: &str;
        let can_be_spawned: bool;
        match ev.mob_type {
            MobType::Knight => {
                texture_path = "textures/mobs/corpses/knight_corpse.png";
                can_be_spawned = true;
            }
            MobType::Mossling => {
                texture_path = "textures/mobs/corpses/mossling_corpse.png";
                can_be_spawned = true;
            }
            MobType::FireMage => {
                texture_path = "textures/mobs/corpses/fire_mage_corpse.png";
                can_be_spawned = true;
            }
            MobType::WaterMage => {
                texture_path = "textures/mobs/corpses/water_mage_corpse.png";
                can_be_spawned = true;
            }
            MobType::JungleTurret => {
                texture_path = "textures/mobs/corpses/plant_corpse.png";
                can_be_spawned = true;
            }
            MobType::Necromancer => {
                texture_path = "textures/mobs/corpses/necromancer_corpse.png";
                can_be_spawned = false;
            }
            MobType::Koldun => {
                texture_path = "textures/mob_corpse_placeholder.png";
                can_be_spawned = false; //maybe true?
            }
            MobType::EarthElemental => {
                texture_path = "textures/mobs/corpses/earth_elemental_corpse.png";
                can_be_spawned = true;
            }
            MobType::FireElemental => {
                texture_path = "textures/mobs/corpses/fire_elemental_corpse.png";
                can_be_spawned = true;
            }
            MobType::WaterElemental => {
                texture_path = "textures/mob_corpse_placeholder.png";
                can_be_spawned = true;
            }
            MobType::AirElemental => {
                texture_path = "textures/mobs/corpses/air_elemental_corpse.png";
                can_be_spawned = true;
            }
            MobType::ClayGolem => {
                texture_path = "textures/mob_corpse_placeholder.png";
                can_be_spawned = true;
            }
            MobType::SkeletMage => {
                texture_path = "textures/mob_corpse_placeholder.png";
                can_be_spawned = true;
            }
            MobType::SkeletWarrior => {
                texture_path = "textures/mob_corpse_placeholder.png";
                can_be_spawned = true;
            }
            MobType::SkeletRanger => {
                texture_path = "textures/mob_corpse_placeholder.png";
                can_be_spawned = true;
            }
            MobType::TankEater => {
                texture_path = "textures/mob_corpse_placeholder.png";
                can_be_spawned = true;
            }
        }
        let texture = asset_server.load(texture_path);
        let grave = commands
            .spawn(SpriteBundle {
                texture,
                transform: Transform::from_xyz(ev.pos.x, ev.pos.y, ev.pos.z),
                ..default()
            })
            .insert(Collider::circle(6.))
            .insert(Sensor)
            .insert(LockedAxes::ROTATION_LOCKED)
            .insert(CollisionLayers::new(GameLayer::Enemy, [GameLayer::Enemy]))
            .insert(Health::new(40))
            .insert(Obstacle)
            .insert(YSort(8.0))
            .id();

        if can_be_spawned {
            commands.entity(grave).insert(Corpse {
                mob_type: ev.mob_type.clone(),
            });
        }
    }
}
