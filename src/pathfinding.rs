//A* Pathfinding for enemies
use crate::player::Player;
use bevy::prelude::*;
pub struct PathfindingPlugin;

impl Plugin for PathfindingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, a_pathfinding);
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum TileType {
    Wall,
    Floor,
    Empty,
}

#[derive(Clone)]
enum Address {
    Address(Box<Node>),
    Nil,
}

#[derive(Clone)]
pub struct Node {
    cost: i32,
    prev_node: Address,
    tile_type: TileType,
    position: Vec2,
}

impl Node {
    pub fn new(cost_new: i32, tile_type_new: TileType, position_new: Vec2) -> Self {
        Node {
            cost: cost_new,
            prev_node: Address::Nil,
            tile_type: tile_type_new,
            position: position_new,
        }
    }
}

fn a_pathfinding(
    mut player_query: Query<(&Transform, &Player)>,
    mut mob_query: Query<(&Transform, With<Mob>)>,
) {
    for i in &mob_query {
        //для каждого моба делать поиск пути?
        if let Ok((mut mob_transform, &Mob)) = player_query.get_single_mut() {
            let mut start_node = Node::new(
                0,
                TileType::Floor,
                Vec2::new(i.transform.translation.x, i.transform.translation.y),
            ); //или бесконечно большую ставить?
        }
    }
}
fn create_new_nodes() {
    //создаем полигоны вокруг себя
}
fn build_path() {
    //воссоздаем и возвращаем путь
}

fn pick_node() {
    //алгоритм выбора нода, который ближе всего к игроку
}
