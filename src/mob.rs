use std::{f32::consts::PI, time::Duration};

use avian2d::prelude::*;
use bevy::prelude::*;
use rand::Rng;

use crate::{
    exp_orb::SpawnExpOrbEvent,
    experience::PlayerExperience,
    gamemap::{LevelGenerator, TileType, ROOM_SIZE},
    health::{Health, PlayerHPChanged},
    invincibility::Invincibility,
    level_completion::PortalEvent,
    pathfinding::Pathfinder,
    player::{Player, PlayerDeathEvent},
    projectile::Projectile,
    GameLayer,
    GameState,
};

pub struct MobPlugin;

impl Plugin for MobPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<MobDeathEvent>()
            .insert_resource(PortalPosition::default())
            .add_systems(OnEnter(GameState::InGame), debug_spawn_mobs)
            .add_systems(
                FixedUpdate,
                (move_mobs, hit_projectiles, hit_player, mob_death).run_if(in_state(GameState::InGame)),
            );
    }
}

#[derive(Component)]
pub struct Mob {
    damage: i32,
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

#[derive(Event)]
pub struct MobDeathEvent {
    pub entity: Entity,
    pub orbs: u32,
    pub pos: Vec3,
    pub dir: Vec3,
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
                if rng.gen::<f32>() > 0.9 && (i > 18 || i < 14) && (j > 18 || j < 14) { // make sure emenies don't spawn too close to player (todo: rewrite)
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
                        .insert(Mob { 
                            damage: 20
                         })
                        .insert(Pathfinder {
                            path: vec![],
                            update_path_timer: Timer::new(Duration::from_millis(rand::thread_rng().gen_range(500..900)),
                                                    TimerMode::Repeating),
                            speed: 2500.,
                        })
                        .insert(MobLoot { orbs: 3 })
                        .insert(Health {
                            max: 100,
                            current: 100,
                        });
                }
            }
        }
    }
}

fn move_mobs(mut mob_query: Query<(&mut LinearVelocity, &Transform, &mut Pathfinder)>, time: Res<Time>) {
    for (mut linvel, transform, mut pathfinder) in mob_query.iter_mut() {
        if pathfinder.path.len() > 0 {
            //let mob_tile_pos = Vec2::new(((transform.translation.x - (ROOM_SIZE / 2) as f32) / ROOM_SIZE as f32).floor(), (transform.translation.y - (ROOM_SIZE / 2) as f32) / ROOM_SIZE as f32).floor();
            let direction = Vec2::new(
                pathfinder.path[0].0 as f32 * 32. - transform.translation.x,
                pathfinder.path[0].1 as f32 * 32. - transform.translation.y,
            )
            .normalize();

            linvel.0 = direction * pathfinder.speed * time.delta_seconds();

            if transform.translation.truncate().distance(Vec2::new(pathfinder.path[0].0 as f32 * 32., pathfinder.path[0].1 as f32 * 32.)) <= 4. {
                pathfinder.path.remove(0);
            }
        }
    }
}

fn hit_projectiles(
    mut commands: Commands,

    projectile_query: Query<(Entity, &Projectile, &Transform)>,
    mut mob_query: Query<(Entity, &mut Health, &Transform, &MobLoot), With<Mob>>,

    mut ev_collision: EventReader<Collision>,
    mut ev_death: EventWriter<MobDeathEvent>,
) {
    for Collision(contacts) in ev_collision.read() {
        let proj_e: Option<Entity>;
        let mob_e: Option<Entity>;
        
        if projectile_query.contains(contacts.entity2)
            && mob_query.contains(contacts.entity1)
        {
            proj_e = Some(contacts.entity2);
            mob_e = Some(contacts.entity1);
        } 
        else if projectile_query.contains(contacts.entity1)
            && mob_query.contains(contacts.entity2) 
        {
            proj_e = Some(contacts.entity1);
            mob_e = Some(contacts.entity2);
        } 
        else 
        {
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

                        if health.current <= 0 {
                            health.current += 10000;
                            ev_death.send(MobDeathEvent {
                                entity: mob_e.unwrap(),
                                orbs: loot.orbs,
                                pos: transform.translation,
                                dir: shot_dir,
                            });
                        }
                    }
                }
            }
        }
    }
}

fn mob_death(
    mut portal_position: ResMut<PortalPosition>,
    player_experience: Res<PlayerExperience>,

    mob_query: Query<&Mob>,

    mut ev_spawn_portal: EventWriter<crate::level_completion::PortalEvent>,
    mut ev_spawn_orb: EventWriter<SpawnExpOrbEvent>,

    mut ev_mob_death: EventReader<MobDeathEvent>,
) {
    for ev in ev_mob_death.read() {
        portal_position.set_pos(ev.pos);

        if mob_query.is_empty() && !portal_position.check{
            portal_position.check = true;
            ev_spawn_portal.send( PortalEvent{pos: portal_position.position});
        }    
    
        let orb_count = (ev.orbs + player_experience.orb_bonus) as i32;
        let half_count = (orb_count as f32 / 2.).round() as i32;
    
        let offset = PI / 12.;
        for i in (-orb_count/2)..half_count {
            // считаем точки, куда будем выбрасывать частицы опыта
            let angle = ev.dir.y.atan2(ev.dir.x) + offset * i as f32;
            let direction = Vec2::from_angle(angle) * 32.0;
            let destination = Vec3::new(
                ev.pos.x + direction.x,
                ev.pos.y + direction.y,
                ev.pos.z,
            );
    
            ev_spawn_orb.send(SpawnExpOrbEvent {
                pos: ev.pos,
                destination,
            });
        }
    }
}

fn hit_player(
    mut commands: Commands,
    mut collision_event_reader: EventReader<Collision>,
    mob_query: Query<(Entity, &Mob), Without<Player>>,
    mut player_query: Query<(Entity, &mut Health, &Player), Without<Invincibility>>,
    mut ev_hp: EventWriter<PlayerHPChanged>,
    mut ev_death: EventWriter<PlayerDeathEvent>,
) {
    for Collision(contacts) in collision_event_reader.read() {

        let mut mob_e = Entity::PLACEHOLDER;

        if mob_query.contains(contacts.entity1)
        && player_query.contains(contacts.entity2)
        {
            mob_e = contacts.entity1;
        }
        else if mob_query.contains(contacts.entity2)
        && player_query.contains(contacts.entity1)
        {
            mob_e = contacts.entity2;
        }

        if let Ok((player_e, mut health, player)) = player_query.get_single_mut() {
            for (mob_cadidate_e, mob) in mob_query.iter() {
                if mob_cadidate_e == mob_e {
                    health.damage(mob.damage);
                    ev_hp.send(PlayerHPChanged);
                    commands.entity(player_e).insert(Invincibility::new(player.invincibility_time));
                    if health.current <= 0 {
                        ev_death.send(PlayerDeathEvent(player_e));
                    }
                }
            }
        }
    }
}