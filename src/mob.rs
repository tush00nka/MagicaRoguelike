//all things about mobs and their spawn/behaviour
use std::{f32::consts::PI, time::Duration};

use avian2d::prelude::*;
use bevy::prelude::*;
use rand::{thread_rng, Rng};

use crate::{
    animation::AnimationConfig,
    chapter::ChapterManager,
    elements::{ElementResistance, ElementType},
    exp_orb::SpawnExpOrbEvent,
    experience::PlayerExperience,
    gamemap::{Map, TileType, ROOM_SIZE},
    health::{Health, Hit},
    level_completion::{PortalEvent, PortalManager},
    pathfinding::{create_new_graph, Pathfinder},
    player::Player,
    projectile::{Friendly, Projectile, SpawnProjectileEvent},
    stun::Stun,
    GameLayer, GameState, TimeState,
};

pub struct MobPlugin;

impl Plugin for MobPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Map::default())
            .add_event::<MobDeathEvent>()
            .add_event::<CorpseSpawnEvent>()
            .add_event::<MobSpawnEvent>()
            .add_systems(
                OnEnter(GameState::Loading),
                spawn_mobs_location.after(create_new_graph),
            )
            .add_systems(
                OnEnter(GameState::Loading),
                first_spawn_mobs.after(spawn_mobs_location),
            )
            .add_systems(
                Update,
                (
                    damage_mobs,
                    mob_death,
                    spawn_corpse,
                    damage_obstacles::<Obstacle>,
                    hit_obstacles::<Obstacle>,
                    animate_mobs,
                    rotate_mobs,
                    corpse_collision,
                    handle_raising,
                    spawner_mob_spawn,
                    mob_shoot,
                    spawn_mob,
                    hit_projectiles,
                    teleport_mobs,
                )
                    .run_if(in_state(TimeState::Unpaused))
                    .run_if(in_state(GameState::InGame)),
            )
            .add_systems(
                FixedUpdate,
                move_mobs
                    .run_if(in_state(TimeState::Unpaused))
                    .run_if(in_state(GameState::InGame)),
            )
            .add_systems(
                OnEnter(TimeState::Paused),
                crate::utils::clear_velocity_for::<Mob>,
            );
    }
}
//Components and bundles
//If you want to add something (create new mob, or add new component), first of all, add bundle and components there (and check, maybe it exists already)
#[derive(Component, Clone)]
pub enum MobType {
    //add your mobtype here
    Knight,
    Mossling,
    FireMage,
    WaterMage,
    JungleTurret,
    Necromancer,
}

#[derive(Component, Clone)]
#[allow(dead_code)]
pub enum ProjectileType {
    // can use to create mobs with different types of projectiles
    Circle,  //spawn some projectiles around
    Missile, // like fireball
    Gatling, // a lot of small ones
}

#[derive(Component)]
pub enum MobTarget{
    Player,
    Corpse,
    HPTank,
    EXPTank,
    Noone,
}
#[derive(Component, Default)]
pub struct PlayerRush;
#[derive(Component, Default)]
pub struct CorpseRush;

//Entity for
#[derive(Component)]
pub struct RotationEntity;

#[derive(Bundle)]
pub struct PhysicalBundle {
    // physical bundle with all physical stats
    collider: Collider,
    axes: LockedAxes,
    gravity: GravityScale,
    collision_layers: CollisionLayers,
    linear_velocity: LinearVelocity,
}

#[derive(Bundle)]
pub struct MobBundle {
    //contains mob stats
    resistance: ElementResistance,
    mob_type: MobType,
    mob: Mob,
    loot: MobLoot,
    body_type: RigidBody,
    health: Health,
}
#[derive(Bundle)]
pub struct MeleeMobBundle {
    // can add smth else, like has phasing or smth idk
    path_finder: Pathfinder,
}
#[derive(Bundle)]
pub struct TurretBundle {
    shoot_ability: ShootAbility,
}

#[derive(Bundle)]
pub struct MageBundle {
    //bundle only for mages
    teleport_ability: Teleport,
    shoot_ability: ShootAbility,
}

