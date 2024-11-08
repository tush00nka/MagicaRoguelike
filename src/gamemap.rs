use bevy::prelude::*;
use avian2d::prelude::*;
use noise::{NoiseFn, Perlin};
use rand::Rng;
use std::collections::HashMap;

use crate::{
    chapter::ChapterManager, health::Health, obstacles::Obstacle, utils::get_random_index_with_weight, GameLayer, GameState
};

pub const ROOM_SIZE: i32 = 32;
pub const TILE_SIZE: f32 = 32.;

pub struct GameMapPlugin;

impl Plugin for GameMapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LevelGenerator::default());
        app.insert_resource(Map::default());
        app.add_systems(OnEnter(GameState::Loading), spawn_map);
        app.add_systems(OnExit(GameState::InGame), init_map);
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum TileType {
    Wall,
    Floor,
    Empty
}

#[derive(Component, Clone, Copy)]
pub struct Floor;

#[derive(Component, Clone, Copy)]
pub struct Wall;

pub struct Walker {
    dir: (f32, f32),
    pos: (f32, f32),
}

#[derive(PartialEq)]
pub struct Tile{
    pub tiletype: TileType,
    pub mob_count: i16,
}

impl Tile{
    fn new(tiletype: TileType, mob_count: i16) -> Self {Tile{tiletype, mob_count}}
}

#[derive(Resource)]
pub struct Map {
    pub map: HashMap<(u16, u16),Tile>,
}

impl Default for Map{
    fn default() -> Self {
        Map{
            map: HashMap::new()
        }
    }
}

fn init_map(mut commands: Commands){
    commands.insert_resource(Map::default());
}

#[derive(Resource)]
pub struct LevelGenerator {
    pub grid: Vec<Vec<TileType>>,
    room_height: usize,
    room_width: usize,
    walkers: Vec<Walker>,
    chance_walker_change_dir: f32,
    chance_walker_spawn: f32,
    chance_walker_destroy: f32,
    chance_walker_spawn_obstacle: f32,
    obstacles: Vec<(f32, f32)>,
    max_walkers: usize,
    percent_to_fill: f32,
}

impl Default for LevelGenerator {
    fn default() -> LevelGenerator {
        LevelGenerator {
            grid: vec![],
            room_height: 0,
            room_width: 0,
            walkers: vec![],
            chance_walker_change_dir: 0.5,
            chance_walker_spawn: 0.05,
            chance_walker_destroy: 0.05,
            chance_walker_spawn_obstacle: 0.05,
            obstacles: vec![],
            max_walkers: 16,
            percent_to_fill: 0.1,
        }
    }
}

impl LevelGenerator {

    fn start(&mut self) {
        self.setup();
        self.create_floors();
        self.create_walls();
        self.remove_single_walls();
    }

    fn random_direction(&self) -> (f32, f32) {
        let choice = rand::thread_rng().gen_range(0..=4);
        match choice {
            0 => (0.0, -1.0), // down
            1 => (-1.0, 0.0), // left
            2 => (0.0, 1.0),  // up
            _ => (1.0, 0.0),  // right
        }
    }

    fn setup(&mut self) {

        self.room_height = ROOM_SIZE as usize;
        self.room_width = ROOM_SIZE as usize;
        // create grid
        self.grid = vec![vec![TileType::Empty; self.room_height]; self.room_width];

        // set first walker
        let spawn_pos = (
            (self.room_width as f32 / 2.0).round(),
            (self.room_height as f32 / 2.0).round(),
        );

        let new_walker = Walker {
            dir: self.random_direction(),
            pos: spawn_pos,
        };

        self.walkers = Vec::new();
        self.walkers.push(new_walker);

        self.obstacles = Vec::new();
    }

