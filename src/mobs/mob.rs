use std::time::Duration;

use super::{
    BossAttackSystem, BusyOrbital, ItemPicked, OnDeathEffect, OnHitEffect, PickupItem,
    PickupItemQueue,
};

use bevy_common_assets::json::JsonAssetPlugin;

//all things about mobs and their spawn/behaviour
use {
    avian2d::prelude::*,
    bevy::prelude::*,
    rand::Rng,
    seldom_state::prelude::*,
    serde_json::{Map as JsonMap, Value},
    std::f32::consts::PI,
};

///add mobs with kinematic body type
pub const STATIC_MOBS: &[MobType] = &[
    MobType::JungleTurret,
    MobType::FireMage,
    MobType::WaterMage,
    MobType::EarthElemental,
];

use crate::{
    animation::AnimationConfig,
    audio::PlayAudioEvent,
    exp_tank::SpawnExpTankEvent,
    health_tank::SpawnHealthTankEvent,
    item::{ItemDatabase, ItemDatabaseHandle, ItemType, SpawnItemEvent},
    pathfinding::Pathfinder,
    save::{Save, SaveHandle},
};
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
    obstacles::CorpseSpawnEvent,
    particles::SpawnParticlesEvent,
    player::Player,
    projectile::{Friendly, Hostile, Projectile, SpawnProjectileEvent},
    stun::Stun,
    GameLayer, GameState,
};
use crate::{
    invincibility::Invincibility,
    mobs::{item_queue_update, rotate_orbital, set_state_thief, thief_collide, PushItemQueryEvent},
};
pub struct MobPlugin;

