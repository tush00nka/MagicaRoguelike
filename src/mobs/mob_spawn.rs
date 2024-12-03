use avian2d::prelude::*;
use bevy::prelude::*;
use seldom_state::prelude::*;

use rand::{thread_rng, Rng};
use serde::de;

use crate::{
    animation::AnimationConfig,
    boss_room::spawn_boss_room,
    camera::YSort,
    chapter::ChapterManager,
    friend::Friend,
    gamemap::{Map, TileType, ROOM_SIZE},
    item::ItemType,
    level_completion::PortalManager,
    mobs::{mob::*, mob_types::*, MultistateAnimationFlag, OnCooldownFlag},
    obstacles::Corpse,
    pathfinding::{create_new_graph, FriendRush},
    stun::Stun,
    GameState,
};

use super::{
    pick_attack_to_perform_koldun, BeforeAttackDelayBoss, BossAttackFlagComp, BossAttackType,
    PickAttackFlag,
};

pub struct MobSpawnPlugin;
impl Plugin for MobSpawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MobSpawnEvent>()
            .add_event::<PushMobQueueEvent>()
            .add_systems(
                OnEnter(GameState::Loading),
                spawn_mobs_location.after(create_new_graph),
            )
            .add_systems(
                OnEnter(GameState::Loading),
                first_spawn_mobs.after(spawn_mobs_location),
            )
            .add_systems(Update, (spawn_mob, push_mob_to_queue))
            .add_systems(
                Update,
                (spawner_mob_spawn, handle_raising).run_if(in_state(GameState::InGame)),
            )
            .add_systems(
                OnEnter(GameState::LoadingBoss),
                (boss_spawn, spawn_mob).after(spawn_boss_room),
            );
    }
}

const DESERT_MOBS: &[MobType] = &[
    MobType::Knight,
    MobType::FireMage,
    MobType::EarthElemental,
    MobType::WaterElemental,
    MobType::Thief,
];

const JUNGLE_MOBS: &[MobType] = &[
    MobType::Mossling,
    MobType::JungleTurret,
    MobType::WaterMage,
    MobType::AirElemental,
    MobType::ClayGolem,
];

const INFERNO_MOBS: &[MobType] = &[
    MobType::Necromancer,
    MobType::FireMage,
    MobType::Knight,
    MobType::FireElemental,
];
//maybe add some minibosses? const BOSSES: &[MobType] = &[MobType::Koldun];

//event to spawn mob with mob_type in pos
#[derive(Event)]
pub struct MobSpawnEvent {
    pub mob_type: MobType,
    pub pos: Vec2,
    pub is_friendly: bool,
    pub owner: Option<Entity>,
    pub loot: Option<ItemPicked>,
    pub exp_amount: i8,
}
pub enum MobAI {
    MeleeWithATK,
    RangeMoving,
    RangeWithTP,
    Spawner,
    Turret,
    Phasing,
    Orbital,
    Thief,
    Koldun,
    //etc, add later
}
//structures for init=======================================================================================================================
//add here if you need to check another one parameter when you spawn mob. don't forget to add this parameter for impl
pub struct SpawnKit<'a> {
    texture_path: &'a str,
    frame_count: u32,
    fps: u8,
    rotation_entity: bool,
    rotation_path: &'a str,
    can_flip: bool,
    has_animation: bool,
    pixel_size: u32,
    ai_type: MobAI,
}

impl Default for SpawnKit<'_> {
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
            ai_type: MobAI::MeleeWithATK,
        }
    }
}

