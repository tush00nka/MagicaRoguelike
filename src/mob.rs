//all things about mobs and their spawn/behaviour
use std::{f32::consts::PI, time::Duration};

use avian2d::prelude::*;
use bevy::prelude::*;
use rand::Rng;

use crate::{
    animation::AnimationConfig,
    exp_orb::SpawnExpOrbEvent,
    experience::PlayerExperience, 
    gamemap::{
        LevelGenerator,
        Map,
        TileType,
        ROOM_SIZE
    },
    health::Health,
    level_completion::{
        PortalEvent,
        PortalManager
    },
    pathfinding::Pathfinder,
    player::Player,
    projectile::{
        Friendly,
        Projectile,
        SpawnProjectileEvent
    },
    stun::Stun,
    GameLayer,
    GameState
};

pub struct MobPlugin;

impl Plugin for MobPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(Map::default())
            .add_event::<MobDeathEvent>()
            .add_systems(OnEnter(GameState::InGame), spawn_mobs)
            .add_systems(
                FixedUpdate,
                (move_mobs, hit_projectiles, teleport_mobs, mob_shoot).run_if(in_state(GameState::InGame)),
            )
            .add_systems(Update, (animate_mobs, mob_death).run_if(in_state(GameState::InGame)));
    }
}
pub enum MobType {
    Mossling,
    Teleport,
}

#[derive(Component)]
pub struct Teleport {
    pub amount_of_tiles: u8,
}

#[derive(Component)]
pub struct ShootAbility{
    pub time_to_shoot: Timer
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

pub fn spawn_mobs(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    room: Res<LevelGenerator>,
    mut mob_map: ResMut<Map>,
) {
    let grid = room.grid.clone();
    for i in 1..grid.len() - 1 {
        for j in 1..grid[i].len() - 1 {
            if grid[i][j] == TileType::Floor {
                let mut rng = rand::thread_rng();
                //need to fix 0 mob levels
                if rng.gen::<f32>() > 0.9 && (i > 18 || i < 14) && (j > 18 || j < 14){ // make sure emenies don't spawn too close to player (todo: rewrite)
                    
                    let mob_type: MobType = rand::random();
                    let texture_path: &str;
                    let mut can_shoot: bool = false;
                    let mut has_teleport: bool = false;
                    let mut amount_of_tiles: u8 = 0;
                    let timer: std::ops::Range<u64>;

                    let frame_count: u32;
                    let fps: u8;

                    match mob_type {
                        MobType::Mossling => {
                            texture_path = "textures/mobs/mossling.png";
                            timer = 500..999;

                            frame_count = 4;
                            fps = 12;
                        }
                        MobType::Teleport => {
                            timer = 3000..5000;
                            amount_of_tiles = 4;
                            has_teleport = true;
                            can_shoot = true;
                            texture_path = "textures/mobs/fire_mage.png";

                            frame_count = 2;
                            fps = 3;
                        }
                    }
                  
                    let texture = asset_server.load(texture_path);

                    let layout = TextureAtlasLayout::from_grid(UVec2::splat(16), frame_count, 1, None, None);
                    let texture_atlas_layout = texture_atlas_layouts.add(layout);
                
                    let animation_config = AnimationConfig::new(0, frame_count as usize - 1, fps);

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

                    commands.entity(mob)
                        .insert(
                            TextureAtlas {
                                layout: texture_atlas_layout.clone(),
                                index: animation_config.first_sprite_index,
                            }
                        )
                        .insert(animation_config);

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
                        .insert(Mob { 
                            damage: 20
                         })
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
                        commands.entity(mob).insert(Teleport { amount_of_tiles }).insert(RigidBody::Kinematic);
                        mob_map.map.get_mut(&(i as u16,j as u16)).unwrap().mob_count += 1;
                    } else {
                        commands.entity(mob).insert(RigidBody::Dynamic);
                    }
                    if can_shoot{
                        commands.entity(mob).insert(ShootAbility{time_to_shoot: Timer::new(
                            Duration::from_millis(rand::thread_rng().gen_range(timer)),
                            TimerMode::Repeating)});
                    }
                }
            }
        }
    }
}

fn teleport_mobs(mut mob_query: Query<(&mut Transform, &mut Pathfinder), (Without<Stun>, With<Teleport>)>) {
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

fn move_mobs(mut mob_query: Query<(&mut LinearVelocity, &Transform, &mut Pathfinder), (Without<Stun>, Without<Teleport>)>, time: Res<Time>) {
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

fn mob_shoot(
    mut ev_shoot: EventWriter<SpawnProjectileEvent>,
    mut mob_query: Query<(&Transform, &mut ShootAbility)>,
    mut player_query: Query<&Transform, (With<Player>, Without<Mob>)>,
    time: Res<Time>,
){
    for (transform, mut can_shoot) in mob_query.iter_mut(){
        if let Ok(player) = player_query.get_single_mut() {
            can_shoot.time_to_shoot.tick(time.delta());
            if can_shoot.time_to_shoot.just_finished() {
                println!("Start to cast");
                let dir = (player.translation.truncate() - transform.translation.truncate()).normalize_or_zero();
                let angle = dir.y.atan2(dir.x);
                ev_shoot.send(
                    SpawnProjectileEvent { 
                        texture_path: "textures/fireball.png".to_string(), 
                        color:  Color::srgb(2.5, 1.25, 1.0), 
                        translation: transform.translation, 
                        angle: angle, 
                        radius: 8.0, 
                        speed: 150., 
                        damage: 20, 
                        element: crate::elements::ElementType::Fire, 
                        is_friendly: false
                    }
                );
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
            ev_spawn_portal.send( PortalEvent{pos: portal_manager.get_pos()});
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