#[derive(Bundle)]
pub struct SummoningBundle {
    summoning_ability: Summoning,
    //maybe add something like flag structure, which will check is that static object, who spawns or something else, so we can add portal for example
}

#[derive(Component)]
pub struct Teleport {
    //todo: change to just tuple? maybe not?
    pub amount_of_tiles: u8,
    pub place_to_teleport: Vec<(u16, u16)>,
    pub time_to_teleport: Timer,
}
//component to mob and structures who can spawn enemy.
#[derive(Component)]
pub struct Summoning {
    pub time_to_spawn: Timer,
    pub is_static: bool,
}

#[derive(Component)]
pub struct ShootAbility {
    pub time_to_shoot: Timer,
    element: ElementType,
    proj_type: ProjectileType,
}
#[derive(Component)]
pub struct Mob {
    //todo: Rename to contact damage or smth, or remove damage and save mob struct as flag
    pub damage: i32,
}
//Corpse component for necromancer.
#[derive(Component)]
pub struct Corpse {
    mob_type: MobType,
}

//struct for obstacles, which can be destroyed(post, corpses, smth)
#[derive(Component)]
pub struct Obstacle;

#[derive(Component)]
pub struct Raising {
    pub mob_type: MobType,
    pub mob_pos: Transform,
    pub corpse_id: Entity,
}

#[derive(Component)]
pub struct BusyRaising;
#[derive(Component)]
pub struct MobLoot {
    //todo: maybe add something like chance to spawn tank with exp/hp?
    // we can add an item for this
    pub orbs: u32,
}
//implemenations
//change it, only if you know what you're doing
impl Mob {
    fn new(damage: i32) -> Self {
        Self { damage }
    }
}

impl MeleeMobBundle {
    fn knight() -> Self {
        Self {
            path_finder: Pathfinder {
                path: vec![],
                update_path_timer: Timer::new(
                    Duration::from_millis(rand::thread_rng().gen_range(500..999)),
                    TimerMode::Repeating,
                ),
                speed: 2000.,
            },
        }
    }

    fn mossling() -> Self {
        Self {
            path_finder: Pathfinder {
                path: vec![],
                update_path_timer: Timer::new(
                    Duration::from_millis(rand::thread_rng().gen_range(500..999)),
                    TimerMode::Repeating,
                ),
                speed: 2500.,
            },
        }
    }

    fn necromancer() -> Self {
        Self {
            path_finder: Pathfinder {
                path: vec![],
                update_path_timer: Timer::new(
                    Duration::from_millis(rand::thread_rng().gen_range(500..999)),
                    TimerMode::Repeating,
                ),
                speed: 3500.,
            },
        }
    }
}

impl TurretBundle {
    fn jungle_turret() -> Self {
        let timer: u64 = rand::thread_rng().gen_range(1500..2000);
        Self {
            shoot_ability: ShootAbility {
                time_to_shoot: Timer::new(Duration::from_millis(timer), TimerMode::Repeating),
                element: ElementType::Earth,
                proj_type: ProjectileType::Gatling,
            },
        }
    }
}

