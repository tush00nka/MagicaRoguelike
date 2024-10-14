//all things about mobs and their spawn/behaviour
use std::{f32::consts::PI, time::Duration};

use avian2d::prelude::*;
use bevy::prelude::*;
use rand::Rng;

use crate::{
    animation::AnimationConfig,
    elements::ElementType,
    exp_orb::SpawnExpOrbEvent,
    experience::PlayerExperience,
    gamemap::{LevelGenerator, Map, TileType, ROOM_SIZE},
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
        app.insert_resource(Map::default())
            .add_event::<MobDeathEvent>()
            .add_systems(OnEnter(GameState::InGame), spawn_mobs)
            .add_systems(
                FixedUpdate,
                (
                    move_mobs,
                    hit_projectiles,
                    mob_death,
                    animate_mobs,
                    teleport_mobs,
                    mob_shoot,
                )
                    .run_if(in_state(GameState::InGame)),
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
}

#[derive(Component, Clone)]
#[allow(dead_code)]
pub enum ProjectileType {
    // can use to create mobs with different types of projectiles
    Circle,  //spawn some projectiles around
    Missile, // like fireball
    Gatling, // a lot of small ones
}

#[derive(Bundle)]
pub struct PhysicalBundle {
    // physical bundle with all physical stats
    collider: Collider,
    axes: LockedAxes,
    gravity: GravityScale,
    collision_layers: CollisionLayers,
    linear_velocity: LinearVelocity,
}
#[derive(Component)]//todo: change res percent to vector too, that there can be different values
pub struct ElementResistance {//resistance component, applies any amount of elementres to entity 
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
}

#[derive(Component)]
pub struct MobLoot {
    //todo: maybe add something like chance to spawn tank with exp/hp?
    // we can add an item for this
    pub orbs: u32,
}
//implemenations
//change it, only if you're know what you're doing
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
            mob: Mob { damage: 20 },
            loot: MobLoot { orbs: 3 },
            body_type: RigidBody::Dynamic,
            health: Health {
                max: 100,
                current: 100,
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
            mob: Mob { damage: 20 },
            loot: MobLoot { orbs: 3 },
            body_type: RigidBody::Kinematic,
            health: Health {
                max: 80,
                current: 80,
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
            //        2 => MobType::WaterMage,
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

pub fn spawn_mobs(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    room: Res<LevelGenerator>,
    mut mob_map: ResMut<Map>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let grid = room.grid.clone();
    for i in 1..grid.len() - 1 {
        for j in 1..grid[i].len() - 1 {
            if grid[i][j] == TileType::Floor {
                let mut rng = rand::thread_rng();
                //need to fix 0 mob levels
                if rng.gen::<f32>() > 0.9 && (i > 18 || i < 14) && (j > 18 || j < 14) {
                    let mob_type: MobType = rand::random();
                    let texture_path: &str;
                    let frame_count: u32;
                    let fps: u8;
                    //pick mob with random, assign some variables
                    match mob_type {
                        MobType::Mossling => {
                            frame_count = 4;
                            fps = 12;
                            texture_path = "textures/mobs/mossling.png";
                        }
                        MobType::FireMage => {
                            texture_path = "textures/mobs/fire_mage.png";
                            frame_count = 2;
                            fps = 3;
                        }
                        MobType::WaterMage => {
                            frame_count = 2;
                            fps = 3;
                            texture_path = "textures/player_placeholder.png";
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
                                (i as i32 * ROOM_SIZE) as f32,
                                (j as i32 * ROOM_SIZE) as f32,
                                1.0,
                            ),
                            ..default()
                        })
                        .id();

                    commands.entity(mob).insert(PhysicalBundle::default());

                    commands
                        .entity(mob) //todo: change it that we could test mobs without animations
                        .insert(TextureAtlas {
                            layout: texture_atlas_layout.clone(),
                            index: animation_config.first_sprite_index,
                        })
                        .insert(animation_config);

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
                                .get_mut(&(i as u16, j as u16))
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
                                .get_mut(&(i as u16, j as u16))
                                .unwrap()
                                .mob_count += 1;
                        }
                    }
                }
            }
        }
    }
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
    mut mob_query: Query<(&Transform, &mut ShootAbility)>,
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

                match can_shoot.proj_type {
                    //todo: change this fragment, that we could spawn small and circle projs, maybe change event?
                    ProjectileType::Circle => texture_path = "textures/earthquake.png".to_string(),
                    ProjectileType::Missile => texture_path = "textures/fireball.png".to_string(),
                    ProjectileType::Gatling => texture_path = "textures/small_fire.png".to_string(),
                }
                let color = {
                    //todo: put it into function in element.rs (same code)
                    match can_shoot.element {
                        ElementType::Fire => Color::srgb(2.5, 1.25, 1.0),
                        ElementType::Water => Color::srgb(1.0, 1.5, 2.5),
                        ElementType::Earth => Color::srgb(2.5, 1.25, 1.25),
                        ElementType::Air => Color::srgb(1.5, 2.0, 1.5),
                        ElementType::Steam => Color::srgb(1.5, 2.0, 1.5),
                    }
                };

                ev_shoot.send(SpawnProjectileEvent {
                    texture_path: texture_path,
                    color: color, //todo: change this fragment, that we could spawn different types of projectiles.
                    translation: transform.translation,
                    angle: angle,
                    radius: 8.0,
                    speed: 150.,
                    damage: 20,
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
    mut mob_query: Query<(Entity, &mut Health, &Transform, &MobLoot, &ElementResistance), With<Mob>>,
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

        for (candidate_e, mut health, transform, loot,resistance) in mob_query.iter_mut() {
            if mob_e.is_some() && mob_e.unwrap() == candidate_e {
                for (proj_candidate_e, projectile, projectile_transform) in projectile_query.iter()
                {
                    if proj_e.is_some() && proj_e.unwrap() == proj_candidate_e {
                        let mut damage_with_res:i32 = projectile.damage.try_into().unwrap();
                        if resistance.elements.contains(&projectile.element){
                            damage_with_res = (damage_with_res as f32 * (1. - resistance.resistance_percent as f32 / 100.)) as i32;
                            print!("damage with res is - {}", damage_with_res);
                        }

                        health.damage(damage_with_res);

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
