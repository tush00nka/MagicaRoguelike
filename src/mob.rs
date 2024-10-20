//all things about mobs and their spawn/behaviour
use std::{f32::consts::PI, time::Duration};

use avian2d::prelude::*;
use bevy::prelude::*;
use rand::{thread_rng, Rng};

use crate::{
    animation::AnimationConfig,
    chapter::ChapterManager,
    elements::ElementType,
    exp_orb::SpawnExpOrbEvent,
    experience::PlayerExperience,
    gamemap::{Map, TileType, ROOM_SIZE},
    health::Health,
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
            .add_systems(
                OnEnter(GameState::Loading),
                spawn_mobs_location.after(create_new_graph),
            )
            .add_systems(
                OnEnter(GameState::Loading),
                spawn_mobs.after(spawn_mobs_location),
            )
            .add_systems(
                Update,
                (
                    damage_mobs,
                    mob_death,
                    animate_mobs,
                    rotate_mobs,
                    mob_shoot,
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
#[derive(Component)]
pub enum MobType {
    //add your mobtype here
    Mossling,
    FireMage,
    WaterMage,
    JungleTurret,
}

#[derive(Component, Clone)]
#[allow(dead_code)]
pub enum ProjectileType {
    // can use to create mobs with different types of projectiles
    Circle,  //spawn some projectiles around
    Missile, // like fireball
    Gatling, // a lot of small ones
}

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
#[derive(Component)] //todo: change res percent to vector too, that there can be different values
pub struct ElementResistance {
    //resistance component, applies any amount of elementres to entity
    elements: Vec<ElementType>,
    resistance_percent: i16,
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

#[derive(Component)]
pub struct Teleport {
    //todo: change to just tuple? maybe not?
    pub amount_of_tiles: u8,
    pub place_to_teleport: Vec<(u16, u16)>,
    pub time_to_teleport: Timer,
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
    pub hit_queue: Vec<(i32, Vec3)>,
}

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
        Self {
            damage,
            hit_queue: vec![],
        }
    }
}
impl MeleeMobBundle {
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
}

impl TurretBundle {
    fn jungle_turret() -> Self {
        let timer: u64 = rand::thread_rng().gen_range(500..999);
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

impl MobBundle {
    fn mossling() -> Self {
        Self {
            resistance: ElementResistance {
                elements: vec![ElementType::Earth],
                resistance_percent: 15,
            },
            mob_type: MobType::Mossling,
            mob: Mob::new(20),
            loot: MobLoot { orbs: 3 },
            body_type: RigidBody::Dynamic,
            health: Health {
                max: 100,
                current: 100,
                extra_lives: 0,
            },
        }
    }
    fn turret() -> Self {
        Self {
            resistance: ElementResistance {
                elements: vec![ElementType::Earth, ElementType::Water],
                resistance_percent: 60,
            },
            mob_type: MobType::JungleTurret,
            mob: Mob::new(20),
            loot: MobLoot { orbs: 3 },
            body_type: RigidBody::Kinematic,
            health: Health {
                max: 200,
                current: 200,
                extra_lives: 0,
            },
        }
    }
    fn fire_mage() -> Self {
        Self {
            resistance: ElementResistance {
                elements: vec![ElementType::Fire],
                resistance_percent: 80,
            },
            mob_type: MobType::FireMage,
            mob: Mob::new(20),
            loot: MobLoot { orbs: 3 },
            body_type: RigidBody::Kinematic,
            health: Health {
                max: 80,
                current: 80,
                extra_lives: 0,
            },
        }
    }

    fn water_mage() -> Self {
        Self {
            resistance: ElementResistance {
                elements: vec![ElementType::Water],
                resistance_percent: 80,
            },
            mob_type: MobType::WaterMage,
            mob: Mob::new(20),
            loot: MobLoot { orbs: 3 },
            body_type: RigidBody::Kinematic,
            health: Health {
                max: 80,
                current: 80,
                extra_lives: 0,
            },
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
        match rng.gen_range(0..=4) {
            0 => MobType::Mossling,
            1 => MobType::FireMage,
            2 => MobType::WaterMage,
            3 => MobType::JungleTurret,
            _ => MobType::Mossling,
        }
    }
}

#[derive(Event)]
pub struct MobDeathEvent {
    pub orbs: u32,
    pub pos: Vec3,
    pub dir: Vec3,
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

pub fn spawn_mobs(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut mob_map: ResMut<Map>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut game_state: ResMut<NextState<GameState>>,
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
                let texture_path: &str;
                let frame_count: u32;
                let fps: u8;
                let rotation_entity: bool;
                let rotation_path: &str;
                let has_animation: bool;
                //pick mob with random, assign some variables
                match mob_type {
                    MobType::Mossling => {
                        frame_count = 4;
                        fps = 12;
                        texture_path = "textures/mobs/mossling.png";
                        rotation_path = "";
                        rotation_entity = false;
                        has_animation = true;
                    }
                    MobType::FireMage => {
                        texture_path = "textures/mobs/fire_mage.png";
                        rotation_path = "";
                        frame_count = 2;
                        fps = 3;
                        rotation_entity = false;
                        has_animation = true;
                    }
                    MobType::WaterMage => {
                        frame_count = 2;
                        fps = 3;
                        texture_path = "textures/mobs/water_mage.png";
                        rotation_path = "";
                        rotation_entity = false;
                        has_animation = true;
                    }
                    MobType::JungleTurret => {
                        frame_count = 1;
                        fps = 1;
                        texture_path = "textures/mobs/plant_body.png";
                        rotation_path = "textures/mobs/plant_head.png";
                        rotation_entity = true;
                        has_animation = false;
                    }
                }
                //get texture and layout
                let texture = asset_server.load(texture_path);

                let layout =
                    TextureAtlasLayout::from_grid(UVec2::splat(16), frame_count, 1, None, None);
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
                match mob_type {
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
                    damage: damage,
                    element: can_shoot.element,
                    is_friendly: false,
                });
            }
        }
    }
}

fn hit_projectiles(
    //todo: change that we could use resistance mechanics
    mut commands: Commands,
    projectile_query: Query<(Entity, &Projectile, &Transform), With<Friendly>>,
    mut mob_query: Query<(Entity, &mut Mob, &Transform, &ElementResistance)>,
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

        for (candidate_e, mut mob, transform, resistance) in mob_query.iter_mut() {
            if mob_e == candidate_e {
                for (proj_candidate_e, projectile, projectile_transform) in projectile_query.iter()
                {
                    if proj_e == proj_candidate_e {
                        // считаем урон с учётом сопротивления к элементам
                        let mut damage_with_res: i32 = projectile.damage.try_into().unwrap();
                        if resistance.elements.contains(&projectile.element) {
                            damage_with_res = (damage_with_res as f32
                                * (1. - resistance.resistance_percent as f32 / 100.))
                                as i32;
                            // print!("damage with res is - {}", damage_with_res);
                        }

                        // направление выстрела
                        let shot_dir =
                            (transform.translation - projectile_transform.translation).normalize();

                        // пушим в очередь попадание
                        mob.hit_queue.push((damage_with_res, shot_dir));

                        // деспавним снаряд
                        commands.entity(proj_e).despawn();
                    }
                }
            }
        }
    }
}

fn damage_mobs(
    mut commands: Commands,
    mut ev_death: EventWriter<MobDeathEvent>,
    mut mob_query: Query<(Entity, &mut Health, &mut Mob, &Transform, &MobLoot), Changed<Mob>>,
) {
    for (entity, mut health, mut mob, transform, loot) in mob_query.iter_mut() {
        if !mob.hit_queue.is_empty() {
            let hit = mob.hit_queue.remove(0);

            // наносим урон
            health.damage(hit.0);

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
                    dir: hit.1,
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
            let (_, _, translation) = global_rotation.to_scale_rotation_translation();
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
