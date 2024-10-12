//all things about mobs and their spawn/behaviour
use std::{f32::consts::PI, time::Duration};

use avian2d::prelude::*;
use bevy::prelude::*;
use dynamics::rigid_body;
use rand::Rng;

use crate::{
    exp_orb::SpawnExpOrbEvent,
    experience::PlayerExperience,
    gamemap::{LevelGenerator, MobMap, TileType, ROOM_SIZE},
    health::Health,
    level_completion::{PortalEvent, PortalManager},
    pathfinding::Pathfinder,
    player::Player,
    projectile::SpawnProjectileEvent,
    projectile::{Friendly, Projectile},
    stun::Stun,
    GameLayer, GameState,
};

pub struct MobPlugin;

impl Plugin for MobPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MobMap::default())
            .add_event::<MobDeathEvent>()
            .add_systems(OnEnter(GameState::InGame), spawn_mobs)
            .add_systems(
                FixedUpdate,
                (
                    move_mobs,
                    hit_projectiles,
                    mob_death,
                    teleport_mobs,
                    mob_shoot,
                )
                    .run_if(in_state(GameState::InGame)),
            );
    }
}
pub enum MobType {
    Mossling,
    Teleport,
}

#[derive(Bundle)]
pub struct PhysicBundle {
    collider: Collider,
    axes: LockedAxes,
    gravity: GravityScale,
    collision_layers: CollisionLayers,
    linear_velocity: LinearVelocity,
}

#[derive(Bundle)]
pub struct MobBundle {
    texture_path: String,
    mob_type: MobType,
    mob: Mob,
    loot: MobLoot,
    body_type: RigidBody,
    health: Health,
}

#[derive(Bundle)]
pub struct FireMageBundle {
    teleport_ability: Teleport,
    shoot_ability: ShootAbility,
}
impl Default for FireMageBundle {
    fn default(timer: Range<u64>) -> Self {//need to have 1 same
        Self {
            teleport_ability: Teleport {
                amount_of_tiles: 4,
                place_to_teleport: vec![],
                time_to_teleport: Timer::new(
                    Duration::from_millis(rand::thread_rng().gen_range(timer.clone())),
                    TimerMode::Repeating,
                ),
            },
            shoot_ability: ShootAbility { time_to_shoot: () },
        }
    }
}
impl MobBundle {
    fn mossling(i: usize, j: usize) -> Self {
        Self {
            texture_path: "textures/mob_mossling.png".to_string(),
            mob_type: MobType::Mossling,
            mob: Mob { damage: 20 },
            loot: MobLoot { orbs: 3 },
            body_type: RigidBody::Dynamic,
            health: Health {
                max: 100,
                current: 100,
            },
        }
    }
    fn fire_mage(i: usize, j: usize) -> Self {
        Self {
            texture_path: "textures/fire_mage.png".to_string(),
            mob_type: MobType::Mossling,
            mob: Mob { damage: 20 },
            loot: MobLoot { orbs: 3 },
            body_type: RigidBody::Kinematic,
            health: Health {
                max: 80,
                current: 80,
            },
        }
    }
}

#[derive(Component)]
pub struct Teleport {
    pub amount_of_tiles: u8,
    pub place_to_teleport: Vec<(u16, u16)>,
    pub time_to_teleport: Timer,
}

#[derive(Component)]
pub struct ShootAbility {
    pub time_to_shoot: Timer,
}
#[derive(Component)]
pub struct Mob {
    pub damage: i32,
}

#[derive(Component)]
pub struct MobLoot {
    pub orbs: u32,
}
// range for enum of mobs
impl rand::distributions::Distribution<MobType> for rand::distributions::Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> MobType {
        match rng.gen_range(0..=3) {
            0 => MobType::Mossling,
            1 => MobType::Teleport,
            _ => MobType::Mossling,
        }
    }
}

#[derive(Event)]
pub struct MobDeathEvent {
    pub entity: Entity,
    pub orbs: u32,
    pub pos: Vec3,
    pub dir: Vec3,
}