    fn create_floors(&mut self) {
        let mut iterations = 0; // loop will not run forever
        let mut rng = rand::thread_rng();

        loop {
            // create floor at position of every walker
            for my_walker in &self.walkers {
                self.grid[my_walker.pos.0 as usize][my_walker.pos.1 as usize] = TileType::Floor;
            }

            // chance: destroy walker
            let number_checks = self.walkers.len();
            for i in 0..number_checks {
                if rng.gen::<f32>() < self.chance_walker_destroy && self.walkers.len() > 1 {
                    self.walkers.remove(i);
                    break; // only destroy one per iteration
                }
            }

            // chance: walker pick new direction
            for i in 0..self.walkers.len() {
                if rng.gen::<f32>() > self.chance_walker_change_dir {
                    self.walkers[i].dir = self.random_direction();
                }
            }

            // chance: spawn new walker
            let number_checks = self.walkers.len();
            for i in 0..number_checks {
                if rng.gen::<f32>() < self.chance_walker_spawn && self.walkers.len() < self.max_walkers {
                    let new_walker = Walker {
                        dir: self.random_direction(),
                        pos: self.walkers[i].pos,
                    };
                    self.walkers.push(new_walker);
                }
            }

            // chance: walker spawns an obstacle/prop
            for walker in self.walkers.iter() {
                if rng.gen::<f32>() < self.chance_walker_spawn_obstacle {
                    if self.obstacles.contains(&(walker.pos)) {
                        continue;    
                    }

                    self.obstacles.push(walker.pos);
                }
            }

            // move walkers
            for walker in &mut self.walkers {
                walker.pos.0 += walker.dir.0;
                walker.pos.1 += walker.dir.1;
            }

            // avoid border of grid
            for walker in &mut self.walkers {
                walker.pos.0 = walker.pos.0.clamp(1.0, (self.room_width - 2) as f32);
                walker.pos.1 = walker.pos.1.clamp(1.0, (self.room_height - 2) as f32);
            }

            // check to exit loop
            if self.number_of_floors() as f32 > (ROOM_SIZE * ROOM_SIZE) as f32 * self.percent_to_fill {
                break;
            }
            iterations += 1;

            if iterations >= 100000 {
                break;
            }
        }
    }

    fn create_walls(&mut self) {
        for x in 0..self.room_width - 1 {
            for y in 0..self.room_height - 1 {
                if self.grid[x][y] == TileType::Floor {
                    if self.grid[x][y + 1] == TileType::Empty {
                        self.grid[x][y + 1] = TileType::Wall;
                    }
                    if self.grid[x][y - 1] == TileType::Empty {
                        self.grid[x][y - 1] = TileType::Wall;
                    }
                    if self.grid[x + 1][y] == TileType::Empty {
                        self.grid[x + 1][y] = TileType::Wall;
                    }
                    if self.grid[x - 1][y] == TileType::Empty {
                        self.grid[x - 1][y] = TileType::Wall;
                    }
                }
            }
        }
    }

    fn remove_single_walls(&mut self) {
        for x in 0..self.room_width - 1 {
            for y in 0..self.room_height - 1 {
                if self.grid[x][y] == TileType::Wall {
                    let mut all_floors = true;
                    for check_x in -1..=1 {
                        for check_y in -1..=1 {
                            if x as isize + check_x < 0 || x as isize + check_x > self.room_width as isize - 1 ||
                               y as isize + check_y < 0 || y as isize + check_y > self.room_height as isize - 1 {
                                continue;
                            }
                            if (check_x != 0 && check_y != 0) || (check_x == 0 && check_y == 0) {
                                continue;
                            }
                            if self.grid[(x as isize + check_x) as usize][(y as isize + check_y) as usize] != TileType::Floor {
                                all_floors = false;
                            }
                        }
                    }
                    if all_floors {
                        self.grid[x][y] = TileType::Floor;
                    }
                }
            }
        }
    }

    fn number_of_floors(&self) -> usize {
        self.grid.iter().flat_map(|row| row.iter()).filter(|&&space| space == TileType::Floor).count()
    }

}

