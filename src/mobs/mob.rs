use std::time::Duration;

use bevy_common_assets::json::JsonAssetPlugin;

//all things about mobs and their spawn/behaviour
use {
    avian2d::prelude::*, bevy::prelude::*, rand::Rng, seldom_state::prelude::*,
    std::f32::consts::PI,
    serde_json::{Map as JsonMap, Value},
};
///add mobs with kinematic body type
pub const STATIC_MOBS: &[MobType] = &[
    MobType::JungleTurret,
    MobType::FireMage,
    MobType::WaterMage,
    MobType::EarthElemental,
];

use crate::animation::AnimationConfig;
use crate::mobs::rotate_orbital;
use crate::{
    blank_spell::SpawnBlankEvent,
    elements::{ElementResistance, ElementType},
    exp_orb::SpawnExpOrbEvent,
    experience::PlayerExperience,
    friend::Friend,
    gamemap::Map,
    health::{Health, Hit},
    level_completion::{PortalEvent, PortalManager},
    mobs::timer_tick_orbital,
    obstacles::{Corpse, CorpseSpawnEvent},
    particles::SpawnParticlesEvent,
    player::Player,
    projectile::{Friendly, Hostile, Projectile, SpawnProjectileEvent},
    stun::Stun,
    GameLayer, GameState,
};
pub struct MobPlugin;

impl Plugin for MobPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(JsonAssetPlugin::<MobDatabase>::new(&["json"]))
            .add_systems(Startup, load_mob_database)
            .add_event::<MobDeathEvent>()
            .add_systems(
            Update,
            (
                damage_mobs,
                mob_death,
                mob_shoot::<Friend, ShootAbility, Friend, Friendly, Hostile>,
                mob_shoot::<Mob, Friend, Friendly, Friend, Friendly>,
                mob_attack::<Enemy>,
                mob_attack::<Friend>,
                hit_projectiles::<Player, Friend, Hostile>,
                hit_projectiles::<Friend, Mob, Friendly>,
                timer_shoot,
                rotate_orbital::<Friend>,
                rotate_orbital::<Enemy>,
                timer_tick_orbital::<Enemy>,
                timer_tick_orbital::<Friend>,
                attack_hit::<Friend, Enemy>,
                attack_hit::<Enemy, Friend>,
                tick_attack_cooldown,
                timer_empty_list,
            )
                .run_if(in_state(GameState::InGame)),
        );
    }
}

#[derive(serde::Deserialize, Asset, TypePath)]
pub struct MobDatabase {
    pub mobs: Vec<JsonMap<String, Value>>,
}

#[derive(Resource)]
pub struct MobDatabaseHandle(pub Handle<MobDatabase>); 

fn load_mob_database(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(MobDatabaseHandle(asset_server.load("mobs.json")));
}

//Events for mobs
///event for mob death, contains amount of orbs, position of mob and direction where exp orbs will drop
#[derive(Event)]
pub struct MobDeathEvent {
    pub orbs: u32,
    pub pos: Vec3,
    pub dir: Vec3,
}

