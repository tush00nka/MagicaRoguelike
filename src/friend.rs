//all things about mobs and their spawn/behaviour
///add mobs with kinematic body type
#[allow(unused)]
use crate::{
    animation::AnimationConfig,
    elements::{ElementResistance, ElementType},
    exp_orb::SpawnExpOrbEvent,
    experience::PlayerExperience,
    gamemap::Map,
    health::{Health, Hit},
    level_completion::{PortalEvent, PortalManager},
    mobs::{FlipEntity, MobDeathEvent, PhysicalBundle, RotationEntity, Mob, MobType, MobLoot, STATIC_MOBS},
    obstacles::{Corpse, CorpseSpawnEvent},
    player::Player,
    projectile::{Friendly, Projectile, SpawnProjectileEvent},
    stun::Stun,
    GameLayer, GameState,
};

use avian2d::prelude::*;
use bevy::prelude::*;

pub struct FriendPlugin;

impl Plugin for FriendPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<FriendSpawnEvent>();
        app.add_systems(
            Update,
            (friend_spawn, friend_damage_mob, damage_friends).run_if(in_state(GameState::InGame)),
        );
    }
}
#[allow(dead_code)]
#[derive(Component)]
pub enum FriendType {
    ClayGolem,
    Zombie,
}

#[derive(Component, Default)]
pub struct Friend;
///maybe add contact damage or add some melee attacks?

#[derive(Bundle)]
pub struct FriendBundle {
    phys_bundle: PhysicalBundle,
    resistance: ElementResistance,
    friend_type: FriendType,
    friend: Friend,
    body_type: RigidBody,
    health: Health,
}
#[allow(dead_code)]
pub struct SpawnFriendKit<'a> {
    texture_path: &'a str,
    frame_count: u32,
    fps: u8,
    rotation_entity: bool,
    rotation_path: &'a str,
    can_flip: bool,
    has_animation: bool,
    pixel_size: u32,
    can_move: bool,
}

impl<'a> Default for SpawnFriendKit<'a> {
    fn default() -> Self {
        Self {
            texture_path: "",
            frame_count: 4,
            fps: 12,
            rotation_entity: false,
            rotation_path: "",
            can_flip: false,
            has_animation: true,
            pixel_size: 16,
            can_move: true,
        }
    }
}

impl<'a> SpawnFriendKit<'a> {
    fn clay_golem() -> Self {
        Self {
            texture_path: "",
            ..default()
        }
    }

    fn zombie(str: &'a str) -> Self {
        Self {
            texture_path: str,
            ..default()
        }
    }
}

impl Default for FriendBundle {
    fn default() -> Self {
        Self {
            phys_bundle: PhysicalBundle {
                collision_layers: CollisionLayers::new(
                    GameLayer::Friend,
                    [
                        GameLayer::Wall,
                        GameLayer::Friend,
                        GameLayer::Projectile,
                        GameLayer::Shield,
                        GameLayer::Enemy,
                        GameLayer::Player,
                    ],
                ),
                ..default()
            },
            resistance: ElementResistance {
                elements: vec![ElementType::Earth],
                resistance_percent: vec![0, 0, 15, 0, 0],
            },
            friend_type: FriendType::Zombie,
            friend: Friend::default(),
            body_type: RigidBody::Dynamic,
            health: Health::new(1),
        }
    }
}

#[derive(Event)]
pub struct FriendSpawnEvent {
    pub friend_type: FriendType,
    pub pos: Vec2,
}

