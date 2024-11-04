//all things about mobs and their spawn/behaviour
use std::f32::consts::PI;

use avian2d::prelude::*;
use bevy::prelude::*;
///add mobs with kinematic body type
pub const STATIC_MOBS: &[MobType] = &[MobType::JungleTurret, MobType::FireMage, MobType::WaterMage];

use crate::{
    elements::{ElementResistance, ElementType},
    exp_orb::SpawnExpOrbEvent,
    experience::PlayerExperience,
    friend::Friend,
    gamemap::Map,
    health::{Health, Hit},
    level_completion::{PortalEvent, PortalManager},
    obstacles::{Corpse, CorpseSpawnEvent},
    player::Player,
    projectile::{Friendly, Hostile, Projectile, SpawnProjectileEvent},
    stun::Stun,
    GameLayer, GameState,
};

pub struct MobPlugin;

impl Plugin for MobPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MobDeathEvent>().add_systems(
            Update,
            (
                damage_mobs,
                mob_death,
                mob_shoot::<Friend, ShootAbility, Friend, Friendly, Hostile>,
                mob_shoot::<Mob, Friend, Friendly, Friend, Friendly>,
                hit_projectiles::<Player, Friend, Hostile>,
                hit_projectiles::<Friend, Mob, Friendly>,
            )
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
#[derive(Component, Clone, PartialEq)]
pub enum MobType {
    Knight,
    Mossling,
    FireMage,
    WaterMage,
    JungleTurret,
    Necromancer,
    Koldun,
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
    pub search_time: Timer,
    pub wander_timer: Timer,
    pub pursue_radius: f32,
    pub last_target_dir: Vec2,
    pub rays: Vec<Ray>,
}

impl Default for SearchAndPursue {
    fn default() -> Self {
        let mut rays: Vec<Ray> = vec![];

        for i in 0..16 {
            rays.push(Ray {
                direction: Vec2::from_angle(i as f32 * PI / 8.),
                weight: 0.0,
            })
        }

        Self {
            speed: 2000.0,
            search_time: Timer::from_seconds(5., TimerMode::Once),
            wander_timer: Timer::from_seconds(3., TimerMode::Repeating),
            pursue_radius: 256.0,
            last_target_dir: Vec2::ZERO,
            rays,
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

#[derive(Component)]
pub struct FlipEntity;
/// Corpse flag, which shows that necromancer is trying to raise mob from this grave
#[derive(Component)]
pub struct BusyRaising;

/// This state should be applied to mob entity if it doesn't need to do anything in particular
#[derive(Component)]
pub struct Idle;

/// This state should be applied to mob entity if it need to be following player and friends
#[derive(Component)]
pub struct PursueFriends;

/// This state should be applied to mob entity if it need to be following ONLY friends
//#[derive(Component)]
//pub struct PursueFriendsOnly;

/// This state should be applied to mob entity if it need to be following enemy mobs
#[derive(Component)]
pub struct PursueMobs;

//Bundles===========================================================================================================================================
//Bundles of components, works like this: PhysicalBundle -> MobBundle -> MobTypeBundle (like turret),
//if you want to add mob - add bundle there and add impl later
// physical bundle with all physical stats
#[derive(Bundle)]
pub struct PhysicalBundle {
    pub collider: Collider,
    pub axes: LockedAxes,
    pub gravity: GravityScale,
    pub collision_layers: CollisionLayers,
    pub linear_velocity: LinearVelocity,
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
                    GameLayer::Friend,
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
    pub fn koldun() -> Self {
        Self {
            phys_bundle: PhysicalBundle {
                collider: Collider::circle(24.),
                ..default()
            },
            resistance: ElementResistance {
                elements: vec![
                    ElementType::Earth,
                    ElementType::Air,
                    ElementType::Fire,
                    ElementType::Water,
                ],
                resistance_percent: vec![20, 20, 20, 20, 20],
            },
            mob_type: (MobType::Koldun),
            mob: Mob::new(40),
            loot: MobLoot { orbs: 100 },
            body_type: RigidBody::Dynamic,
            health: Health::new(2000),
        }
    }
}

//actual code==============================================================================================================================
fn mob_shoot<
    Target: Component,
    Shoot: Component,
    Filter1: Component,
    Filter2: Component,
    ProjType: Component,
>(
    spatial_query: SpatialQuery,
    mut ev_shoot: EventWriter<SpawnProjectileEvent>,
    mut shoot_query: Query<
        (Entity, &Transform, &mut ShootAbility),
        (With<Shoot>, Without<Stun>, Without<Filter1>),
    >,
    target_query: Query<(Entity, &Transform), (With<Target>, Without<Filter2>)>,
    time: Res<Time>,
    avoid_query: Query<Entity, With<Corpse>>,
) {
    for (mob_e, &mob_transform, mut can_shoot) in shoot_query.iter_mut() {
        if target_query.iter().len() != 0 {
            let mut target_transform: Transform = mob_transform.clone();
            let mut target_e: Entity = mob_e;
            let mut dist: f32 = f32::MAX;
            for (temp_e, temp_pos) in target_query.iter() {
                let temp_dist: f32 = (mob_transform.translation - temp_pos.translation)
                    .x
                    .powf(2.)
                    + (mob_transform.translation - temp_pos.translation)
                        .y
                        .powf(2.);
                if dist > temp_dist {
                    dist = temp_dist;
                    target_transform = *temp_pos;
                    target_e = temp_e;
                }
            }
            let dir = (target_transform.translation.truncate()
                - mob_transform.translation.truncate())
            .normalize_or_zero();

            let Some(first_hit) = spatial_query.cast_ray_predicate(
                mob_transform.translation.truncate(),
                Dir2::new_unchecked(dir),
                512.,
                true,
                SpatialQueryFilter::default().with_excluded_entities(&avoid_query),
                &|entity| entity != mob_e,
            ) else {
                continue;
            };
            let mut friendly_proj: bool = false;
            
            if std::any::type_name::<ProjType>() == std::any::type_name::<Friendly>() {
                friendly_proj = true;
            }

            can_shoot.time_to_shoot.tick(time.delta());
            if can_shoot.time_to_shoot.just_finished() && first_hit.entity == target_e {
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
                    translation: mob_transform.translation,
                    angle,
                    radius: 8.0,
                    speed: 150.,
                    damage,
                    element: can_shoot.element,
                    is_friendly: friendly_proj,
                });
            }
        }
    }
}

fn hit_projectiles<Filter: Component, FilterTrue: Component, Side: Component>(
    mut commands: Commands,
    projectile_query: Query<(Entity, &Projectile, &Transform), With<Side>>,
    mut mob_query: Query<
        (
            &CollidingEntities,
            &mut Health,
            &Transform,
            &ElementResistance,
        ),
        (With<Mob>, Without<Filter>, With<FilterTrue>),
    >,
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

pub fn damage_mobs(
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
        (With<Mob>, Without<Friend>),
    >,
    mut mob_map: ResMut<Map>,
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

fn mob_death(
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
    }
}
