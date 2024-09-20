//A* Pathfinding for enemies
use crate::player::Player;
use bevy::prelude::*;
pub struct PathfindingPlugin;

impl Plugin for PathfindingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Graph>();
        app.add_systems(Startup, create_new_graph);
        app.add_systems(Update, a_pathfinding);
    }
}

#[derive(Clone, PartialEq)]
enum Address {
    Address(Box<Node>),
    Nil,
}

#[derive(Clone, PartialEq)]
pub struct Node {
    cost: i32,
    prev_node: Address,
    tile_type: crate::gamemap::TileType,
    position: Vec2,
}
#[derive(Resource)] // change and add methods to res
pub struct Graph {
    adj_list: std::collections::HashMap<(u16, u16), Vec<Node>>,//Я ГЕНИЙ, нужно отрефакторить посмотреть и прописать коменты
    }

impl Default for Graph {
    fn default() -> Graph {
        Graph {
            adj_list: std::collections::HashMap::new(),
        }
    }
}
impl Graph {
    fn Add_Node_List(&mut self, tup: (u16, u16), node: Node) {
        let mut vec = Vec::new();
        vec.push(node);
        self.adj_list.insert(tup, vec);
    }
    fn Add_Node_To_List(&mut self, tup: (u16, u16), node: Node) {
        let mut vec = self.adj_list[&tup].clone();
        vec.push(node);
        self.adj_list.insert(tup, vec);
    }
    fn Get_Node_Where_Object_is(&mut self, vec: &Vec2) -> Node {
        let i = vec.x / 32.;
        let j = vec.y / 32.;//мб рефакторинг?

        return self.adj_list[&(i as u16, j as u16)][0].clone();
        //берем коорды, конвертим их в примерную вершину, в примерном векторе по идее - нужный нод нулевой, нужно посмотреть еще раз
    }
    fn Get_List(&mut self, vec: Vec2) -> Vec<Node>{
        let i = vec.x / 32.;
        let j = vec.y / 32.;

        return self.adj_list[&(i as u16, j as u16)].clone();
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
    mut player_query: Query<&Transform, With<Player>>, //maybe use globalTransform?
    mut mob_query: Query<(&Transform, With<Mob>)>,
    mut room: ResMut<crate::gamemap::LevelGenerator>,
    mut graph_search: ResMut<Graph>,
) {
    for i in &mob_query {
        //для каждого моба делать поиск пути?
        let mut start_node = Node::new(
            0,
            crate::gamemap::TileType::Floor, //надо искать нод в графе как-то сделать функцию чтобы искать ближайший нод?
            Vec2::new(i.transform.translation.x, i.transform.translation.y),
        );
        let mut goal_node: Node = Node::new(0,crate::gamemap::TileType::Floor,Vec2::new(0.,0.));
        if let Ok(player_transform) = player_query.get_single(){
            goal_node = graph_search.Get_Node_Where_Object_is(&Vec2::new(player_transform.translation.x,player_transform.translation.y));
        }
        let mut reachable = std::collections::HashMap::new();
        let mut explored = std::collections::HashMap::new();

        reachable.insert((start_node.position.x as usize,start_node.position.y as usize),start_node);

        while reachable.len() > 0 {
            let mut node: Node = pick_node(reachable.values().cloned().collect(), goal_node);
            
            if node == goal_node{
            //    return build_path(node);
            return;//can't use return
            }

            reachable.remove(&(node.position.x as usize, node.position.y as usize));//???? maybe another func or change vec to hashmap
            explored.insert((node.position.x as usize, node.position.y as usize), node);

            let new_reachable: Vec<Node> = Vec::new();
            new_reachable = graph_search.Get_List(node.position);

            for adjacent in new_reachable{
                match reachable.get(&(adjacent.position.x as usize, adjacent.position.y as usize)){
                    Some(adj) => reachable.insert((adjacent.position.x as usize, adjacent.position.y as usize), adjacent)
                };

                if node.cost + 1 < adjacent.cost{
                    adjacent.prev_node = Address::Address(Box::new(node));
                    adjacent.cost = node.cost+1;
                }
            }
        }
    }
    return;//can't use return
}

fn create_new_graph(room: ResMut<crate::gamemap::LevelGenerator>, mut graph_search: ResMut<Graph>) {
    let grid = room.grid.clone();//мб рефакторинг?
    for i in 1..grid.len() - 1 {
        for j in 1..grid[i].len() - 1 {
            if grid[i][j] == crate::gamemap::TileType::Floor {
                //NEED TO TRANSFER GRAPH TO MAIN FUNC, MAYBE CREATE AS RESOURCE?
                graph_search.Add_Node_List(
                    (i as u16, j as u16),
                    Node::new(
                        i32::MAX,
                        crate::gamemap::TileType::Floor,
                        Vec2::new(i as f32 * 32., j as f32 * 32.),
                    ),
                );
                for k in i - 1..i + 2 {
                    for m in j - 1..j + 2 {
                        if (grid[k][m] == crate::gamemap::TileType::Floor) & (k != i) & (m != j) {
                            graph_search.Add_Node_To_List(
                                (i as u16, j as u16),
                                Node::new(
                                    1,
                                    crate::gamemap::TileType::Floor,
                                    Vec2::new(k as f32 * 32., m as f32 * 32.),
                                ),
                            );
                        }
                    }
                }
            }
        }
    }
    //скипаем пустые места и стены, находим клетку пола, проверяем 8 направлений, добавляем, подходит подход с листом смежности
}
fn build_path(mut nod: Node) -> Vec<Node> {
    let mut path: Vec<Node> = Vec::new();
    loop {
        path.push(nod.clone());
        match nod.prev_node {
            Address::Address(ref mut next_address) => {
                nod = *next_address.clone();
            }
            Address::Nil => {
                break;
            }
        }
    }//мб рефакторинг?
    return path;
}

fn pick_node(reachable: Vec<Node>, goal_node: Node) -> Node {
    let mut min_cost: usize = usize::MAX;
    let mut best_node: Node = Node::new(-1, crate::gamemap::TileType::Floor, Vec2::new(0., 0.));

    for node in reachable {
        let cost_to_start: usize = node.cost as usize;
        let cost_node_to_goal: usize = distance(&node, &goal_node);
        let total_cost: usize = cost_to_start + cost_node_to_goal;

        if min_cost > total_cost {
            min_cost = total_cost;
            best_node = node;
        }
    }//мб рефакторинг?
    return best_node;
}

fn distance(node1: &Node, node2: &Node) -> usize {
    return ((node1.position.x - node2.position.x).powf(2.)
        + (node1.position.y - node2.position.y).powf(2.))
    .sqrt() as usize;
}
