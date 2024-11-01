use avian2d::prelude::*;
use bevy::prelude::*;
use rand::{thread_rng, Rng};

use crate::{
    animation::AnimationConfig,
    chapter::ChapterManager,
    gamemap::{Map, TileType, ROOM_SIZE},
    level_completion::PortalManager,
    mobs::mob::*,
    mobs::mob_types::*,
    obstacles::Corpse,
    pathfinding::create_new_graph,
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
            );
    }
}

const DESERT_MOBS: &[MobType] = &[MobType::Knight, MobType::FireMage];
const JUNGLE_MOBS: &[MobType] = &[MobType::Mossling, MobType::JungleTurret, MobType::WaterMage];

//event to spawn mob with mob_type in pos
#[derive(Event)]
pub struct MobSpawnEvent {
    pub mob_type: MobType,
    pub pos: Vec2,
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
    fn necromancer() -> Self {
        Self {
            frame_count: 4,
            fps: 12,
            texture_path: "textures/mobs/necromancer.png",
            pixel_size: 24,
            can_flip:true,
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
        if spawn_kit.can_flip{
            commands.entity(mob).insert(FlipEntity);
        }
        match ev.mob_type {
            MobType::Knight => {
                commands.entity(mob).insert(MeleeMobBundle::knight());
                commands.entity(mob).insert(SearchAndPursue::default());
                commands.entity(mob).insert(RayCaster::default());
            }
            MobType::Mossling => {
                commands.entity(mob).insert(MeleeMobBundle::mossling());
                commands.entity(mob).insert(SearchAndPursue::default());
                commands.entity(mob).insert(RayCaster::default());
            }
            MobType::FireMage => {
                commands.entity(mob).insert(MageBundle::fire_mage());
                commands.entity(mob).insert(RayCaster::default());

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
                commands.entity(mob).insert(RayCaster::default());

                mob_map
                    .map
                    .get_mut(&((x / ROOM_SIZE as f32) as u16, (y / ROOM_SIZE as f32) as u16))
                    .unwrap()
                    .mob_count += 1;
            }
            MobType::JungleTurret => {
                commands.entity(mob).insert(TurretBundle::jungle_turret());
                commands.entity(mob).insert(RayCaster::default());

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
                match chapter_manager.get_current_chapter() {
                    1 => {
                        let mob_index = rand::thread_rng().gen_range(0..DESERT_MOBS.len());
                        mob_type = DESERT_MOBS[mob_index].clone();
                    }
                    2 => {
                        let mob_index = rand::thread_rng().gen_range(0..JUNGLE_MOBS.len());
                        mob_type = JUNGLE_MOBS[mob_index].clone();
                    }
                    _ => {
                        mob_type = rand::random();
                    }
                }

                ev_mob_spawn.send(MobSpawnEvent {
                    mob_type,
                    pos: Vec2::new((x * ROOM_SIZE) as f32, (y * ROOM_SIZE) as f32),
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