impl MageBundle {
    fn fire_mage() -> Self {
        let timer: u64 = rand::thread_rng().gen_range(3000..5000);
        Self {
            teleport_ability: Teleport {
                amount_of_tiles: 4,
                place_to_teleport: vec![],
                time_to_teleport: Timer::new(Duration::from_millis(timer), TimerMode::Repeating),
            },
            shoot_ability: ShootAbility {
                time_to_shoot: Timer::new(Duration::from_millis(timer), TimerMode::Repeating),
                element: ElementType::Fire,
                proj_type: ProjectileType::Missile,
            },
        }
    }
    fn water_mage() -> Self {
        //maybe ice idk?
        let timer: u64 = rand::thread_rng().gen_range(3000..5000);
        Self {
            teleport_ability: Teleport {
                amount_of_tiles: 4,
                place_to_teleport: vec![],
                time_to_teleport: Timer::new(Duration::from_millis(timer), TimerMode::Repeating),
            },
            shoot_ability: ShootAbility {
                time_to_shoot: Timer::new(Duration::from_millis(timer), TimerMode::Repeating),
                element: ElementType::Water,
                proj_type: ProjectileType::Missile,
            },
        }
    }
}
impl SummoningBundle {
    fn necromancer() -> Self {
        Self {
            summoning_ability: Summoning {
                time_to_spawn: Timer::new(
                    Duration::from_millis(rand::thread_rng().gen_range(1000..2000)),
                    TimerMode::Repeating,
                ),
                is_static: false,
            },
        }
    }
}
impl MobBundle {
    fn knight() -> Self {
        Self {
            resistance: ElementResistance {
                elements: vec![],
                resistance_percent: vec![0, 0, 0, 0, 0],
            },
            mob_type: MobType::Mossling,
            mob: Mob::new(20),
            loot: MobLoot { orbs: 3 },
            body_type: RigidBody::Dynamic,
            health: Health::new(100),
        }
    }
    fn mossling() -> Self {
        Self {
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
    fn turret() -> Self {
        Self {
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
    fn fire_mage() -> Self {
        Self {
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

    fn water_mage() -> Self {
        Self {
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

    fn necromancer() -> Self {
        Self {
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

// range for enum of mobs todo: change to better spawn?
impl rand::distributions::Distribution<MobType> for rand::distributions::Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> MobType {
        match rng.gen_range(0..=5) {
            0 => MobType::Mossling,
            1 => MobType::Knight,
            2 => MobType::FireMage,
            3 => MobType::WaterMage,
            4 => MobType::JungleTurret,
            5 => MobType::Necromancer,
            _ => MobType::Mossling,
        }
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
#[derive(Event)]
pub struct MobSpawnEvent {
    pub mob_type: MobType,
    pub pos: (u16, u16),
}
//event to spawn corpse
#[derive(Event)]
pub struct CorpseSpawnEvent {
    pub pos: Vec3,
    pub mob_type: MobType,
}

fn spawn_mobs_location(mut mob_map: ResMut<Map>, chapter_manager: Res<ChapterManager>) {
    let chap_num = chapter_manager.get_current_chapter();
    let mut rng = thread_rng();
    let mut mobs_amount: u16 = rng.gen_range(1 + 5 * chap_num as u16..5 + 5 * chap_num as u16);
    let mut chance: f32;

    while mobs_amount > 0 {
        for x in 1..ROOM_SIZE - 1 {
            for y in 1..ROOM_SIZE - 1 {
                chance = ((x - 16).abs() + (y - 16).abs()) as f32 - 1.0; // |delta x| + |delta y| - distance from player

                if rng.gen::<f32>() < (chance / ROOM_SIZE as f32)
                    && mobs_amount != 0
                    && mob_map.map.get(&(x as u16, y as u16)).unwrap().tiletype == TileType::Floor
                    && mob_map.map.get(&(x as u16, y as u16)).unwrap().mob_count != i16::MAX
                {
                    mob_map
                        .map
                        .get_mut(&(x as u16, y as u16))
                        .unwrap()
                        .mob_count = i16::MAX;
                    mobs_amount -= 1;
                }
            }
        }
    }
}
pub fn spawn_mob(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut mob_map: ResMut<Map>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut ev_mob_spawn: EventReader<MobSpawnEvent>,
) {
    for ev in ev_mob_spawn.read() {
        let texture_path: &str;
        let frame_count: u32;
        let fps: u8;
        let rotation_entity: bool;
        let rotation_path: &str;
        let has_animation: bool;
        let pixel_size: u32;
        let target_for_melee: MobTarget;

        let x = ev.pos.0;
        let y = ev.pos.1;

        //pick mob with random, assign some variables
        match ev.mob_type {
            MobType::Knight => {
                frame_count = 4;
                fps = 12;
                texture_path = "textures/mobs/knight.png";
                rotation_path = "";
                rotation_entity = false;
                has_animation = true;
                pixel_size = 16;
                target_for_melee = MobTarget::Player;
            }
            MobType::Mossling => {
                frame_count = 4;
                fps = 12;
                texture_path = "textures/mobs/mossling.png";
                rotation_path = "";
                rotation_entity = false;
                has_animation = true;
                pixel_size = 16;
                target_for_melee = MobTarget::Player;
            }
            MobType::FireMage => {
                texture_path = "textures/mobs/fire_mage.png";
                rotation_path = "";
                frame_count = 2;
                fps = 3;
                rotation_entity = false;
                has_animation = true;
                pixel_size = 16;
                target_for_melee = MobTarget::Noone;
            }
            MobType::WaterMage => {
                frame_count = 2;
                fps = 3;
                texture_path = "textures/mobs/water_mage.png";
                rotation_path = "";
                rotation_entity = false;
                has_animation = true;
                pixel_size = 16;
                target_for_melee = MobTarget::Noone;
            }
            MobType::JungleTurret => {
                frame_count = 1;
                fps = 1;
                texture_path = "textures/mobs/plant_body.png";
                rotation_path = "textures/mobs/plant_head.png";
                rotation_entity = true;
                has_animation = false;
                pixel_size = 16;
                target_for_melee = MobTarget::Noone;
            }
            MobType::Necromancer => {
                frame_count = 4;
                fps = 12;
                texture_path = "textures/mobs/necromancer.png";
                rotation_path = "";
                rotation_entity = false;
                has_animation = true;
                pixel_size = 24;
                target_for_melee = MobTarget::Corpse;
            }
        }
        //get texture and layout
        let texture = asset_server.load(texture_path);
        let layout =
            TextureAtlasLayout::from_grid(UVec2::splat(pixel_size), frame_count, 1, None, None);
        let texture_atlas_layout = texture_atlas_layouts.add(layout);
        //setup animation cfg
        let animation_config = AnimationConfig::new(0, frame_count as usize - 1, fps);
        //spawn mob with texture
        let mob = commands
            .spawn(SpriteBundle {
                texture,
                transform: Transform::from_xyz(
                    (x as i32 * ROOM_SIZE) as f32,
                    (y as i32 * ROOM_SIZE) as f32,
                    1.0,
                ),
                ..default()
            })
            .id();

        commands.entity(mob).insert(PhysicalBundle::default());

        if has_animation {
            commands
                .entity(mob) //todo: change it that we could test mobs without animations
                .insert(TextureAtlas {
                    layout: texture_atlas_layout.clone(),
                    index: animation_config.first_sprite_index,
                })
                .insert(animation_config);
        }
        match ev.mob_type {
            MobType::Knight => {
                commands
                    .entity(mob)
                    .insert(MobBundle::knight())
                    .insert(MeleeMobBundle::knight());
            }
            MobType::Mossling => {
                commands
                    .entity(mob)
                    .insert(MobBundle::mossling())
                    .insert(MeleeMobBundle::mossling());
            }
            MobType::FireMage => {
                commands
                    .entity(mob)
                    .insert(MobBundle::fire_mage())
                    .insert(MageBundle::fire_mage());

                mob_map
                    .map
                    .get_mut(&(x as u16, y as u16))
                    .unwrap()
                    .mob_count += 1;
            }
            MobType::WaterMage => {
                commands
                    .entity(mob)
                    .insert(MobBundle::water_mage())
                    .insert(MageBundle::water_mage());

                mob_map
                    .map
                    .get_mut(&(x as u16, y as u16))
                    .unwrap()
                    .mob_count += 1;
            }
            MobType::JungleTurret => {
                commands
                    .entity(mob)
                    .insert(MobBundle::turret())
                    .insert(TurretBundle::jungle_turret());

                mob_map
                    .map
                    .get_mut(&(x as u16, y as u16))
                    .unwrap()
                    .mob_count += 1;
            }
            MobType::Necromancer => {
                commands
                    .entity(mob)
                    .insert(MobBundle::necromancer())
                    .insert(SummoningBundle::necromancer())
                    .insert(MeleeMobBundle::necromancer());
                //add necro bundles
            }
        }
        match target_for_melee{
            MobTarget::Player => {
                commands.entity(mob).insert(PlayerRush);
            }
            MobTarget::Corpse => {
                commands.entity(mob).insert(CorpseRush);
            }
            MobTarget::EXPTank => {}
            MobTarget::HPTank => {}
            MobTarget::Noone => {}
        }
        if rotation_entity {
            commands.entity(mob).with_children(|parent| {
                parent
                    .spawn(SpriteBundle {
                        texture: asset_server.load(rotation_path),
                        transform: Transform::from_xyz(0., 0., 1.0),
                        ..default()
                    })
                    .insert(RotationEntity);
            });
        }
    }
}

pub fn first_spawn_mobs(
    mut mob_map: ResMut<Map>,
    mut game_state: ResMut<NextState<GameState>>,
    mut ev_mob_spawn: EventWriter<MobSpawnEvent>,
    mut portal_manager: ResMut<PortalManager>,
) {
    for x in 1..ROOM_SIZE - 1 {
        for y in 1..ROOM_SIZE - 1 {
            if mob_map.map.get(&(x as u16, y as u16)).unwrap().mob_count == i16::MAX {
                mob_map
                    .map
                    .get_mut(&(x as u16, y as u16))
                    .unwrap()
                    .mob_count = 0;

                let mob_type: MobType = rand::random();
                portal_manager.push_mob();
                ev_mob_spawn.send(MobSpawnEvent {
                    mob_type: mob_type,
                    pos: (x as u16, y as u16),
                });
            }
        }
    }

    game_state.set(GameState::InGame);
}

fn teleport_mobs(mut mob_query: Query<(&mut Transform, &mut Teleport), Without<Stun>>) {
    for (mut transform, mut mob) in mob_query.iter_mut() {
        if mob.place_to_teleport.len() > 0 {
            transform.translation = Vec3::new(
                mob.place_to_teleport[0].0 as f32 * ROOM_SIZE as f32,
                mob.place_to_teleport[0].1 as f32 * ROOM_SIZE as f32,
                1.0,
            );
            mob.place_to_teleport.remove(0);
        }
    }
}

fn move_mobs(
    mut mob_query: Query<
        (&mut LinearVelocity, &Transform, &mut Pathfinder),
        (Without<Stun>, Without<Teleport>, Without<Raising>),
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
    mut mob_query: Query<(&Transform, &mut ShootAbility), Without<Stun>>,
    mut player_query: Query<&Transform, (With<Player>, Without<Mob>)>,
    time: Res<Time>,
) {
    for (&transform, mut can_shoot) in mob_query.iter_mut() {
        if let Ok(player) = player_query.get_single_mut() {
            can_shoot.time_to_shoot.tick(time.delta());
            if can_shoot.time_to_shoot.just_finished() {
                let dir = (player.translation.truncate() - transform.translation.truncate())
                    .normalize_or_zero();
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
//Function for moving spawner mobs
fn spawner_mob_spawn(
    mut commands: Commands,
    mut ev_spawn: EventWriter<MobSpawnEvent>,
    mut summoner_query: Query<(Entity, &mut Summoning, &Raising, &mut Sprite), Without<Stun>>,
    corpse_query: Query<Entity,(With<Corpse>, With<BusyRaising>)>,
    time: Res<Time>,
    mut portal_manager: ResMut<PortalManager>,
) {
    for (summoner, mut summoning, raising, mut sprite) in summoner_query.iter_mut() {
        if !corpse_query.contains(raising.corpse_id){
            commands.entity(summoner).remove::<Raising>();
            sprite.color = Color::srgb(1., 1., 1.);
            continue;
        }
        summoning.time_to_spawn.tick(time.delta());
        if summoning.time_to_spawn.just_finished() {
            let mob_pos: (u16, u16) = (
                (raising.mob_pos.translation.x / 32.).floor() as u16,
                (raising.mob_pos.translation.y / 32.).floor() as u16,
            );
            ev_spawn.send(MobSpawnEvent {
                mob_type: raising.mob_type.clone(),
                pos: mob_pos,
            });

            commands.entity(raising.corpse_id).despawn();
            commands.entity(summoner).remove::<Raising>();
            sprite.color = Color::srgb(1., 1., 1.);
            portal_manager.push_mob();
        }
    }
}

fn corpse_collision(
    mut commands: Commands,
    mut summoner_query: Query<
        (Entity, &Transform, &mut Summoning, &Pathfinder),
        (Without<Raising>, Without<Stun>),
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
                        commands.entity(spawner_e).insert(Raising {
                            mob_type: corpse.mob_type.clone(),
                            mob_pos: *transform,
                            corpse_id: corpse_e,
                        });
                        commands.entity(corpse_e).insert(BusyRaising);
                    }
                }
            }
        }
    }
}
fn hit_projectiles(
    mut commands: Commands,
    projectile_query: Query<(Entity, &Projectile, &Transform), With<Friendly>>,
    mut mob_query: Query<(Entity, &mut Health, &Mob, &Transform, &ElementResistance), With<Mob>>,
    mut ev_collision: EventReader<Collision>,
) {
    for Collision(contacts) in ev_collision.read() {
        let mut proj_e = Entity::PLACEHOLDER;
        let mut mob_e = Entity::PLACEHOLDER;

        if projectile_query.contains(contacts.entity2) && mob_query.contains(contacts.entity1) {
            proj_e = contacts.entity2;
            mob_e = contacts.entity1;
        } else if projectile_query.contains(contacts.entity1)
            && mob_query.contains(contacts.entity2)
        {
            proj_e = contacts.entity1;
            mob_e = contacts.entity2;
        }

        for (candidate_e, mut health, _mob, transform, resistance) in mob_query.iter_mut() {
            if mob_e == candidate_e {
                for (proj_candidate_e, projectile, projectile_transform) in projectile_query.iter()
                {
                    if proj_e == proj_candidate_e {
                        // считаем урон с учётом сопротивления к элементам
                        let mut damage = projectile.damage as i32;
                        resistance.calculate_for(&mut damage, Some(projectile.element));

                        // направление выстрела
                        let shot_dir =
                            (transform.translation - projectile_transform.translation).normalize();

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
    }
}

fn hit_obstacles<T: Component>(
    //TODO: ADD LOOT DROP FROM OBSTACLES IDK, MAYBE ADD LOOT TO THEM
    mut commands: Commands,
    projectile_query: Query<(Entity, &Projectile, &Transform), With<Friendly>>,
    mut mob_query: Query<(Entity, &mut Health, &Transform), With<T>>,
    mut ev_collision: EventReader<Collision>,
) {
    for Collision(contacts) in ev_collision.read() {
        let mut proj_e = Entity::PLACEHOLDER;
        let mut obstacle_e = Entity::PLACEHOLDER;

        if projectile_query.contains(contacts.entity2) && mob_query.contains(contacts.entity1) {
            proj_e = contacts.entity2;
            obstacle_e = contacts.entity1;
        } else if projectile_query.contains(contacts.entity1)
            && mob_query.contains(contacts.entity2)
        {
            proj_e = contacts.entity1;
            obstacle_e = contacts.entity2;
        }

        for (candidate_e, mut health, transform) in mob_query.iter_mut() {
            if obstacle_e == candidate_e {
                for (proj_candidate_e, projectile, projectile_transform) in projectile_query.iter()
                {
                    if proj_e == proj_candidate_e {
                        // считаем урон с учётом сопротивления к элементам
                        let damage = projectile.damage as i32;

                        // направление выстрела
                        let shot_dir =
                            (transform.translation - projectile_transform.translation).normalize();

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
    }
}

fn handle_raising(
    mut raising_query: Query<(&mut Sprite, &mut LinearVelocity), Changed<Raising>>,
) {
    for (mut sprite, mut linvel) in raising_query.iter_mut() {
        sprite.color = Color::srgb(1., 3., 3.);
        linvel.0 = Vec2::ZERO;
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

            // шлём ивент смерти
            if health.current <= 0 {
                // деспавним сразу
                commands.entity(entity).despawn_recursive();
                //TODO: ADD LOOT SPAWN
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

                // события "поле смерти"
                ev_death.send(MobDeathEvent {
                    orbs: loot.orbs,
                    pos: transform.translation,
                    dir: hit.direction,
                });
                // спавним труп на месте смерти моба
                ev_corpse.send(CorpseSpawnEvent {
                    mob_type: mob_type.clone(),
                    pos: transform.translation,
                });
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
                texture_path = "textures/mob_corpse_placeholder.png";
                can_be_spawned = true;
            }
            MobType::Mossling => {
                texture_path = "textures/mob_corpse_placeholder.png";
                can_be_spawned = true;
            }
            MobType::FireMage => {
                texture_path = "textures/mob_corpse_placeholder.png";
                can_be_spawned = true;
            }
            MobType::WaterMage => {
                texture_path = "textures/mob_corpse_placeholder.png";
                can_be_spawned = true;
            }
            MobType::JungleTurret => {
                texture_path = "textures/mob_corpse_placeholder.png";
                can_be_spawned = true;
            }
            MobType::Necromancer => {
                texture_path = "textures/mob_corpse_placeholder.png";
                can_be_spawned = false;
            }
        }
        let texture = asset_server.load(texture_path);
        let grave = commands
            .spawn(SpriteBundle {
                texture,
                transform: Transform::from_xyz(ev.pos.x, ev.pos.y, ev.pos.z),
                ..default()
            })
            .insert(Collider::circle(8.))
            .insert(Sensor)
            .insert(LockedAxes::ROTATION_LOCKED)
            .insert(GravityScale(0.0))
            .insert(CollisionLayers::new(
                GameLayer::Enemy,
                [GameLayer::Projectile, GameLayer::Enemy],
            ))
            .insert(RigidBody::Dynamic)
            .insert(Health::new(40))
            .insert(Obstacle)
            .id();
        if can_be_spawned {
            commands.entity(grave).insert(Corpse {
                mob_type: ev.mob_type.clone(),
            });
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

fn animate_mobs(
    time: Res<Time>,
    mut query: Query<(&mut AnimationConfig, &mut TextureAtlas), (With<Mob>, Without<Stun>)>,
) {
    for (mut config, mut atlas) in &mut query {
        // we track how long the current sprite has been displayed for
        config.frame_timer.tick(time.delta());

        // If it has been displayed for the user-defined amount of time (fps)...
        if config.frame_timer.just_finished() {
            if atlas.index == config.last_sprite_index {
                // ...and it IS the last frame, then we move back to the first frame and stop.
                atlas.index = config.first_sprite_index;
            } else {
                // ...and it is NOT the last frame, then we move to the next frame...
                atlas.index += 1;
                // ...and reset the frame timer to start counting all over again
                config.frame_timer = AnimationConfig::timer_from_fps(config.fps);
            }
        }
    }
}

fn rotate_mobs(
    player_query: Query<&Transform, (With<Player>, Without<RotationEntity>)>,
    mut rotation_query: Query<
        (&GlobalTransform, &mut Transform),
        (With<RotationEntity>, Without<Player>, Without<Stun>),
    >,
    time: Res<Time>,
) {
    for (global_rotation, mut rotation_en) in &mut rotation_query {
        if let Ok(player_transform) = player_query.get_single() {
            let translation = global_rotation.translation();
            let diff = Vec3::new(
                player_transform.translation.x,
                player_transform.translation.y,
                translation.z,
            ) - translation;
            let angle = diff.y.atan2(diff.x);
            rotation_en.rotation = rotation_en
                .rotation
                .lerp(Quat::from_rotation_z(angle), 12.0 * time.delta_seconds());
        }
    }
}
