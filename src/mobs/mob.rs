//all things about mobs and their spawn/behaviour
use std::f32::consts::PI;

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    elements::{ElementResistance, ElementType},
    exp_orb::SpawnExpOrbEvent,
    experience::PlayerExperience,
    gamemap::Map,
    health::{Health, Hit},
    level_completion::{PortalEvent, PortalManager},
    obstacles::CorpseSpawnEvent,
    player::Player,
    projectile::{Friendly, Projectile, SpawnProjectileEvent},
    stun::Stun,
    GameLayer, GameState,
};

pub struct MobPlugin;

impl Plugin for MobPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MobDeathEvent>().add_systems(
            Update,
            (damage_mobs, mob_death, mob_shoot, hit_projectiles)
                .run_if(in_state(GameState::InGame)),
        );
    }
}

//Events for mobs
//event for mob death, contains amount of orbs, position of mob and direction where exp orbs will drop
#[derive(Event)]
pub struct MobDeathEvent {
    pub orbs: u32,
    pub pos: Vec3,
    pub dir: Vec3,
}

//Enum components========================================================================================================================================
//MobtypesHere(Better say mob names, bcz types are like turret, spawner etc.)
#[derive(Component, Clone)]
pub enum MobType {
    Knight,
    Mossling,
    FireMage,
    WaterMage,
    JungleTurret,
    Necromancer,
}

//projectile types
#[derive(Component, Clone)]
#[allow(dead_code)]
pub enum ProjectileType {
    // can use to create mobs with different types of projectiles
    Circle,  // spawn some projectiles around
    Missile, // like fireball
    Gatling, // a lot of small ones
}

//targets for mob pathfinding as enum
#[allow(unused)]
#[derive(Component)]
pub enum MobTarget {
    Player,
    Corpse,

    // Для моба-вора? ну типа
    HPTank,
    EXPTank,

    Runaway,
    Noone,
}
//Pure components=========================================================================================================================================
//If you want to add something (create new mob, or add new component), first of all, add components there (and check, maybe it exists already)
//ability to teleport, contains timer and range in tiles from target
#[derive(Component)]
pub struct Teleport {
    //todo: change to just tuple? maybe not?
    pub amount_of_tiles: u8,
    pub place_to_teleport: Vec<(u16, u16)>,
    pub time_to_teleport: Timer,
}

//component to mob and structures who can spawn enemy.
#[allow(dead_code)]
#[derive(Component)]
pub struct Summoning {
    pub time_to_spawn: Timer,
    pub is_static: bool,
}

//component to shoot, has timer, element and proj type according to enum ProjectileType
#[derive(Component)]
pub struct ShootAbility {
    pub time_to_shoot: Timer,
    pub element: ElementType,
    pub proj_type: ProjectileType,
}

//component to deal contact damage
#[derive(Component)]
pub struct Mob {
    pub damage: i32,
}

/// Struct for convenient mob sight handling 
#[derive(Debug)]
pub struct Ray {
    pub direction: Vec2,
    pub weight: f32,
}

/// Component for mobs that pursue player
#[derive(Component)]
pub struct SearchAndPursue {
    pub speed: f32,
    pub pursue_radius: f32,
    pub last_player_dir: Vec2,
    pub rays: Vec<Ray>,
}

impl Default for SearchAndPursue {
    fn default() -> Self {

        let mut rays: Vec<Ray> = vec![];

        for i in 0..16 {
            rays.push(Ray {
                direction: Vec2::from_angle(i as f32 * PI/8.),
                weight: 0.0
            })
        }

        Self {
            speed: 2000.0,
            pursue_radius: 256.0,
            last_player_dir: Vec2::ZERO,
            rays
        }
    }
}

//Component to raising mobs from the dead
#[derive(Component)]
pub struct Raising {
    pub mob_type: MobType,
    pub mob_pos: Transform,
    pub corpse_id: Entity,
}

//mob loot(amount of exp)
#[derive(Component)]
pub struct MobLoot {
    //todo: maybe add something like chance to spawn tank with exp/hp?
    // we can add an item for this
    pub orbs: u32,
}

//Flags===========================================================================================================================================
/// Flag to pathfinding: rush to player
#[derive(Component, Default)]
pub struct PlayerRush;

/// Flag to pathfinding: rush to corpse
#[derive(Component, Default)]
pub struct CorpseRush;

/// Flag to pathfinding: try to run away
#[derive(Component, Default)]
pub struct RunawayRush;

/// Flag for entities with rotation parts
#[derive(Component)]
pub struct RotationEntity;

/// Corpse flag, which shows that necromancer is trying to raise mob from this grave
#[derive(Component)]
pub struct BusyRaising;