pub fn pick_mob_to_spawn(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    room: Res<LevelGenerator>,
    mut mob_map: ResMut<MobMap>,
) {
    let mut mob_id = 1;
    let grid = room.grid.clone();
    for i in 1..grid.len() - 1 {
        for j in 1..grid[i].len() - 1 {
            if grid[i][j] == TileType::Floor {
                let mut rng = rand::thread_rng();
                //need to fix 0 mob levels
                if rng.gen::<f32>() > 0.9 && (i > 18 || i < 14) && (j > 18 || j < 14) {
                    // make sure emenies don't spawn too close to player (todo: rewrite)

                    let mob_type: MobType = rand::random();
                    let texture_path: &str;
                    let mut can_shoot: bool = false;
                    let mut has_teleport: bool = false;
                    let mut amount_of_tiles: u8 = 0;
                    let timer: std::ops::Range<u64>;

                    match mob_type {
                        MobType::Mossling => {
                            texture_path = "textures/mob_mossling.png";
                            timer = 500..999;
                        }
                        MobType::Teleport => {
                            timer = 3000..5000;
                            amount_of_tiles = 4;
                            has_teleport = true;
                            can_shoot = true;
                            texture_path = "textures/mob_teleport_placeholder.png"
                        }
                    }

                    let mob = commands
                        .spawn(SpriteBundle {
                            texture: asset_server.load(texture_path),
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
                        .insert(Mob { damage: 20 })
                        .insert(Pathfinder {
                            path: vec![],
                            update_path_timer: Timer::new(
                                Duration::from_millis(rand::thread_rng().gen_range(timer.clone())),
                                TimerMode::Repeating,
                            ),
                            speed: 2500.,
                        })
                        .insert(MobLoot { orbs: 3 })
                        .insert(Health {
                            max: 100,
                            current: 100,
                        });

                    if has_teleport {
                        commands
                            .entity(mob)
                            .insert(Teleport { amount_of_tiles })
                            .insert(RigidBody::Kinematic);
                        mob_map.map[i][j] = mob_id;
                        mob_id += 1;
                    } else {
                        commands.entity(mob).insert(RigidBody::Dynamic);
                    }
                    if can_shoot {
                        commands.entity(mob).insert(ShootAbility {
                            time_to_shoot: Timer::new(
                                Duration::from_millis(rand::thread_rng().gen_range(timer)),
                                TimerMode::Repeating,
                            ),
                        });
                    }
                }
            }
        }
    }
}

fn teleport_mobs(
    mut mob_query: Query<(&mut Transform, &mut Pathfinder), (Without<Stun>, With<Teleport>)>,
) {
    // maybe add time dependency to teleport time? idk
    for (mut transform, mut mob) in mob_query.iter_mut() {
        if mob.path.len() > 0 {
            transform.translation = Vec3::new(
                mob.path[0].0 as f32 * ROOM_SIZE as f32,
                mob.path[0].1 as f32 * ROOM_SIZE as f32,
                1.0,
            );
            mob.path.remove(0);
        }
    }
}

fn move_mobs(
    mut mob_query: Query<
        (&mut LinearVelocity, &Transform, &mut Pathfinder),
        (Without<Stun>, Without<Teleport>),
    >,
    time: Res<Time>,
) {
    for (mut linvel, transform, mut pathfinder) in mob_query.iter_mut() {
        if pathfinder.path.len() > 0 {
            //let mob_tile_pos = Vec2::new(((transform.translation.x - (ROOM_SIZE / 2) as f32) / ROOM_SIZE as f32).floor(), (transform.translation.y - (ROOM_SIZE / 2) as f32) / ROOM_SIZE as f32).floor();
            let direction = Vec2::new(
                pathfinder.path[0].0 as f32 * 32. - transform.translation.x,
                pathfinder.path[0].1 as f32 * 32. - transform.translation.y,
            )
            .normalize();

            linvel.0 = direction * pathfinder.speed * time.delta_seconds();

            if transform.translation.truncate().distance(Vec2::new(
                pathfinder.path[0].0 as f32 * 32.,
                pathfinder.path[0].1 as f32 * 32.,
            )) <= 4.
            {
                pathfinder.path.remove(0);
            }
        }
    }
}

fn mob_shoot(
    mut ev_shoot: EventWriter<SpawnProjectileEvent>,
    mut mob_query: Query<(&Transform, &mut ShootAbility)>,
    mut player_query: Query<&Transform, (With<Player>, Without<Mob>)>,
    time: Res<Time>,
) {
    for (transform, mut can_shoot) in mob_query.iter_mut() {
        if let Ok(player) = player_query.get_single_mut() {
            can_shoot.time_to_shoot.tick(time.delta());
            if can_shoot.time_to_shoot.just_finished() {
                println!("Start to cast");
                let dir = (player.translation.truncate() - transform.translation.truncate())
                    .normalize_or_zero();
                let angle = dir.y.atan2(dir.x);
                ev_shoot.send(SpawnProjectileEvent {
                    texture_path: "textures/fireball.png".to_string(),
                    color: Color::srgb(2.5, 1.25, 1.0),
                    translation: transform.translation,
                    angle: angle,
                    radius: 8.0,
                    speed: 150.,
                    damage: 20,
                    element: crate::elements::ElementType::Fire,
                    is_friendly: false,
                });
                println!("sent event");
            }
        }
    }
}

fn hit_projectiles(
    mut commands: Commands,
    projectile_query: Query<(Entity, &Projectile, &Transform), With<Friendly>>,
    mut mob_query: Query<(Entity, &mut Health, &Transform, &MobLoot), With<Mob>>,
    mut ev_collision: EventReader<Collision>,
    mut ev_death: EventWriter<MobDeathEvent>,
) {
    for Collision(contacts) in ev_collision.read() {
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

                        // кидаем стан на моба
                        commands.entity(mob_e.unwrap()).insert(Stun::new(0.5));

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
    mut commands: Commands,

    mut portal_manager: ResMut<PortalManager>,
    player_experience: Res<PlayerExperience>,

    mut ev_spawn_portal: EventWriter<crate::level_completion::PortalEvent>,
    mut ev_spawn_orb: EventWriter<SpawnExpOrbEvent>,

    mut ev_mob_death: EventReader<MobDeathEvent>,
) {
    for ev in ev_mob_death.read() {
        portal_manager.set_pos(ev.pos);
        portal_manager.pop_mob();
        if portal_manager.no_mobs_on_level() {
            ev_spawn_portal.send(PortalEvent {
                pos: portal_manager.get_pos(),
            });
        }

        let orb_count = (ev.orbs + player_experience.orb_bonus) as i32;
        let half_count = (orb_count as f32 / 2.).round() as i32;

        let offset = PI / 12.;
        for i in (-orb_count / 2)..half_count {
            // считаем точки, куда будем выбрасывать частицы опыта
            let angle = ev.dir.y.atan2(ev.dir.x) + offset * i as f32;
            let direction = Vec2::from_angle(angle) * 32.0;
            let destination = Vec3::new(ev.pos.x + direction.x, ev.pos.y + direction.y, ev.pos.z);

            ev_spawn_orb.send(SpawnExpOrbEvent {
                pos: ev.pos,
                destination,
            });
        }

        commands.entity(ev.entity).despawn();
    }
}
