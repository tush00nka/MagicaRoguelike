use std::{f32::consts::PI, time::Duration};

use avian2d::prelude::*;
use bevy::prelude::*;
use rand::Rng;

use crate::{
    exp_orb::{ExpOrb, ExpOrbDrop},
    gamemap::{LevelGenerator, TileType, ROOM_SIZE},
    health::{DeathEvent, Health},
    level_completion::PortalEvent,
    projectile::Projectile,
    GameLayer, GameState,
};

pub struct MobPlugin;

impl Plugin for MobPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PortalPosition::default())
            .add_systems(OnEnter(GameState::InGame), debug_spawn_mobs)
            .add_systems(
                FixedUpdate,
                (move_mobs, hit_projectiles).run_if(in_state(GameState::InGame)),
            );
    }
}

#[derive(Component)]
pub struct Mob {
    pub path: Vec<(u16, u16)>, 
    pub update_path_timer: Timer,
    speed: f32,
}

#[derive(Resource)]
pub struct PortalPosition {
    position: Vec3,
    pub check: bool //maybe change to i32, if there would be some bugs with despawn, portal may not spawn, i suppose?
}
impl Default for PortalPosition{
    fn default() -> PortalPosition{
        PortalPosition{position: Vec3{x:0.,y:0.,z:0.},check: false}
    }
}
impl PortalPosition {
    fn set_pos(&mut self, pos:Vec3){
        self.position = pos;
    }
}

#[derive(Component)]
pub struct MobLoot {
    pub orbs: u32,
}

fn debug_spawn_mobs(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    room: Res<LevelGenerator>,
) {
    let grid = room.grid.clone();
    for i in 1..grid.len() - 1 {
        for j in 1..grid[i].len() - 1 {
            if grid[i][j] == TileType::Floor {
                let mut rng = rand::thread_rng();
                if rng.gen::<f32>() > 0.9 {
                    let mob = commands
                        .spawn(SpriteBundle {
                            texture: asset_server.load("textures/mob_mossling.png"),
                            transform: Transform::from_xyz(
                                (i as i32 * ROOM_SIZE) as f32,
                                (j as i32 * ROOM_SIZE) as f32,
                                1.0,
                            ),
                            ..default()
                        })
                        .id();

                    commands
                        .entity(mob)
                        .insert(RigidBody::Dynamic)
                        .insert(GravityScale(0.0))
                        .insert(LockedAxes::ROTATION_LOCKED)
                        .insert(Collider::circle(6.0))
                        .insert(CollisionLayers::new(
                            GameLayer::Enemy,
                            [
                                GameLayer::Wall,
                                GameLayer::Projectile,
                                GameLayer::Shield,
                                GameLayer::Enemy,
                                GameLayer::Player,
                            ],
                        ))
                        .insert(LinearVelocity::ZERO)
                        .insert(Mob { path: vec![], update_path_timer: Timer::new(Duration::from_millis(rand::thread_rng().gen_range(500..900)), TimerMode::Repeating), speed: 2500. })
                        .insert(MobLoot { orbs: 2 })
                        .insert(Health {
                            max: 100,
                            current: 100,
                        });
                }
            }
        }
    }
}

fn move_mobs(mut mob_query: Query<(&mut LinearVelocity, &Transform, &mut Mob)>, time: Res<Time>) {
    for (mut linvel, transform, mut mob) in mob_query.iter_mut() {
        if mob.path.len() > 0 {
            //let mob_tile_pos = Vec2::new(((transform.translation.x - (ROOM_SIZE / 2) as f32) / ROOM_SIZE as f32).floor(), (transform.translation.y - (ROOM_SIZE / 2) as f32) / ROOM_SIZE as f32).floor();
            let direction = Vec2::new(
                mob.path[0].0 as f32 * 32. - transform.translation.x,
                mob.path[0].1 as f32 * 32. - transform.translation.y,
            )
            .normalize();

            linvel.0 = direction * mob.speed * time.delta_seconds();

            if transform.translation.truncate().distance(Vec2::new(mob.path[0].0 as f32 * 32., mob.path[0].1 as f32 * 32.)) <= 4. {
                mob.path.remove(0);
            }
        }
    }
}

fn hit_projectiles(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut collision_event_reader: EventReader<Collision>,
    projectile_query: Query<(Entity, &Projectile, &Transform)>,
    mut mob_query: Query<(Entity, &mut Health, &Transform, &MobLoot), With<Mob>>,
    mut ev_death: EventWriter<DeathEvent>,
    mut amount_mobs: ResMut<PortalPosition>,
    mut ev_spawn_portal: EventWriter<crate::level_completion::PortalEvent>,
) {
    if mob_query.is_empty() && !amount_mobs.check{
        amount_mobs.check = true;
        ev_spawn_portal.send( PortalEvent{pos:amount_mobs.position});
    }
    for Collision(contacts) in collision_event_reader.read() {
        let proj_e: Option<Entity>;
        let mob_e: Option<Entity>;
        
        if projectile_query.contains(contacts.entity2) && mob_query.contains(contacts.entity1) {
            proj_e = Some(contacts.entity2);
            mob_e = Some(contacts.entity1);
        } else if projectile_query.contains(contacts.entity1)
            && mob_query.contains(contacts.entity2)
        {
            proj_e = Some(contacts.entity1);
            mob_e = Some(contacts.entity2);
        } else {
            proj_e = None;
            mob_e = None;
        }

        for (candidate_e, mut health, transform, loot) in mob_query.iter_mut() {
            if mob_e.is_some() && mob_e.unwrap() == candidate_e {
                for (proj_candidate_e, projectile, projectile_transform) in projectile_query.iter()
                {
                    if proj_e.is_some() && proj_e.unwrap() == proj_candidate_e {
                        health.damage(projectile.damage.try_into().unwrap());
                        commands.entity(proj_e.unwrap()).despawn();

                        let shot_dir =
                            (transform.translation - projectile_transform.translation).normalize();
                        commands.entity(mob_e.unwrap()).insert(
                            ExternalImpulse::new(shot_dir.truncate() * 50_000.0)
                                .with_persistence(false),
                        );

                        if health.current <= 0 {
                            amount_mobs.set_pos(transform.translation);
                            ev_death.send(DeathEvent(mob_e.unwrap()));

                            let offset = PI / 12.;
                            for i in -1..(loot.orbs as i32 - 1) {
                                // считаем точки, куда будем выбрасывать частицы опыта
                                let angle = shot_dir.y.atan2(shot_dir.x) + offset * i as f32;
                                let direction = Vec2::from_angle(angle) * 32.0;
                                let destination = Vec3::new(
                                    transform.translation.x + direction.x,
                                    transform.translation.y + direction.y,
                                    transform.translation.z,
                                );

                                commands
                                    .spawn(SpriteBundle {
                                        texture: asset_server.load("textures/exp_particle.png"),
                                        transform: Transform::from_translation(
                                            transform.translation,
                                        ),
                                        ..default()
                                    })
                                    .insert(ExpOrb { exp: 5 })
                                    .insert(ExpOrbDrop {
                                        drop_destination: destination,
                                    });
                            }
                        }
                    }
                }
            }
        }
    }
}