///спавн именно особых энтити, не поднятие дохлых, дохлых поднимать можно через mob_spawn
fn friend_spawn(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut ev_friend_spawn: EventReader<FriendSpawnEvent>,
) {
    for ev in ev_friend_spawn.read() {
        let spawn_kit: SpawnFriendKit;

        let x = ev.pos.x;
        let y = ev.pos.y;

        //pick mob with random, assign some variables
        match ev.friend_type {
            FriendType::ClayGolem => {
                spawn_kit = SpawnFriendKit::clay_golem();
            }
            FriendType::Zombie => {
                spawn_kit = SpawnFriendKit::zombie("textures/mobs/fire_mage.png");
            }
        }

        //get texture and layout
        let texture = asset_server.load(spawn_kit.texture_path);
        let layout = TextureAtlasLayout::from_grid(
            UVec2::splat(spawn_kit.pixel_size),
            spawn_kit.frame_count,
            1,
            None,
            None,
        );
        let texture_atlas_layout = texture_atlas_layouts.add(layout);
        //setup animation cfg
        let animation_config =
            AnimationConfig::new(0, spawn_kit.frame_count as usize - 1, spawn_kit.fps);
        //spawn mob with texture
        let mob = commands
            .spawn(SpriteBundle {
                texture,
                transform: Transform::from_xyz(x, y, 1.0),
                ..default()
            })
            .id();

        if spawn_kit.has_animation {
            commands
                .entity(mob) //todo: change it that we could test mobs without animations
                .insert(TextureAtlas {
                    layout: texture_atlas_layout.clone(),
                    index: animation_config.first_sprite_index,
                })
                .insert(animation_config);
        }
        if spawn_kit.can_flip {
            commands.entity(mob).insert(FlipEntity);
        }
        match ev.friend_type {
            FriendType::ClayGolem => {
                commands
                    .entity(mob)
                    .insert(crate::mobs::MeleeMobBundle::knight());
                commands
                    .entity(mob)
                    .insert(crate::mobs::SearchAndPursue::default());
                commands.entity(mob).insert(crate::mobs::Idle);
                commands
                    .entity(mob)
                    .insert(RayCaster::default())
                    .insert(Friend::default());
            }
            FriendType::Zombie => {
                commands
                    .entity(mob)
                    .insert(crate::mobs::MageBundle::fire_mage());
                //commands
                //    .entity(mob)
                //     .insert(crate::mobs::::default());
                //commands.entity(mob).insert(crate::mobs::Idle);
                commands
                    .entity(mob)
                    .insert(RayCaster::default())
                    .insert(Friend::default());
            }
        }

        if spawn_kit.rotation_entity {
            commands.entity(mob).with_children(|parent| {
                parent
                    .spawn(SpriteBundle {
                        texture: asset_server.load(spawn_kit.rotation_path),
                        transform: Transform::from_xyz(0., 0., 1.0),
                        ..default()
                    })
                    .insert(RotationEntity);
            });
        }
    }
}

fn friend_damage_mob(
    mut friend_query: Query<(&CollidingEntities,
        &mut Health, &Mob),
        (With<Friend>, Without<Player>, With<Mob>),
    >,
    mut mob_query: Query<(Entity,&mut Health, &Mob),(With<Mob>, Without<Friend>)>
) {
    for (friend_e, mut health_f, mob_f) in friend_query.iter_mut() {
        for (mob_e, mut health_m, mob_m) in mob_query.iter_mut() {
            if friend_e.contains(&mob_e) {
                health_f.hit_queue.push( Hit {
                    damage: mob_m.damage as i32,
                    element: Some(ElementType::Earth),
                    direction: Vec3::ZERO,
                });
            
                health_m.hit_queue.push( Hit {
                    damage: mob_f.damage as i32,
                    element: Some(ElementType::Earth),
                    direction: Vec3::ZERO,
                });
            }
        }
    }
}

pub fn damage_friends(
    mut commands: Commands,
    mut ev_corpse: EventWriter<CorpseSpawnEvent>,
    mut mob_query: Query<
        (
            Entity,
            &mut Health,
            &mut Mob,
            &Transform,
            &MobType,
        ),
        (With<Mob>, With<Friend>)
    >,
    mut mob_map: ResMut<Map>,
) {
    for (entity, mut health, _mob, transform, mob_type) in mob_query.iter_mut() {
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
                /* 
                // события "поcле смерти"
                ev_death.send(MobDeathEvent {
                    orbs: loot.orbs,
                    pos: transform.translation,
                    dir: hit.direction,
                });
*/
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