//Enum components========================================================================================================================================
///MobtypesHere(Better say mob names, bcz types are like turret, spawner etc.)
#[derive(Component, Clone, PartialEq)]
pub enum MobType {
    Knight,
    Mossling,
    FireMage,
    WaterMage,
    JungleTurret,
    Necromancer,
    Koldun,
    ClayGolem,      //walking tank(like fat, not shooting, attacks around)
    WaterElemental, //walking range mob
    FireElemental,  //like ghost(melee mob with phasing) // done
    SkeletWarrior,  //mechanics
    SkeletMage,     //mechanics
    SkeletRanger,   // add arrow texture
    EarthElemental, //turret i guess? //done
    AirElemental,   // just an orbital?
    TankEater,
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

#[derive(Clone)]
#[allow(dead_code)]
pub enum AttackType {
    Slash,
    Rush,
    Spear,
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
#[derive(Component, Clone)]
pub struct Teleport {
    //todo: change to just tuple? maybe not?
    pub amount_of_tiles: u8,
    pub place_to_teleport: Vec<(u16, u16)>,
    pub time_to_teleport: Timer,
}
///Flag to teleport state
#[derive(Component, Clone)]
pub struct TeleportFlag;

///Flag to mobs with phasing
#[derive(Component, Clone)]
pub struct Phasing {
    pub speed: f32,
}
#[derive(Component, Clone)]
pub struct PhasingFlag;

//component to mob and structures who can spawn enemy.
#[allow(dead_code)]
#[derive(Component)]
pub struct Summoning {
    pub time_to_spawn: Timer,
    pub is_static: bool,
}

//component to shoot, has timer, element and proj type according to enum ProjectileType
#[derive(Component, Clone)]
pub struct ShootAbility {
    pub time_to_shoot: Timer,
    pub element: ElementType,
    pub proj_type: ProjectileType,
}
///flag to state system
#[derive(Component, Clone)]
pub struct ShootFlag;

//component to deal contact damage
#[derive(Component)]
pub struct Mob {
    pub damage: i32,
}

/// Marks mob to be hostile towards player and friends
#[derive(Component, Default)]
pub struct Enemy;

#[derive(Component)]
pub struct Orbital {
    pub time_to_live: Timer,
    pub is_eternal: bool,
    pub speed: f32,
    pub parent: Option<Box<Entity>>,
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
///TEMPORARY COMP FOR MOB AI
#[derive(Component, Clone)]
pub struct RaisingFlag;
//mob loot(amount of exp)

#[derive(Component)]
pub struct HitList {
    timer_to_clear: Timer,
    been_punched: bool,
    id_list: Vec<u16>,
}
impl Default for HitList {
    fn default() -> Self {
        Self {
            timer_to_clear: Timer::new(Duration::from_millis(5000), TimerMode::Repeating),
            been_punched: false,
            id_list: vec![],
        }
    }
}
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
#[derive(Component, Default, Clone)]
pub struct CorpseRush;

/// Flag to pathfinding: try to run away
#[derive(Component, Default, Clone)]
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
#[derive(Component, Clone)]
pub struct Idle;

/// This state should be applied to STATIC mob entity if it doesn't need to do anything in particular(for state machines)
#[derive(Component, Clone)]
pub struct IdleStatic;

/// This state make mob search for target and follow it
#[derive(Component, Clone)]
pub struct Pursue;

/// This Flag is for mob attack animation
#[derive(Component, Clone)]
pub struct Attack {
    pub damage: i32,
    pub element: Option<ElementType>,
    pub dir: Vec2,
    pub hit_id: u16,
}

/// This Flag is for mob AI (melee attacks)
#[derive(Component, Clone)]
pub struct AttackFlag;

/// This Flag make mob attack in melee if he's near his range
#[derive(Component, Clone)]
pub struct AttackComponent {
    pub range: f32,
    pub attack_type: AttackType,
    pub target: Option<Entity>,
    pub cooldown: Timer,
    pub attacked: bool,
    pub damage: i32,
    pub element: Option<ElementType>,
}

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
    pub phys_bundle: PhysicalBundle,
    pub resistance: ElementResistance,
    pub mob_type: MobType,
    pub mob: Mob,
    pub loot: MobLoot,
    pub body_type: RigidBody,
    pub health: Health,
    pub hit_list: HitList,
}

//implemenations
//change it, only if you know what you're doing
//add something there, and later go to spawn_mob
impl Mob {
    pub fn new(damage: i32) -> Self {
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
impl Default for MobBundle {
    fn default() -> Self {
        Self {
            phys_bundle: PhysicalBundle::default(),
            resistance: ElementResistance {
                elements: vec![],
                resistance_percent: vec![0, 0, 0, 0, 0],
            },
            mob_type: MobType::Mossling,
            mob: Mob::new(20),
            loot: MobLoot { orbs: 3 },
            body_type: RigidBody::Dynamic,
            health: Health::new(100),
            hit_list: HitList::default(),
        }
    }
}

fn mob_attack<Who: Component + std::default::Default>(
    mut commands: Commands,
    //    spatial_query: SpatialQuery,
    mob_query: Query<
        (Entity, &mut LinearVelocity, &Transform, &AttackComponent),
        (Changed<AttackFlag>, With<Who>),
    >,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    asset_server: Res<AssetServer>,
    transform_query: Query<&Transform>,
) {
    for (entity, _a, mob_transform, range) in mob_query.iter() {
        let dir;
        match range.target {
            None => continue,
            Some(parent) => {
                dir = transform_query.get(parent).unwrap().translation.truncate()
                    - mob_transform.translation.truncate();
                commands.entity(entity).insert(Done::Success);
            }
        };

        let hit_id: u16 = rand::thread_rng().gen::<u16>();

        let animation_config = AnimationConfig::new(0, 4, 24);
        let mut texture_path;

        let mut friendly: bool = false;

        if std::any::type_name::<Who>() == std::any::type_name::<Friend>() {
            friendly = true;
        }
        match range.attack_type {
            AttackType::Slash => {
                texture_path = "textures/slash_horisontal3_enemy.png";

                if friendly {
                    texture_path = "textures/slash_horisontal3.png";
                    println!("GOT heRe");
                }
            }
            AttackType::Spear => {
                texture_path = "textures/slash_horisontal3_enemy.png";

                if friendly {
                    texture_path = "textures/slash_horisontal3.png";
                }
            }
            AttackType::Rush => {
                texture_path = "textures/slash_horisontal3_enemy.png";

                if friendly {
                    texture_path = "textures/slash_horisontal3.png";
                }
            } // todo: change to choose from mob type?
        };

        let texture = asset_server.load(texture_path);
        let layout = TextureAtlasLayout::from_grid(UVec2::splat(16), 5, 1, None, None);
        let texture_atlas_layout = texture_atlas_layouts.add(layout);

        let mut transform_attack: Transform = Transform::from_xyz(0., 0., 0.);

        let pos_new = Vec2::from_angle(dir.y.atan2(dir.x)) * range.range;

        transform_attack.translation = Vec3::new(pos_new.x, pos_new.y, 0.);
        transform_attack.rotation = Quat::from_rotation_z(dir.normalize_or_zero().to_angle());

        commands.entity(entity).with_children(|parent| {
            parent
                .spawn(SpriteBundle {
                    texture,
                    transform: transform_attack,
                    ..default()
                })
                .insert(animation_config)
                .insert(Attack {
                    damage: range.damage,
                    element: range.element,
                    dir: dir,
                    hit_id: hit_id,
                })
                .insert(TextureAtlas {
                    layout: texture_atlas_layout.clone(),
                    index: 0,
                })
                .insert(Collider::rectangle(16., 16.))
                .insert(Sensor)
                .insert(LockedAxes::ROTATION_LOCKED)
                .insert(Who::default());
        });
    }
}

fn tick_attack_cooldown(time: Res<Time>, mut mob_query: Query<&mut AttackComponent>) {
    for mut attack_cd in mob_query.iter_mut() {
        if attack_cd.attacked {
            attack_cd.cooldown.tick(time.delta());

            if attack_cd.cooldown.just_finished() {
                attack_cd.attacked = false;
            }
        }
    }
}

fn attack_hit<Who: Component, Target: Component>(
    mut attack_query: Query<(Entity, &mut Attack), (With<Who>, Without<Target>)>,
    mut target_query: Query<
        (&CollidingEntities, &mut Health, &ElementResistance, &mut HitList),
        (Without<Who>, With<Target>),
    >,
) {
    for (entities, mut hp, el_res, mut hit_list) in target_query.iter_mut() {
        //maybe apply el_res?
        for (attack_e, attack) in attack_query.iter_mut() {
            if entities.contains(&attack_e) && !hit_list.id_list.contains(&attack.hit_id){
                let mut damage = attack.damage;
                el_res.calculate_for(&mut damage, attack.element);
                
                hit_list.id_list.push(attack.hit_id);
                hit_list.been_punched = true;
                hit_list.timer_to_clear.reset();

                hp.hit_queue.push(Hit {
                    damage: damage,
                    element: attack.element,
                    direction: Vec3::new(attack.dir.x, attack.dir.y, 1.0),
                });
            }
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
    mut commands: Commands,
    spatial_query: SpatialQuery,
    mut ev_shoot: EventWriter<SpawnProjectileEvent>,
    mut shoot_query: Query<
        (Entity, &Transform, &mut ShootAbility),
        (
            With<Shoot>,
            Without<Stun>,
            Without<Filter1>,
            With<ShootFlag>,
        ),
    >,
    target_query: Query<(Entity, &Transform), (With<Target>, Without<Filter2>)>,
    avoid_query: Query<Entity, With<Corpse>>,
) {
    for (mob_e, &mob_transform, can_shoot) in shoot_query.iter_mut() {
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
                commands.entity(mob_e).insert(Done::Failure);
                continue;
            };
            let mut friendly_proj: bool = false;

            if std::any::type_name::<ProjType>() == std::any::type_name::<Friendly>() {
                friendly_proj = true;
            }

            if first_hit.entity == target_e {
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
            } else if first_hit.entity != target_e {
                commands.entity(mob_e).insert(Done::Failure);
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
    mut ev_spawn_particles: EventWriter<SpawnParticlesEvent>,
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

                // спавним партиклы
                ev_spawn_particles.send(SpawnParticlesEvent {
                    pattern: crate::particles::ParticlePattern::Burst {
                        direction: -projectile.direction,
                        distance: rand::thread_rng().gen_range(8.0..12.0),
                        spread: PI / 3.,
                    },
                    position: projectile_transform.translation,
                    amount: 3,
                    color: projectile.element.color(),
                    speed: 10.,
                    rotate: false,
                });
            }
        }
    }
}
pub fn timer_empty_list(time: Res<Time>, mut list_query: Query<&mut HitList>) {
    for mut list in list_query.iter_mut() {
        if list.been_punched {
            list.timer_to_clear.tick(time.delta());
            if list.timer_to_clear.just_finished() {
                list.been_punched = false;
                list.id_list = vec![];
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
    mut blank_spawn_ev: EventWriter<SpawnBlankEvent>,
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

                if *mob_type == MobType::AirElemental {
                    blank_spawn_ev.send(SpawnBlankEvent {
                        range: 8.,
                        position: transform.translation,
                        speed: 10.,
                        side: false,
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

fn mob_death(
    mut portal_manager: ResMut<PortalManager>,
    player_experience: Res<PlayerExperience>,

    mut ev_spawn_portal: EventWriter<crate::level_completion::PortalEvent>,
    mut ev_spawn_orb: EventWriter<SpawnExpOrbEvent>,
    mut ev_spawn_particles: EventWriter<SpawnParticlesEvent>,

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

        // спавним партиклы
        ev_spawn_particles.send(SpawnParticlesEvent {
            pattern: crate::particles::ParticlePattern::Circle {
                radius: rand::thread_rng().gen_range(12.0..20.0),
            },
            position: ev.pos,
            amount: rand::thread_rng().gen_range(6..10),
            color: Color::hsl(10., 1., 0.5),
            speed: 5.,
            rotate: false,
        });
    }
}

pub fn timer_shoot(
    mut shoot_query: Query<(Entity, &mut ShootAbility)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (mob_e, mut timer) in shoot_query.iter_mut() {
        timer.time_to_shoot.tick(time.delta());
        if timer.time_to_shoot.just_finished() {
            commands.entity(mob_e).insert(Done::Success);
        }
    }
}