impl SpawnKit<'_> {
    fn knight() -> Self {
        Self {
            texture_path: "textures/mobs/knight.png",
            ..default()
        }
    }
    fn mossling() -> Self {
        Self {
            texture_path: "textures/mobs/mossling.png",
            ..default()
        }
    }
    fn jungle_turret() -> Self {
        Self {
            texture_path: "textures/mobs/plant_body.png",
            rotation_path: "textures/mobs/plant_head.png",
            rotation_entity: true,
            has_animation: false,
            pixel_size: 24,
            ai_type: MobAI::Turret,
            ..default()
        }
    }
    fn water_mage() -> Self {
        Self {
            frame_count: 2,
            fps: 3,
            texture_path: "textures/mobs/water_mage.png",
            ai_type: MobAI::RangeWithTP,
            ..default()
        }
    }
    fn fire_mage() -> Self {
        Self {
            frame_count: 2,
            fps: 3,
            texture_path: "textures/mobs/fire_mage.png",
            ai_type: MobAI::RangeWithTP,
            ..default()
        }
    }
    fn skelet_mage() -> Self {
        Self {
            frame_count: 2,
            fps: 3,
            texture_path: "textures/mobs/fire_mage1.png",
            ..default()
        }
    }
    fn skelet_warrior() -> Self {
        Self {
            frame_count: 2,
            fps: 3,
            texture_path: "textures/mobs/fire_mage2.png",
            ..default()
        }
    }
    fn skelet_ranger() -> Self {
        Self {
            frame_count: 2,
            fps: 3,
            texture_path: "textures/mobs/fire_mage3.png",
            ..default()
        }
    }
    fn clay_golem() -> Self {
        Self {
            frame_count: 4,
            fps: 12,
            texture_path: "textures/mobs/clay_golem.png",
            pixel_size: 32,
            ..default()
        }
    }
    fn fire_elemental() -> Self {
        Self {
            frame_count: 2,
            fps: 3,
            texture_path: "textures/mobs/fire_elemental.png",
            ai_type: MobAI::Phasing,
            has_animation: false,
            ..default()
        }
    }
    fn water_elemental() -> Self {
        Self {
            frame_count: 2,
            fps: 3,
            texture_path: "textures/mobs/water_elemental.png",
            ai_type: MobAI::RangeMoving,
            has_animation: false,
            ..default()
        }
    }
    fn earth_elemental() -> Self {
        Self {
            frame_count: 2,
            fps: 3,
            texture_path: "textures/mobs/earth_elemental.png",
            ai_type: MobAI::Turret,
            has_animation: false,
            ..default()
        }
    }
    fn air_elemental() -> Self {
        Self {
            frame_count: 2,
            fps: 3,
            texture_path: "textures/mobs/air_elemental.png",
            ai_type: MobAI::Orbital,
            has_animation: false,
            ..default()
        }
    }
    fn thief() -> Self {
        Self {
            frame_count: 6,
            fps: 12,
            texture_path: "textures/mobs/lurker_run.png",
            can_flip: true,
            ai_type: MobAI::Thief,
            pixel_size: 24,
            ..default()
        }
    }
    fn necromancer() -> Self {
        Self {
            frame_count: 4,
            fps: 12,
            texture_path: "textures/mobs/necromancer.png",
            pixel_size: 24,
            can_flip: true,
            ai_type: MobAI::Spawner,
            ..default()
        }
    }
    fn koldun() -> Self {
        Self {
            texture_path: "textures/mobs/koldun.png",
            frame_count: 2,
            fps: 3,
            can_flip: true,
            has_animation: true,
            pixel_size: 48,
            ai_type: MobAI::Koldun,
            ..default()
        }
    }
}
//assist functions==================================================================================
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

