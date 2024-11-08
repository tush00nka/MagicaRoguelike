use avian2d::prelude::*;
use bevy::prelude::*;
use seldom_state::prelude::*;

use rand::{thread_rng, Rng};
use std::cmp::Ordering;

use crate::{
    alert::SpawnAlertEvent,
    animation::AnimationConfig,
    blank_spell::Blank,
    boss_room::spawn_boss_room,
    chapter::ChapterManager,
    gamemap::{Map, TileType, ROOM_SIZE},
    level_completion::PortalManager,
    mobs::{mob::*, mob_types::*},
    obstacles::Corpse,
    pathfinding::create_new_graph,
    shield_spell::Shield,
    stun::Stun,
    GameState,
};

pub struct MobSpawnPlugin;
impl Plugin for MobSpawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MobSpawnEvent>()
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
                (spawn_mob, spawner_mob_spawn, handle_raising).run_if(in_state(GameState::InGame)),
            )
            .add_systems(
                OnEnter(GameState::LoadingBoss),
                (boss_spawn, spawn_mob).after(spawn_boss_room),
            );
    }
}

const DESERT_MOBS: &[MobType] = &[
    MobType::Knight,
    //MobType::FireMage
];
const JUNGLE_MOBS: &[MobType] = &[
    MobType::Mossling,
    //MobType::JungleTurret, MobType::WaterMage
];
const INFERNO_MOBS: &[MobType] = &[MobType::Necromancer, MobType::FireMage, MobType::Knight];
//maybe add some minibosses? const BOSSES: &[MobType] = &[MobType::Koldun];