impl Plugin for MobPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(JsonAssetPlugin::<MobDatabase>::new(&["json"]))
            .add_systems(Startup, load_mob_database)
            .add_event::<MobDeathEvent>()
            .add_event::<PushItemQueryEvent>()
            .add_event::<OnDeathEffectEvent>()
            .add_event::<OnHitEffectEvent>()
            .add_systems(
                Update,
                (on_death_effects_handler).run_if(in_state(GameState::InGame)),
            )
            .add_systems(
                Update,
                (
                    damage_mobs,
                    mob_death,
                    thief_collide,
                    mob_attack::<Enemy>,
                    mob_attack::<Friend>,
                    hit_projectiles::<Player, Friend, Hostile>,
                    hit_projectiles::<Friend, Mob, Friendly>,
                    rotate_orbital::<Friend>,
                    rotate_orbital::<Enemy>,
                    timer_tick_orbital::<Enemy>,
                    timer_tick_orbital::<Friend>,
                    attack_hit::<Friend, Enemy>,
                    attack_hit::<Enemy, Friend>,
                    tick_attack_cooldown,
                    timer_empty_list,
                    before_attack_delay,
                    item_queue_update,
                    on_hit_effects,
                    pos_pathfinder,
                    set_state_thief,
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

fn load_mob_database(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(MobDatabaseHandle(asset_server.load("mobs.json")));
}

//Events for mobs
///event for mob death, contains amount of orbs, position of mob and direction where exp orbs will drop
#[derive(Event)]
pub struct MobDeathEvent {
    pub mob_unlock_tag: String,
    pub orbs: u32,
    pub pos: Vec3,
    pub dir: Vec3,
    pub is_spawned: bool,
}

#[derive(Event)]
pub struct OnHitEffectEvent {
    pub pos: Vec3,
    pub dir: Vec3,
    pub vec_of_objects: Vec<i32>,
    pub on_hit_effect_type: OnHitEffect,
    pub is_friendly: bool,
}

#[derive(Event)]
pub struct OnDeathEffectEvent {
    pub pos: Vec3,
    pub dir: Vec3,
    pub vec_of_objects: Vec<i32>,
    pub on_death_effect_type: OnDeathEffect,
    pub is_friendly: bool,
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
    Thief,
}

//projectile types
#[derive(Component, Clone)]
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
    Range,
    Circle,
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

impl SearchAndPursue {
    pub fn range_units() -> Self {
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
            pursue_radius: 512.0,
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
            timer_to_clear: Timer::new(Duration::from_millis(2000), TimerMode::Repeating),
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

#[derive(Component, Clone)]
pub struct BeforeAttackDelay {
    pub timer: Timer,
}

impl Default for BeforeAttackDelay {
    fn default() -> Self {
        Self {
            timer: Timer::new(Duration::from_millis(350), TimerMode::Once),
        }
    }
}

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
    pub proj_type: Option<ProjectileType>,
}

impl Default for AttackComponent {
    fn default() -> Self {
        Self {
            range: 28.,
            attack_type: AttackType::Slash,
            target: None,
            cooldown: Timer::new(Duration::from_millis(2000), TimerMode::Repeating),
            attacked: true,
            damage: 1,
            element: None,
            proj_type: None,
        }
    }
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
    pub exp_loot: MobLoot,
    pub item_loot: PickupItem,
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
            exp_loot: MobLoot { orbs: 3 },
            item_loot: PickupItem {
                item_type: ItemPicked::Obstacle,
                item_name: None,
            },
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
    mut ev_shoot: EventWriter<SpawnProjectileEvent>,
) {
    for (entity, _a, mob_transform, range) in mob_query.iter() {
        let dir;
        match range.target {
            None => continue,
            Some(parent) => {
                if transform_query.contains(parent) {
                    dir = transform_query.get(parent).unwrap().translation.truncate()
                        - mob_transform.translation.truncate();
                    commands.entity(entity).insert(Done::Success);
                } else {
                    commands.entity(entity).insert(Done::Success);
                    continue;
                }
            }
        };

        let hit_id: u16 = rand::thread_rng().gen::<u16>();

        let animation_config = AnimationConfig::new(0, 4, 24);
        let mut texture_path;

        let mut friendly: bool = false;

        if std::any::type_name::<Who>() == std::any::type_name::<Friend>() {
            friendly = true;
        }

        let mut transform_attack: Transform = Transform::from_xyz(0., 0., 0.);

        let pos_new = Vec2::from_angle(dir.y.atan2(dir.x)) * range.range;

        transform_attack.translation = Vec3::new(pos_new.x, pos_new.y, 0.);
        transform_attack.rotation = Quat::from_rotation_z(dir.normalize_or_zero().to_angle());
        let mut multiple_to_spawn = false;
        let mut amount_to_spawn = 1;

        match range.attack_type {
            AttackType::Slash => {
                texture_path = "textures/slash_horisontal_enemy.png";

                if friendly {
                    texture_path = "textures/slash_horisontal.png";
                }
            }
            AttackType::Spear => {
                texture_path = "textures/pierce_enemy.png";

                if friendly {
                    texture_path = "textures/pierce.png";
                }
            }
            AttackType::Circle => {
                texture_path = "textures/slash_horisontal_enemy.png";

                if friendly {
                    texture_path = "textures/slash_horisontal.png";
                } // change

                multiple_to_spawn = true;
                amount_to_spawn = 16;
            }
            AttackType::Rush => {
                texture_path = "textures/slash_horisontal_enemy.png";

                if friendly {
                    texture_path = "textures/slash_horisontal.png";
                }
            } // todo: change to choose from mob type?

            AttackType::Range => {
                commands.entity(entity).insert(Done::Success);

                let angle = dir.y.atan2(dir.x); //math
                let texture_path: String;
                let damage: u32;
                match range.proj_type {
                    //todo: change this fragment, that we could spawn small and circle projs, maybe change event?
                    Some(ProjectileType::Circle) => {
                        texture_path = "textures/earthquake.png".to_string();
                        damage = 20;
                    }
                    Some(ProjectileType::Missile) => {
                        texture_path = "textures/fireball.png".to_string();
                        damage = 25;
                    }
                    Some(ProjectileType::Gatling) => {
                        texture_path = "textures/small_fire.png".to_string();
                        damage = 10;
                    }
                    None => continue,
                };

                let color = range
                    .element
                    .expect("Range attack without element, refactor this code.")
                    .color();

                ev_shoot.send(SpawnProjectileEvent {
                    texture_path,
                    color, //todo: change this fragment, that we could spawn different types of projectiles.
                    translation: mob_transform.translation,
                    angle,
                    collider_radius: 8.0,
                    speed: 150.,
                    damage,
                    element: range
                        .element
                        .expect("Range attack without element, refactor this code."),
                    is_friendly: friendly,
                    trajectory: crate::projectile::Trajectory::Straight,
                    can_go_through_walls: false,
                });

                continue;
            }
        };

        let texture = asset_server.load(texture_path);
        let layout = TextureAtlasLayout::from_grid(UVec2::splat(16), 5, 1, None, None);
        let texture_atlas_layout = texture_atlas_layouts.add(layout);
        if multiple_to_spawn {
            for i in 0..amount_to_spawn {
                let dir_multiple = Vec2::from_angle(i as f32 * PI * 2. / amount_to_spawn as f32);

                let mut transform_attack_multiple: Transform = Transform::from_xyz(0., 0., 0.);

                let pos_new = Vec2::from_angle(dir_multiple.y.atan2(dir_multiple.x)) * range.range;

                transform_attack_multiple.translation = Vec3::new(pos_new.x, pos_new.y, 0.);
                transform_attack_multiple.rotation =
                    Quat::from_rotation_z(dir_multiple.normalize_or_zero().to_angle());

                commands.entity(entity).with_children(|parent| {
                    parent
                        .spawn(SpriteBundle {
                            texture: texture.clone(),
                            transform: transform_attack_multiple,
                            ..default()
                        })
                        .insert(animation_config.clone())
                        .insert(Attack {
                            damage: range.damage,
                            element: range.element,
                            dir: dir_multiple,
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
            continue;
        }
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
        (
            &CollidingEntities,
            &mut Health,
            &ElementResistance,
            &mut HitList,
        ),
        (Without<Who>, With<Target>, Without<Invincibility>),
    >,
) {
    for (entities, mut hp, el_res, mut hit_list) in target_query.iter_mut() {
        //maybe apply el_res?
        for (attack_e, attack) in attack_query.iter_mut() {
            if entities.contains(&attack_e) && !hit_list.id_list.contains(&attack.hit_id) {
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
pub fn mob_type_to_tag_convert(mob_type: MobType) -> String {
    match mob_type {
        MobType::Knight => "knight.png",
        MobType::Mossling => "mossling.png",
        MobType::FireMage => "fire_mage.png",
        MobType::WaterMage => "water_mage.png",
        MobType::JungleTurret => "plant.png",
        MobType::Necromancer => "necromancer.png",
        MobType::Koldun => "",
        MobType::ClayGolem => "golem.png",
        MobType::WaterElemental => "water_elemental.png",
        MobType::FireElemental => "fire_elemental.png",
        MobType::SkeletWarrior => "",
        MobType::SkeletMage => "",
        MobType::SkeletRanger => "",
        MobType::EarthElemental => "earth_elemental.png",
        MobType::AirElemental => "air_elemental.png",
        MobType::Thief => "lurker.png",
    }
    .to_string()
}
pub fn damage_mobs(
    mut commands: Commands,
    mut ev_play_audio: EventWriter<PlayAudioEvent>,
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

    on_hit_query: Query<&OnHitEffect>,
    on_death_effect: Query<&OnDeathEffect>,

    mut on_hit_event: EventWriter<OnHitEffectEvent>,
    mut on_death_event: EventWriter<OnDeathEffectEvent>,

    boss_query: Query<&BossAttackSystem>,

    mut global_transform_query: Query<&mut GlobalTransform, With<BusyOrbital>>,

    mut thief_query: Query<&mut PickupItemQueue>,
) {
    for (entity, mut health, _mob, transform, loot, mob_type) in mob_query.iter_mut() {
        let mut translation = transform.translation;

        if *mob_type == MobType::AirElemental && global_transform_query.contains(entity) {
            translation = global_transform_query
                .get_mut(entity)
                .unwrap()
                .translation();
        }

        if !health.hit_queue.is_empty() {
            let hit = health.hit_queue.remove(0);

            // наносим урон
            health.damage(hit.damage);
            ev_play_audio.send(PlayAudioEvent::from_file("mob_hit.ogg")); 
            
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
                    pos: translation,
                    dir: hit.direction,
                    vec_of_objects: vec_objects,
                    on_hit_effect_type: on_hit_eff,
                    is_friendly: false,
                });
            }

            // кидаем стан

            if !boss_query.contains(entity) {
                commands.entity(entity).insert(Stun::new(0.5));
            }

            // шлём ивент смерти
            if health.current <= 0 {
                if on_death_effect.contains(entity) {
                    let vec_objects;
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

                let mob_unlock_tag = mob_type_to_tag_convert(mob_type.clone());

                // события "поcле смерти"
                ev_death.send(MobDeathEvent {
                    mob_unlock_tag,
                    orbs: loot.orbs,
                    pos: translation,
                    dir: hit.direction,
                    is_spawned: false,
                });

                // спавним труп на месте смерти моба
                ev_corpse.send(CorpseSpawnEvent {
                    mob_type: mob_type.clone(),
                    pos: translation.with_z(0.05),
                });

                if *mob_type == MobType::AirElemental {
                    blank_spawn_ev.send(SpawnBlankEvent {
                        range: 8.,
                        position: translation,
                        speed: 10.,
                        is_friendly: false,
                    });
                }
                if STATIC_MOBS.contains(mob_type) {
                    let mob_pos = (
                        (translation.x.floor() / 32.).floor() as u16,
                        (translation.y.floor() / 32.).floor() as u16,
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

fn mob_death(
    mut portal_manager: ResMut<PortalManager>,
    player_experience: Res<PlayerExperience>,

    mut ev_spawn_portal: EventWriter<crate::level_completion::PortalEvent>,
    mut ev_spawn_orb: EventWriter<SpawnExpOrbEvent>,
    mut ev_spawn_particles: EventWriter<SpawnParticlesEvent>,

    mut ev_mob_death: EventReader<MobDeathEvent>,

    mut saves: ResMut<Assets<Save>>,
    save_handle: Res<SaveHandle>,
) {
    for ev in ev_mob_death.read() {
        portal_manager.set_pos(ev.pos);

        if !ev.is_spawned {
            portal_manager.pop_mob();
        }

        if !ev.is_spawned && portal_manager.no_mobs_on_level() {
            ev_spawn_portal.send(PortalEvent {
                pos: portal_manager.get_pos(),
            });
        }

        let save = saves.get_mut(save_handle.0.id()).unwrap();
        if !save.seen_mobs.contains(&ev.mob_unlock_tag) {
            save.seen_mobs.push((ev.mob_unlock_tag).to_string());
        }

        let orb_count = (ev.orbs + player_experience.orb_bonus) as i32;
        let half_count = (orb_count as f32 / 2.).round() as i32;

        let offset = PI / 12.;
        for i in (-orb_count / 2)..half_count {
            // считаем точки, куда будем выбрасывать частицы опыта
            let angle = ev.dir.y.atan2(ev.dir.x) + offset * i as f32;
            let direction = Vec2::from_angle(angle) * 8.0;
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

pub fn before_attack_delay(
    mut timer_query: Query<(Entity, &mut BeforeAttackDelay), Without<Stun>>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (timer_e, mut delay) in timer_query.iter_mut() {
        delay.timer.tick(time.delta());
        if delay.timer.just_finished() {
            commands.entity(timer_e).insert(Done::Success);
        }
    }
}

fn pos_pathfinder(
    mut pathfinder_query: Query<
        (
            Entity,
            &mut Pathfinder,
            &Transform,
            &mut crate::pathfinding::FriendRush,
        ),
        Without<Stun>,
    >,
    mut commands: Commands,
    target_query: Query<(Entity, &Transform), With<Friend>>,
    time: Res<Time>,
) {
    for (pathfinder_e, _pathfinder, transform, mut timer) in pathfinder_query.iter_mut() {
        if target_query.iter().len() == 0 {
            commands.entity(pathfinder_e).insert(Done::Success);
            return;
        }

        timer.timer.tick(time.delta());

        if timer.timer.just_finished() {
            commands.entity(pathfinder_e).insert(Done::Success);
            continue;
        }

        let sorted_targets: Vec<(Entity, &Transform)> = target_query
            .iter()
            .sort_by::<&Transform>(|item1, item2| {
                item1
                    .translation
                    .distance(transform.translation)
                    .total_cmp(&item2.translation.distance(transform.translation))
            })
            .collect();

        let (target_e, mut target) = sorted_targets[0];

        if target_e == pathfinder_e {
            if sorted_targets.iter().len() < 2 {
                commands.entity(pathfinder_e).insert(Done::Success);
                continue;
            }
            target = sorted_targets[1].1;
        }

        if transform.translation.distance(target.translation) <= 100. {
            commands.entity(pathfinder_e).insert(Done::Success);
        }
    }
}

pub fn on_death_effects_handler(
    mut ev_on_death: EventReader<OnDeathEffectEvent>,
    mut ev_spawn_projectile: EventWriter<SpawnProjectileEvent>,
) {
    for ev in ev_on_death.read() {
        match ev.on_death_effect_type {
            OnDeathEffect::CircleAttack => {
                for i in 0..ev.vec_of_objects.len() {
                    let color = Color::WHITE;
                    ev_spawn_projectile.send(SpawnProjectileEvent {
                        texture_path: "textures/rib.png".to_string(),
                        color,
                        translation: ev.pos,
                        angle: (PI * i as f32 * 2.) / ev.vec_of_objects.len() as f32,
                        collider_radius: 8.0,
                        speed: 100.,
                        damage: 20,
                        element: ElementType::Steam,
                        is_friendly: ev.is_friendly,
                        trajectory: crate::projectile::Trajectory::Straight,
                        can_go_through_walls: false,
                    });
                }
            }
        }
    }
}

fn convert_i32_to_item(pick: i32) -> ItemType {
    match pick {
        0 => ItemType::Amulet,
        1 => ItemType::Bacon,
        2 => ItemType::Heart,
        3 => ItemType::LizardTail,
        4 => ItemType::SpeedPotion,
        5 => ItemType::WispInAJar,
        6 => ItemType::WaterbendingScroll,
        7 => ItemType::Mineral,
        8 => ItemType::Glider,
        9 => ItemType::GhostInTheShell,
        10 => ItemType::VampireTooth,
        11 => ItemType::BloodGoblet,
        12 => ItemType::BlindRage,
        _ => {
            println!("Update function convert_i32_to_item, there's no such item");
            ItemType::Amulet
        }
    }
}

pub fn on_hit_effects(
    mut ev_on_hit: EventReader<OnHitEffectEvent>,
    mut ev_spawn_item: EventWriter<SpawnItemEvent>,
    mut ev_spawn_hp_tank: EventWriter<SpawnHealthTankEvent>,
    mut ev_spawn_exp_tank: EventWriter<SpawnExpTankEvent>,

    item_database: Res<Assets<ItemDatabase>>,
    handle: Res<ItemDatabaseHandle>,
) {
    for ev in ev_on_hit.read() {
        match ev.on_hit_effect_type {
            OnHitEffect::DropItemFromBag => {
                let mut is_item = false;
                let mut count = 2;
                let offset = PI / 12.;
                for i in ev.vec_of_objects.iter() {
                    let dir = ev.dir * Vec3::new(-1., -1., 0.);

                    let angle = dir.y.atan2(dir.x) + offset * count as f32;

                    count += 1;

                    let direction = Vec2::from_angle(angle) * 24.0;
                    let destination =
                        Vec3::new(ev.pos.x + direction.x, ev.pos.y + direction.y, ev.pos.z);

                    if is_item {
                        let item_type = convert_i32_to_item(*i);

                        let item_name: String = item_database.get(handle.0.id()).unwrap().items
                            [item_type as usize]["name"]
                            .as_str()
                            .unwrap()
                            .to_string();
                        let texture_name: String = item_database.get(handle.0.id()).unwrap().items
                            [item_type as usize]["texture_name"]
                            .as_str()
                            .unwrap()
                            .to_string();
                        let item_description: String =
                            item_database.get(handle.0.id()).unwrap().items[item_type as usize]
                                ["description"]
                                .as_str()
                                .unwrap()
                                .to_string();

                        let texture_path = format!("textures/items/{}", texture_name);

                        ev_spawn_item.send(SpawnItemEvent {
                            pos: destination,
                            item_type: item_type,
                            texture_path: texture_path,
                            item_name: item_name,
                            item_description: item_description,
                        });
                        is_item = false;
                        continue;
                    }
                    if *i == ItemPicked::Item as i32 {
                        is_item = true;
                        continue;
                    }

                    if *i == ItemPicked::HPTank as i32 {
                        ev_spawn_hp_tank.send(SpawnHealthTankEvent {
                            pos: destination,
                            hp: 20,
                        });
                        continue;
                    }

                    if *i == ItemPicked::EXPTank as i32 {
                        ev_spawn_exp_tank.send(SpawnExpTankEvent {
                            pos: destination,
                            orbs: 6,
                        });
                        continue;
                    }
                }
            }
        };
    }
}