//actual code========================================================================================================
fn spawn_mobs_location(mut mob_map: ResMut<Map>, chapter_manager: Res<ChapterManager>) {
    let chap_num = chapter_manager.get_current_chapter();
    let mut rng = thread_rng();
    let mut mobs_amount: u16 = rng.gen_range(1 + 5 * chap_num as u16..5 + 5 * chap_num as u16);
    let mut chance: f32;

    if chapter_manager.get_current_chapter() % 4 == 0 {
        mobs_amount = chapter_manager.get_current_chapter() as u16 / 4 as u16;
    }

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
    mut portal_manager: ResMut<PortalManager>,
    mut summon_queue_ev: EventWriter<PushMobQueueEvent>,
) {
    for ev in ev_mob_spawn.read() {
        if !ev.is_friendly {
            portal_manager.push_mob();
        }
        let spawn_kit: SpawnKit;

        let x = ev.pos.x;
        let y = ev.pos.y;

        //pick mob with random, assign some variables
        match ev.mob_type {
            MobType::Knight => {
                spawn_kit = SpawnKit::knight();
            }
            MobType::Mossling => {
                spawn_kit = SpawnKit::mossling();
            }
            MobType::FireMage => {
                spawn_kit = SpawnKit::fire_mage();
            }
            MobType::WaterMage => {
                spawn_kit = SpawnKit::water_mage();
            }
            MobType::JungleTurret => {
                spawn_kit = SpawnKit::jungle_turret();
            }
            MobType::Necromancer => {
                spawn_kit = SpawnKit::necromancer();
            }
            MobType::Koldun => {
                spawn_kit = SpawnKit::koldun();
            }
            MobType::EarthElemental => {
                spawn_kit = SpawnKit::earth_elemental();
            }
            MobType::FireElemental => {
                spawn_kit = SpawnKit::fire_elemental();
            }
            MobType::WaterElemental => {
                spawn_kit = SpawnKit::water_elemental();
            }
            MobType::AirElemental => {
                spawn_kit = SpawnKit::air_elemental();
            }
            MobType::ClayGolem => {
                spawn_kit = SpawnKit::clay_golem();
            }
            MobType::SkeletMage => {
                spawn_kit = SpawnKit::skelet_mage();
            }
            MobType::SkeletWarrior => {
                spawn_kit = SpawnKit::skelet_warrior();
            }
            MobType::SkeletRanger => {
                spawn_kit = SpawnKit::skelet_ranger();
            }
            MobType::Thief => {
                spawn_kit = SpawnKit::thief();
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

        //spawn mob, add state machine to it
        let mob: Entity;

        match spawn_kit.ai_type {
            MobAI::Koldun => {
                mob = commands
                    .spawn((
                        StateMachine::default()
                            .trans::<OnCooldownFlag, _>(done(Some(Done::Success)), PickAttackFlag)
                            .trans_builder(
                                pick_attack_to_perform_koldun,
                                |_: &PickAttackFlag, attack_type| {
                                    Some(match attack_type {
                                        Some(val) => match val {
                                            _ => BossAttackFlagComp { attack_picked: val },
                                        },
                                        None => BossAttackFlagComp {
                                            attack_picked: BossAttackType::Wall,
                                        },
                                    })
                                },
                            )
                            .trans::<BossAttackFlagComp, _>(
                                done(Some(Done::Success)),
                                OnCooldownFlag,
                            )
                            .on_enter::<BossAttackFlagComp>(move |entity| {
                                entity.insert(BeforeAttackDelayBoss::default());
                            })
                            //         .on_exit::<BossAttackFlagComp>(|entity| {
                            //             entity.remove::<BeforeAttackDelayBoss>();
                            //         })
                            ,
                        OnCooldownFlag,
                    ))
                    .id();
            }
            MobAI::Orbital => {
                mob = commands
                    .spawn((
                        StateMachine::default()
                            .trans::<FreeOrbital, _>(done(Some(Done::Success)), BusyOrbital)
                            .trans::<BusyOrbital, _>(done(Some(Done::Success)), FreeOrbital),
                        FreeOrbital,
                    ))
                    .id()
            }
            MobAI::Phasing => {
                mob = commands
                    .spawn((
                        StateMachine::default()
                            .trans::<PhasingFlag, _>(
                                done(Some(Done::Success)),
                                BeforeAttackDelay::default(),
                            )
                            .trans::<BeforeAttackDelay, _>(done(Some(Done::Success)), AttackFlag)
                            .trans::<AttackFlag, _>(done(Some(Done::Success)), PhasingFlag),
                        PhasingFlag,
                    ))
                    .id()
            }

            MobAI::MeleeWithATK => {
                // done
                mob = commands
                    .spawn((
                        StateMachine::default()
                            .trans::<Idle, _>(done(Some(Done::Success)), Pursue)
                            .trans::<Pursue, _>(
                                done(Some(Done::Success)),
                                BeforeAttackDelay::default(),
                            )
                            .trans::<Pursue, _>(done(Some(Done::Failure)), Idle)
                            .trans::<BeforeAttackDelay, _>(done(Some(Done::Success)), AttackFlag)
                            .trans::<AttackFlag, _>(done(Some(Done::Success)), Idle),
                        Idle,
                    ))
                    .id()
            }

            MobAI::RangeMoving => {
                if ev.is_friendly {
                    mob = commands
                        .spawn((
                            StateMachine::default()
                                .trans::<FriendRush, _>(done(Some(Done::Success)), Idle)
                                .trans::<Idle, _>(done(Some(Done::Success)), Pursue)
                                .trans::<Idle, _>(done(Some(Done::Failure)), FriendRush::default())
                                .trans::<Pursue, _>(
                                    done(Some(Done::Success)),
                                    BeforeAttackDelay::default(),
                                )
                                .trans::<Pursue, _>(done(Some(Done::Failure)), Idle)
                                .trans::<BeforeAttackDelay, _>(
                                    done(Some(Done::Success)),
                                    AttackFlag,
                                )
                                .trans::<AttackFlag, _>(done(Some(Done::Success)), Idle),
                            FriendRush::default(),
                        ))
                        .id()
                } else {
                    println!("Spawn is correct");
                    mob = commands
                        .spawn((
                            StateMachine::default()
                                .trans::<Idle, _>(done(Some(Done::Success)), Pursue)
                                .trans::<Pursue, _>(
                                    done(Some(Done::Success)),
                                    BeforeAttackDelay::default(),
                                )
                                .trans::<Pursue, _>(done(Some(Done::Failure)), Idle)
                                .trans::<BeforeAttackDelay, _>(
                                    done(Some(Done::Success)),
                                    AttackFlag,
                                )
                                .trans::<AttackFlag, _>(done(Some(Done::Success)), Idle),
                            Idle,
                        ))
                        .id()
                }
            }
            MobAI::RangeWithTP => {
                //done
                mob = commands
                    .spawn((
                        StateMachine::default()
                            .trans::<TeleportFlag, _>(done(Some(Done::Success)), IdleStatic)
                            .trans::<IdleStatic, _>(
                                done(Some(Done::Success)),
                                BeforeAttackDelay::default(),
                            )
                            .trans::<IdleStatic, _>(done(Some(Done::Failure)), TeleportFlag)
                            .trans::<BeforeAttackDelay, _>(done(Some(Done::Success)), AttackFlag)
                            .trans::<AttackFlag, _>(done(Some(Done::Success)), TeleportFlag), //change: need to create smth like tp -> check range -> alert -> shoot
                        TeleportFlag, //TODO: add impl for complex components w/ type of mobs, add
                    ))
                    .id()
            }

            MobAI::Spawner => {
                //done
                mob = commands
                    .spawn((
                        StateMachine::default()
                            .trans::<RunawayRush, _>(done(Some(Done::Success)), CorpseRush)
                            .trans::<CorpseRush, _>(done(Some(Done::Success)), RaisingFlag)
                            .trans::<CorpseRush, _>(done(Some(Done::Failure)), RunawayRush)
                            .trans::<RaisingFlag, _>(done(Some(Done::Success)), CorpseRush),
                        RunawayRush,
                    ))
                    .id()
            }

            MobAI::Turret => {
                //done
                mob = commands
                    .spawn((
                        StateMachine::default()
                            .trans::<IdleStatic, _>(
                                done(Some(Done::Success)),
                                BeforeAttackDelay::default(),
                            )
                            .trans::<BeforeAttackDelay, _>(done(Some(Done::Success)), AttackFlag)
                            .trans::<AttackFlag, _>(done(Some(Done::Success)), IdleStatic),
                        IdleStatic, //TODO: add impl for complex components w/ type of mobs, add
                    ))
                    .id()
            }
            MobAI::Thief => {
                mob = commands
                    .spawn((
                        StateMachine::default()
                            .trans_builder(nearest_interactable, |_: &RunawayRush, value| {
                                Some(match value {
                                    Some(_) => PickTargetForSteal { target: value },
                                    None => PickTargetForSteal { target: None },
                                })
                            })
                            .trans_builder(pick_item_to_steal, |_: &PickTargetForSteal, value| {
                                Some(match value {
                                    Some(val) => match val {
                                        ItemPicked::HPTank => {
                                            ItemPickedFlag::Some(ItemPicked::HPTank)
                                        }
                                        ItemPicked::EXPTank => {
                                            ItemPickedFlag::Some(ItemPicked::EXPTank)
                                        }
                                        ItemPicked::Item => ItemPickedFlag::Some(ItemPicked::Item),
                                        ItemPicked::Obstacle => {
                                            ItemPickedFlag::Some(ItemPicked::Obstacle)
                                        }
                                    },
                                    None => ItemPickedFlag::None,
                                })
                            })
                            .trans::<ItemPickedFlag, _>(
                                done(Some(Done::Success)),
                                SearchingInteractableFlag,
                            )
                            .trans::<ItemPickedFlag, _>(done(Some(Done::Failure)), RunawayRush)
                            .trans::<SearchingInteractableFlag, _>(
                                done(Some(Done::Success)),
                                RunawayRush,
                            )
                            .trans::<SearchingInteractableFlag, _>(
                                done(Some(Done::Failure)),
                                RunawayRush,
                            ),
                        RunawayRush,
                    ))
                    .id()
            }
        };
        if ev.is_friendly {
            commands.entity(mob).insert(Friend);
        } else {
            commands.entity(mob).insert(Enemy);
        }
        
        let mut ysort_size = spawn_kit.pixel_size as f32 / 2.; 
        
        if ev.mob_type == MobType::FireElemental || ev.mob_type == MobType::AirElemental{
            ysort_size = 48.;
        }

        commands
            .entity(mob)
            .insert(SpriteBundle {
                texture,
                transform: Transform::from_xyz(x, y, 1.0),
                ..default()
            })
            .insert(YSort(ysort_size));

        if spawn_kit.has_animation {
            commands
                .entity(mob)
                .insert(TextureAtlas {
                    layout: texture_atlas_layout.clone(),
                    index: animation_config.first_sprite_index,
                })
                .insert(animation_config);
        }
        if spawn_kit.can_flip {
            commands.entity(mob).insert(FlipEntity);
        }
        match ev.mob_type {
            MobType::Knight => {
                commands
                    .entity(mob)
                    .insert(MeleeMobBundle::knight())
                    .insert(SearchAndPursue::default())
                    .insert(MultistateAnimationFlag);
            }
            MobType::Mossling => {
                commands
                    .entity(mob)
                    .insert(MeleeMobBundle::mossling())
                    .insert(SearchAndPursue::default())
                    .insert(MultistateAnimationFlag);
            }
            MobType::FireMage => {
                commands.entity(mob).insert(MageBundle::fire_mage());
            }
            MobType::WaterMage => {
                commands.entity(mob).insert(MageBundle::water_mage());
            }
            MobType::JungleTurret => {
                commands.entity(mob).insert(TurretBundle::jungle_turret());
            }
            MobType::Necromancer => {
                commands.entity(mob).insert(SpawnerBundle::necromancer());
                //add necro bundles
            }
            MobType::Koldun => {
                commands
                    .entity(mob)
                    .insert(BossBundle::koldun())
                    .insert(FirstPhase);
            }

            MobType::EarthElemental => {
                commands.entity(mob).insert(TurretBundle::earth_elemental());
            }
            MobType::FireElemental => {
                commands
                    .entity(mob)
                    .insert(MeleePhasingBundle::fire_elemental());
            }
            MobType::WaterElemental => {
                commands
                    .entity(mob)
                    .insert(RangeMobBundle::water_elemental());
            }
            MobType::AirElemental => {
                commands.entity(mob).insert(OrbitalBundle::air_elemental());
            }
            MobType::ClayGolem => {
                commands
                    .entity(mob)
                    .insert(MeleeMobBundle::clay_golem())
                    .insert(SearchAndPursue::default());
            }
            MobType::SkeletMage => {}
            MobType::SkeletWarrior => {}
            MobType::SkeletRanger => {}
            MobType::Thief => {
                commands.entity(mob).insert(ThiefBundle::default());
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

        if STATIC_MOBS.contains(&ev.mob_type) {
            let mob_pos = (
                (x.floor() / 32.).floor() as u16,
                (y.floor() / 32.).floor() as u16,
            );

            mob_map
                .map
                .get_mut(&(mob_pos.0, mob_pos.1))
                .unwrap()
                .mob_count += 1;
        }
        if ev.owner.is_some() {
            summon_queue_ev.send(PushMobQueueEvent {
                owner: ev.owner.unwrap(),
                mob_type: ev.mob_type.clone(),
                mob_e: mob,
            });
        }
        if ev.exp_amount >= 0 {
            commands.entity(mob).remove::<MobLoot>();
            commands.entity(mob).insert(MobLoot {
                orbs: ev.exp_amount as u32,
            });
        }
        if ev.loot.is_some() {
            commands.entity(mob).remove::<PickupItem>();
            let mut item: Option<ItemType> = None;
            if ev.loot.clone().unwrap() == ItemPicked::Item {
                item = Some(rand::random());
            }
            commands.entity(mob).insert(PickupItem {
                item_type: ev.loot.clone().unwrap(),
                item_name: item,
            });
        }
    }
}

pub fn first_spawn_mobs(
    mut mob_map: ResMut<Map>,
    mut game_state: ResMut<NextState<GameState>>,
    mut ev_mob_spawn: EventWriter<MobSpawnEvent>,
    chapter_manager: Res<ChapterManager>,
) {
    for x in 1..ROOM_SIZE - 1 {
        for y in 1..ROOM_SIZE - 1 {
            if mob_map.map.get(&(x as u16, y as u16)).unwrap().mob_count == i16::MAX {
                mob_map
                    .map
                    .get_mut(&(x as u16, y as u16))
                    .unwrap()
                    .mob_count = 0;

                let mob_type: MobType;
                println!("chapter  {}", chapter_manager.get_current_chapter());
                let chapter: u8 = chapter_manager.get_current_chapter() % 4;
                match chapter {
                    1 => {
                        let mob_index = rand::thread_rng().gen_range(0..DESERT_MOBS.len());
                        mob_type = DESERT_MOBS[mob_index].clone();
                    }
                    2 => {
                        let mob_index = rand::thread_rng().gen_range(0..JUNGLE_MOBS.len());
                        mob_type = JUNGLE_MOBS[mob_index].clone();
                    }
                    3 => {
                        let mob_index = rand::thread_rng().gen_range(0..INFERNO_MOBS.len());
                        mob_type = INFERNO_MOBS[mob_index].clone();
                    }
                    _ => {
                        mob_type = rand::random();
                    }
                }

                ev_mob_spawn.send(MobSpawnEvent {
                    mob_type,
                    pos: Vec2::new((x * ROOM_SIZE) as f32, (y * ROOM_SIZE) as f32),
                    is_friendly: false,
                    owner: None, // can add smth
                    loot: None,  //
                    exp_amount: -1,
                });
            }
        }
    }

    game_state.set(GameState::InGame);
}

fn spawner_mob_spawn(
    mut commands: Commands,
    mut ev_spawn: EventWriter<MobSpawnEvent>,
    mut summoner_query: Query<
        (Entity, &mut Summoning, &Raising, &mut Sprite),
        (Without<Stun>, With<RaisingFlag>),
    >,
    corpse_query: Query<Entity, (With<Corpse>, With<BusyRaising>)>,
    time: Res<Time>,
) {
    for (summoner, mut summoning, raising, mut sprite) in summoner_query.iter_mut() {
        if !corpse_query.contains(raising.corpse_id) {
            commands.entity(summoner).insert(Done::Success);
            sprite.color = Color::srgb(1., 1., 1.);
            continue;
        }
        summoning.time_to_spawn.tick(time.delta());
        if summoning.time_to_spawn.just_finished() {
            ev_spawn.send(MobSpawnEvent {
                mob_type: raising.mob_type.clone(),
                pos: raising.mob_pos.translation.truncate(),
                is_friendly: false,
                owner: None,
                loot: None,
                exp_amount: 0,
            });

            commands.entity(raising.corpse_id).despawn();
            commands.entity(summoner).insert(Done::Success);
            sprite.color = Color::srgb(1., 1., 1.);
        }
    }
}

fn handle_raising(
    mut raising_query: Query<(&mut Sprite, &mut LinearVelocity), Changed<RaisingFlag>>,
) {
    for (mut sprite, mut linvel) in raising_query.iter_mut() {
        sprite.color = Color::srgb(1., 3., 3.);
        linvel.0 = Vec2::ZERO;
    }
}

fn boss_spawn(
    mut ev_spawn: EventWriter<MobSpawnEvent>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    ev_spawn.send(MobSpawnEvent {
        mob_type: MobType::Koldun,
        pos: Vec2::new(
            (ROOM_SIZE / 2) as f32 * 32.,
            (ROOM_SIZE / 2 - 5) as f32 * 32.,
        ),
        is_friendly: false,
        owner: None,
        loot: None,
        exp_amount: -1,
    });
    game_state.set(GameState::InGame);
}

pub fn push_mob_to_queue(
    mut push_mob_ev: EventReader<PushMobQueueEvent>,
    mut commands: Commands,
    mut list_query: Query<&mut SummonQueue>,
    transform_query: Query<&Transform>,
    mut ev_mob_death: EventWriter<MobDeathEvent>,
) {
    for ev in push_mob_ev.read() {
   /*println!(
            "add entity: {}, mob type to push - {}",
            ev.mob_e,
            ev.mob_type.clone() as u32
        );
 */     
        if list_query.contains(ev.owner) {
            
            let mut summoner = list_query.get_mut(ev.owner).unwrap();
            let mut despawn_entity = SummonUnit {
                entity: None,
                mob_type: MobType::Mossling,
            };

            if summoner.queue[summoner.queue.len() - 1].entity.is_some() {
                despawn_entity = summoner.queue[summoner.queue.len() - 1].clone();
            }

            for i in (1..summoner.queue.len() - 1).rev() {
                summoner.queue[i] = summoner.queue[i - 1].clone();
            }

            summoner.queue[0] = SummonUnit {
                entity: Some(ev.mob_e),
                mob_type: ev.mob_type.clone(),
            };

        /*    for i in summoner.queue.iter() {
               println!(
                    "entity:{} mob_type: {}",
                    i.entity.is_some(),
                    i.mob_type.clone() as u32
                );
            }

            println!(
                "despawn entity:{} mob_type: {}",
                despawn_entity.entity.is_some(),
                despawn_entity.mob_type as u32
            );
*/
            if despawn_entity.entity.is_some() {
                let transform = transform_query.get(despawn_entity.entity.unwrap()).unwrap();

                commands
                    .entity(despawn_entity.entity.unwrap())
                    .despawn_recursive();
                println!("wow");

                ev_mob_death.send(MobDeathEvent {
                    orbs: 0,
                    pos: transform.translation,
                    dir: Vec3::ZERO,
                });
            }
        }
    }
}
