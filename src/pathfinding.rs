//A* Pathfinding for enemies
use crate::player::Player;
use bevy::prelude::*;
pub struct PathfindingPlugin;

impl Plugin for PathfindingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, create_new_graph);
        app.add_systems(Update, a_pathfinding);
    }
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
    tile_type: crate::gamemap::TileType,
    position: Vec2,
}
pub struct Graph{
    adj_list: std::collections::HashMap<(u32,u32),std::collections::LinkedList<Node>>,
}

impl Graph{
    pub fn new()->Self{
        Graph{
            adj_list:std::collections::HashMap::new()
        }
    }
    fn Add_Node_List(&mut self,tup:(u32,u32),node:Node){
        let mut list = std::collections::LinkedList::new();
        list.push_back(node);
        self.adj_list.insert(tup,list.clone());
    }
    fn Add_Node_To_List(&mut self,tup:(u32,u32),node:Node){
        let mut list = self.adj_list[&tup].clone();
        list.push_back(node);
        self.adj_list.insert(tup,list);
    }
}
impl Node {
    pub fn new(cost_new: i32, tile_type_new: crate::gamemap::TileType, position_new: Vec2) -> Self {
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
    mut room: ResMut<crate::gamemap::LevelGenerator>
) {
    for i in &mob_query {
        //для каждого моба делать поиск пути?
        if let Ok((mut mob_transform, &Mob)) = player_query.get_single_mut() {
            let mut start_node = Node::new(
                0,
                crate::gamemap::TileType::Floor,
                Vec2::new(i.transform.translation.x, i.transform.translation.y),
            ); 
        }
    }
}

fn create_new_graph(room: ResMut<crate::gamemap::LevelGenerator>) {
    let grid = room.grid.clone();
    let mut graph_search:Graph = Graph::new();
    
    for i in 1..grid.len()-1{
        for j in 1..grid[i].len()-1{
            if grid[i][j] == crate::gamemap::TileType::Floor{
                
            }
        }
    }
    //скипаем пустые места и стены, находим клетку пола, проверяем 4 направления, добавляем, подходит подход с листом смежности 
}
fn build_path() {
    //воссоздаем и возвращаем путь
}

fn pick_node() {
    //алгоритм выбора нода, который ближе всего к игроку
}