/// This state should be applied to mob entity if it doesn't need to do anything in particular
#[derive(Component)]
pub struct Idle;

/// This state should be applied to mob entity if it need to be following player
#[derive(Component)]
pub struct PursuePlayer;

//Bundles===========================================================================================================================================
//Bundles of components, works like this: PhysicalBundle -> MobBundle -> MobTypeBundle (like turret),
//if you want to add mob - add bundle there and add impl later
// physical bundle with all physical stats
#[derive(Bundle)]
pub struct PhysicalBundle {
    collider: Collider,
    axes: LockedAxes,
    gravity: GravityScale,
    collision_layers: CollisionLayers,
    linear_velocity: LinearVelocity,
}

//bundle with all basic parameters of mob
#[derive(Bundle)]
pub struct MobBundle {
    //contains mob stats
    phys_bundle: PhysicalBundle,
    resistance: ElementResistance,
    mob_type: MobType,
    mob: Mob,
    loot: MobLoot,
    body_type: RigidBody,
    health: Health,
}

//implemenations
//change it, only if you know what you're doing
//add something there, and later go to spawn_mob
impl Mob {
    fn new(damage: i32) -> Self {
        Self { damage }
    }
}

impl Default for PhysicalBundle {
    //don't change if you're not sure
    fn default() -> Self {
        Self {
            collider: Collider::circle(6.),
            axes: LockedAxes::ROTATION_LOCKED,
            gravity: GravityScale(0.0),
            collision_layers: CollisionLayers::new(
                GameLayer::Enemy,
                [
                    GameLayer::Wall,
                    GameLayer::Projectile,
                    GameLayer::Shield,
                    GameLayer::Enemy,
                    GameLayer::Player,
                ],
            ),
            linear_velocity: LinearVelocity::ZERO,
        }
    }
}

impl MobBundle {
    pub fn knight() -> Self {
        Self {
            phys_bundle: PhysicalBundle::default(),
            resistance: ElementResistance {
                elements: vec![],
                resistance_percent: vec![0, 0, 0, 0, 0],
            },
            mob_type: MobType::Knight,
            mob: Mob::new(20),
            loot: MobLoot { orbs: 3 },
            body_type: RigidBody::Dynamic,
            health: Health::new(100),
        }
    }
    pub fn mossling() -> Self {
        Self {
            phys_bundle: PhysicalBundle::default(),
            resistance: ElementResistance {
                elements: vec![ElementType::Earth, ElementType::Water],
                resistance_percent: vec![0, 15, 15, 0, 0],
            },
            mob_type: MobType::Mossling,
            mob: Mob::new(20),
            loot: MobLoot { orbs: 3 },
            body_type: RigidBody::Dynamic,
            health: Health::new(100),
        }
    }
    pub fn turret() -> Self {
        Self {
            phys_bundle: PhysicalBundle::default(),
            resistance: ElementResistance {
                elements: vec![ElementType::Earth, ElementType::Water],
                resistance_percent: vec![0, 60, 60, 0, 0],
            },
            mob_type: MobType::JungleTurret,
            mob: Mob::new(20),
            loot: MobLoot { orbs: 3 },
            body_type: RigidBody::Static,
            health: Health::new(200),
        }
    }
    pub fn fire_mage() -> Self {
        Self {
            phys_bundle: PhysicalBundle::default(),
            resistance: ElementResistance {
                elements: vec![ElementType::Fire],
                resistance_percent: vec![80, 0, 0, 0, 0],
            },
            mob_type: MobType::FireMage,
            mob: Mob::new(20),
            loot: MobLoot { orbs: 3 },
            body_type: RigidBody::Static,
            health: Health::new(80),
        }
    }

    pub fn water_mage() -> Self {
        Self {
            phys_bundle: PhysicalBundle::default(),
            resistance: ElementResistance {
                elements: vec![ElementType::Water],
                resistance_percent: vec![0, 80, 0, 0, 0],
            },
            mob_type: MobType::WaterMage,
            mob: Mob::new(20),
            loot: MobLoot { orbs: 3 },
            body_type: RigidBody::Static,
            health: Health::new(80),
        }
    }

    pub fn necromancer() -> Self {
        Self {
            phys_bundle: PhysicalBundle::default(),
            resistance: ElementResistance {
                elements: vec![ElementType::Earth],
                resistance_percent: vec![0, 0, 30, 0, 0],
            },
            mob_type: MobType::Necromancer,
            mob: Mob::new(20),
            loot: MobLoot { orbs: 5 },
            body_type: RigidBody::Dynamic,
            health: Health::new(140),
        }
    }
}