//event to spawn mob with mob_type in pos
#[derive(Event)]
pub struct MobSpawnEvent {
    pub mob_type: MobType,
    pub pos: Vec2,
    pub is_friendly: bool,
}
//enum for types of AI
//#[derive(Component)]
pub enum Mob_AI {
    MeleeWithATK,
    Range,
    RangeWithTP,
    Spawner,
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
    ai_type: Mob_AI,
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
            ai_type: Mob_AI::MeleeWithATK,
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
            ..default()
        }
    }
    fn water_mage() -> Self {
        Self {
            frame_count: 2,
            fps: 3,
            texture_path: "textures/mobs/water_mage.png",
            ..default()
        }
    }
    fn fire_mage() -> Self {
        Self {
            frame_count: 2,
            fps: 3,
            texture_path: "textures/mobs/fire_mage.png",
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
            frame_count: 2,
            fps: 3,
            texture_path: "textures/mobs/fire_mage4.png",
            ..default()
        }
    }
    fn fire_elemental() -> Self {
        Self {
            frame_count: 2,
            fps: 3,
            texture_path: "textures/mobs/fire_mage5.png",
            ..default()
        }
    }
    fn water_elemental() -> Self {
        Self {
            frame_count: 2,
            fps: 3,
            texture_path: "textures/mobs/fire_mage6.png",
            ..default()
        }
    }
    fn earth_elemental() -> Self {
        Self {
            frame_count: 2,
            fps: 3,
            texture_path: "textures/mobs/fire_mage7.png",
            ..default()
        }
    }
    fn wind_elemental() -> Self {
        Self {
            frame_count: 2,
            fps: 3,
            texture_path: "textures/mobs/fire_mage8.png",
            ..default()
        }
    }
    fn tank_eater() -> Self {
        Self {
            frame_count: 2,
            fps: 3,
            texture_path: "textures/mobs/fire_mage9.png",
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
) {
    for ev in ev_mob_spawn.read() {
        portal_manager.push_mob();
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
                spawn_kit = SpawnKit::wind_elemental();
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
            MobType::TankEater => {
                spawn_kit = SpawnKit::tank_eater();
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
            Mob_AI::MeleeWithATK =>
                mob = commands
                    .spawn((
                        StateMachine::default()
                            .trans::<Idle, _>(done(Some(Done::Success)), Pursue)
                            .trans::<Pursue, _>(done(Some(Done::Success)), Attack)
                            .trans::<Pursue, _>(done(Some(Done::Failure)), Idle)
                            .trans::<Attack, _>(done(Some(Done::Success)), Idle),
                        Idle,
                    ))
                    .insert(Enemy) //?????????????
                    .id(),

            Mob_AI::Range =>
                mob = commands
                    .spawn((
                        StateMachine::default()
                            .trans::<Idle, _>(done(Some(Done::Success)), Pursue)
                            .trans::<Pursue, _>(done(Some(Done::Success)), Attack)
                            .trans::<Pursue, _>(done(Some(Done::Failure)), Idle)
                            .trans::<Attack, _>(done(Some(Done::Success)), Idle),
                        Idle,
                    ))
                    .insert(Enemy) //?????????????
                    .id(),

            Mob_AI::RangeWithTP =>
                mob = commands
                    .spawn((
                        StateMachine::default()
                            .trans::<Teleport, _>(done(Some(Done::Success)), ShootAbility::default())
                            .trans::<ShootAbility, _>(done(Some(Done::Success)), Teleport::default())
                            .trans::<ShootAbility, _>(done(Some(Done::Failure)), Teleport::default()),
                        Teleport::default(), //TODO: add impl for complex components w/ type of mobs, add
                    ))
                    .insert(Enemy) //?????????????
                    .id(),
            
            Mob_AI::Spawner => 
                mob = commands
                    .spawn((
                        StateMachine::default()
                            .trans::<RunawayRush, _>(done(Some(Done::Success)), CorpseRush)
                            .trans::<CorpseRush, _>(done(Some(Done::Success)), RaisingFlag)
                            .trans::<CorpseRush, _>(done(Some(Done::Failure)), RunawayRush)
                            .trans::<RaisingFlag, _>(done(Some(Done::Success)), CorpseRush),
                        RunawayRush,
                    ))
                    .insert(Enemy) //?????????????
                    .id(),
        }

        commands
            .entity(mob)
            .insert(SpriteBundle {
                texture,
                transform: Transform::from_xyz(x, y, 1.0),
                ..default()
            })
            .insert(Enemy); //?????????????

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
                    .insert(Idle);
            }
            MobType::Mossling => {
                commands
                    .entity(mob)
                    .insert(MeleeMobBundle::mossling())
                    .insert(SearchAndPursue::default())
                    .insert(Idle);
            }
            MobType::FireMage => {
                commands.entity(mob).insert(MageBundle::fire_mage());

                mob_map
                    .map
                    .get_mut(&(
                        (x / ROOM_SIZE as f32).floor() as u16,
                        (y / ROOM_SIZE as f32).floor() as u16,
                    ))
                    .unwrap()
                    .mob_count += 1;
            }
            MobType::WaterMage => {
                commands.entity(mob).insert(MageBundle::water_mage());
                mob_map
                    .map
                    .get_mut(&((x / ROOM_SIZE as f32) as u16, (y / ROOM_SIZE as f32) as u16))
                    .unwrap()
                    .mob_count += 1;
            }
            MobType::JungleTurret => {
                commands.entity(mob).insert(TurretBundle::jungle_turret());

                mob_map
                    .map
                    .get_mut(&((x / ROOM_SIZE as f32) as u16, (y / ROOM_SIZE as f32) as u16))
                    .unwrap()
                    .mob_count += 1;
            }
            MobType::Necromancer => {
                commands.entity(mob).insert(SpawnerBundle::necromancer());
                //add necro bundles
            }
            MobType::Koldun => {
                commands.entity(mob).insert(BossBundle::koldun());
            }
            MobType::EarthElemental => {}
            MobType::FireElemental => {}
            MobType::WaterElemental => {}
            MobType::AirElemental => {}
            MobType::ClayGolem => {}
            MobType::SkeletMage => {}
            MobType::SkeletWarrior => {}
            MobType::SkeletRanger => {}
            MobType::TankEater => {}
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
                });
            }
        }
    }

    game_state.set(GameState::InGame);
}

fn spawner_mob_spawn(
    mut commands: Commands,
    mut ev_spawn: EventWriter<MobSpawnEvent>,
    mut summoner_query: Query<(Entity, &mut Summoning, &Raising, &mut Sprite), Without<Stun>>,
    corpse_query: Query<Entity, (With<Corpse>, With<BusyRaising>)>,
    time: Res<Time>,
) {
    for (summoner, mut summoning, raising, mut sprite) in summoner_query.iter_mut() {
        if !corpse_query.contains(raising.corpse_id) {
            commands.entity(summoner).remove::<Raising>();
            sprite.color = Color::srgb(1., 1., 1.);
            continue;
        }
        summoning.time_to_spawn.tick(time.delta());
        if summoning.time_to_spawn.just_finished() {
            ev_spawn.send(MobSpawnEvent {
                mob_type: raising.mob_type.clone(),
                pos: raising.mob_pos.translation.truncate(),
                is_friendly: false,
            });

            commands.entity(raising.corpse_id).despawn();
            commands.entity(summoner).remove::<Raising>();
            sprite.color = Color::srgb(1., 1., 1.);
        }
    }
}

fn handle_raising(mut raising_query: Query<(&mut Sprite, &mut LinearVelocity), Changed<Raising>>) {
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
    });
    game_state.set(GameState::InGame);
}