pub fn spawn_map(
    mut room: ResMut<LevelGenerator>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut map: ResMut<Map>,
    chapter_manager: Res<ChapterManager>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    time: Res<Time>,
) {
    room.start();
    let room_height = room.room_height;
    let room_width = room.room_width;
    let grid = &room.grid;
    let obstacles = &room.obstacles;

    for x in 0..room_width {
        for y in 0..room_height {
            match grid[x as usize][y as usize] {
                TileType::Floor => {
                    let texture = asset_server.load(format!("textures/t_floor_{}.png", chapter_manager.get_current_chapter()));
                    let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 4, 1, None, None);
                    let texture_atlas_layout = texture_atlas_layouts.add(layout);

                    map.map.insert((x as u16, y as u16), Tile::new(TileType::Floor, 0));
                    let floor = commands.spawn((
                        SpriteBundle {
                            sprite: Sprite {
                                flip_x: rand::random(),
                                ..default()
                            },
                            texture,
                            transform: Transform::from_xyz(
                                TILE_SIZE * x as f32,
                                TILE_SIZE * y as f32,
                                0.0,
                            ),
                        ..default()
                        },
                        TextureAtlas {
                            layout: texture_atlas_layout,
                            index: get_random_index_with_weight(vec![10, 3, 2, 1])
                        }
                    ))
                    .insert(Floor)
                    .id();

                if y < room_height
                && grid[x as usize][y as usize + 1] == TileType::Wall {
                    commands.entity(floor).with_children(|parent| {
                        parent.spawn(SpriteBundle {
                            texture: asset_server.load("textures/t_shadow.png"),
                            transform: Transform::from_xyz(
                                0.0, 
                                0.0,
                                0.1,
                            ),
                            ..default()
                        });
                    });
                } 

                },
                TileType::Wall => {
                    map.map.insert((x as u16, y as u16), Tile::new(TileType::Wall, 0));

                    let texture_path = {
                        if y > 0 {
                            match grid[x as usize][y as usize - 1] {
                                TileType::Floor => format!("textures/t_wall_top_{}.png", chapter_manager.get_current_chapter()),
                                _ => format!("textures/t_wall_{}.png", chapter_manager.get_current_chapter()),
                            }
                        }
                        else {
                            format!("textures/t_wall_{}.png", chapter_manager.get_current_chapter())
                        }
                    };

                    commands
                        .spawn(SpriteBundle {
                            texture: asset_server.load(texture_path),
                            transform: Transform::from_xyz(
                                TILE_SIZE * x as f32,
                                TILE_SIZE * y as f32,
                                0.0,
                            ),
                            ..default()
                        })
                        .insert(RigidBody::Static)
                        .insert(Collider::rectangle(TILE_SIZE - 0.01, TILE_SIZE - 0.01))
                        .insert(CollisionLayers::new(GameLayer::Wall, [GameLayer::Enemy, GameLayer::Player, GameLayer::Projectile]))
                        .insert(Wall);
                },
                TileType::Empty => {
                    map.map.insert((x as u16, y as u16), Tile::new(TileType::Empty, 0));
                }
            }
        }
    }

    let perlin = Perlin::new(time.elapsed_seconds().round() as u32);

    for pos in obstacles.iter() {
        let height = perlin.get([pos.0 as f64 * 0.1, pos.1 as f64 * 0.1]);

        let texture_path = {
            if height >= 0.5 { "textures/obstacles/claypot.png" }
            else { "textures/obstacles/crate.png"} 
        };

        commands.spawn(SpriteBundle {
            texture: asset_server.load(texture_path),
            transform: Transform::from_xyz(pos.0 * TILE_SIZE, pos.1 * TILE_SIZE, 0.1),
            ..default()
        })
        .insert(Collider::circle(6.))
        .insert(Sensor)
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(CollisionLayers::new(GameLayer::Interactable, [GameLayer::Player, GameLayer::Projectile]))
        .insert(Health::new(10))
        .insert(Obstacle);
    }   

}