//actual code==============================================================================================================================
fn mob_shoot(
    mut ev_shoot: EventWriter<SpawnProjectileEvent>,
    mut mob_query: Query<(&Transform, &mut ShootAbility, &mut RayCaster, &RayHits), Without<Stun>>,
    mut player_query: Query<(Entity, &Transform), (With<Player>, Without<Mob>)>,
    time: Res<Time>,
) {
    for (&transform, mut can_shoot, mut raycaster, ray_hits) in mob_query.iter_mut() {
        if let Ok((player_e, player_transform)) = player_query.get_single_mut() {
            let dir = (player_transform.translation.truncate() - transform.translation.truncate())
                .normalize_or_zero();
            raycaster.direction = Dir2::new_unchecked(dir);
       
            let hits = ray_hits.iter_sorted().collect::<Vec<RayHitData>>();

            can_shoot.time_to_shoot.tick(time.delta());
            if can_shoot.time_to_shoot.just_finished()
            && !hits.is_empty() {
                if hits[0].entity == player_e {
                    let angle = dir.y.atan2(dir.x); //math
                    let texture_path: String;
                    let damage: u32;

                    match can_shoot.proj_type {
                        //todo: change this fragment, that we could spawn small and circle projs, maybe change event?
                        ProjectileType::Circle => {
                            texture_path = "textures/earthquake.png".to_string();
                            damage = 20;
                        }
                        ProjectileType::Missile => {
                            texture_path = "textures/fireball.png".to_string();
                            damage = 25;
                        }
                        ProjectileType::Gatling => {
                            texture_path = "textures/small_fire.png".to_string();
                            damage = 10;
                        }
                    }

                    let color = can_shoot.element.color();

                    ev_shoot.send(SpawnProjectileEvent {
                        texture_path,
                        color, //todo: change this fragment, that we could spawn different types of projectiles.
                        translation: transform.translation,
                        angle,
                        radius: 8.0,
                        speed: 150.,
                        damage,
                        element: can_shoot.element,
                        is_friendly: false,
                    });
                }
            }
        }
    }
}

fn hit_projectiles(
    mut commands: Commands,
    projectile_query: Query<(Entity, &Projectile, &Transform), With<Friendly>>,
    mut mob_query: Query<(&CollidingEntities, &mut Health, &Transform, &ElementResistance), With<Mob>>,
) {
    for (colliding_e, mut health, mob_transform, resistance) in mob_query.iter_mut() {
        for (proj_e, projectile, projectile_transform) in projectile_query.iter() {
            if colliding_e.contains(&proj_e) {
                // считаем урон с учётом сопротивления к элементам
                let mut damage = projectile.damage as i32;
                resistance.calculate_for(&mut damage, Some(projectile.element));

                // направление выстрела
                let shot_dir =
                    (mob_transform.translation - projectile_transform.translation).normalize();

                // пушим в очередь попадание
                health.hit_queue.push( Hit {
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

fn damage_mobs(
    mut commands: Commands,
    mut ev_death: EventWriter<MobDeathEvent>,
    mut ev_corpse: EventWriter<CorpseSpawnEvent>,
    mut mob_query: Query<
        (
            Entity,
            &mut Health,
            &mut Mob,
            &Transform,
            &MobLoot,
            &MobType,
        ),
        With<Mob>,
    >,
) {
    for (entity, mut health, _mob, transform, loot, mob_type) in mob_query.iter_mut() {
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

                // события "поcле смерти"
                ev_death.send(MobDeathEvent {
                    orbs: loot.orbs,
                    pos: transform.translation,
                    dir: hit.direction,
                });

                // спавним труп на месте смерти моба
                ev_corpse.send(CorpseSpawnEvent {
                    mob_type: mob_type.clone(),
                    pos: transform.translation.with_z(0.05),
                });
            }
        }
    }
}

fn mob_death(
    mut portal_manager: ResMut<PortalManager>,
    player_experience: Res<PlayerExperience>,

    mut ev_spawn_portal: EventWriter<crate::level_completion::PortalEvent>,
    mut ev_spawn_orb: EventWriter<SpawnExpOrbEvent>,

    mut ev_mob_death: EventReader<MobDeathEvent>,

    mut mob_map: ResMut<Map>,
) {
    for ev in ev_mob_death.read() {
        portal_manager.set_pos(ev.pos);
        portal_manager.pop_mob();
        let mob_pos = (
            (ev.pos.x.floor() / 32.).floor() as u16,
            (ev.pos.y.floor() / 32.).floor() as u16,
        );

        mob_map
            .map
            .get_mut(&(mob_pos.0, mob_pos.1))
            .unwrap()
            .mob_count -= 1;

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
    }
}
