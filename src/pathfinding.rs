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
pub struct Node {
    tile_type: crate::gamemap::TileType,
    position: Vec2,
}

impl Node {
    pub fn new(tile_type_new: crate::gamemap::TileType, position_new: Vec2) -> Self {
        Node {
            tile_type: tile_type_new,
            position: position_new,
        }
    }
}

#[derive(Resource)] // change and add methods to res
pub struct Graph {
    adj_list: std::collections::HashMap<(u16, u16), Vec<Node>>, //Я ГЕНИЙ, нужно отрефакторить посмотреть и прописать коменты
}

impl Default for Graph {
    fn default() -> Graph {
        Graph {
            adj_list: std::collections::HashMap::new(),
        }
    }
}
impl Graph {
    fn new() -> Graph {
        Graph {
            adj_list: std::collections::HashMap::new(),
        }
    }
}

fn Add_Node_List(slf: &mut Graph, tup: (u16, u16), node: Node) {
    let mut vec = Vec::new();
    vec.push(node);
    slf.adj_list.insert(tup, vec);
}

fn Add_Node_To_List(slf: &mut Graph, tup: (u16, u16), node: Node) {
    let mut vec = slf.adj_list[&tup].clone();
    vec.push(node);
    slf.adj_list.insert(tup, vec);
}

fn Get_Node_Where_Object_is(slf: &mut Graph, vec: &Vec2) -> Node {
    let i = vec.x / 32.;
    let j = vec.y / 32.; //мб рефакторинг?

    return slf.adj_list[&(i as u16, j as u16)][0].clone();
    //берем коорды, конвертим их в примерную вершину, в примерном векторе по идее - нужный нод нулевой, нужно посмотреть еще раз
}

fn Get_List(slf: &Graph, vec: Vec2, node: Node) -> Vec<Node> {
    let i = vec.x / 32.;
    let j = vec.y / 32.;

    let mut ans = slf.adj_list[&(i as u16, j as u16)].clone();
    for k in 0..ans.len() {
        if (ans[k].position == node.position) {
            ans.remove(k);
            return ans;
        }
    }
    return ans;
}

#[derive(Clone)]
struct cost_node {
    cost: u16,
    path: std::collections::LinkedList<(u16, u16)>,
}
impl cost_node {
    fn new(cost_new: u16) -> Self {
        cost_node {
            cost: cost_new,
            path: std::collections::LinkedList::new(),
        }
    }
    fn change_cost(&mut self, cost_new: u16) {
        self.cost = cost_new;
    }
    fn add_node(&mut self, new_node: (u16, u16)) {
        self.path.push_back(new_node);
    }
}

fn a_pathfinding(
    mut player_query: Query<&Transform, With<Player>>, //maybe use globalTransform?
    mut mob_query: Query<(&Transform, With<Mob>)>,
    mut graph_search: ResMut<Graph>,
) {
    //для каждого моба делать поиск пути?
    let start_node = Node::new(
        crate::gamemap::TileType::Floor, //надо искать нод в графе как-то сделать функцию чтобы искать ближайший нод?
        Vec2::new(mob.x, mob.y),
    );
    let mut field: Vec<Vec<cost_node>> = Vec::new();
    //9x10
    for i in 0..9 {
        field.push(Vec::new());
        for j in 0..10 {
            field[i].push(cost_node::new(u8::MAX as u16)); //будто в части где смотрим по стоимости надо смотреть по полю с ценами и путями
        }
    }

    let goal_node = Get_Node_Where_Object_is(&mut graph_search, &Vec2::new(player.x, player.y));

    field[(mob.x / 32.) as usize][(mob.y / 32.) as usize].change_cost(0);

    let mut reachable = std::collections::HashMap::new();
    let mut explored = std::collections::HashMap::new();

    reachable.insert(
        (
            start_node.position.x as usize,
            start_node.position.y as usize,
        ),
        start_node,
    );
    while reachable.len() > 0 {
        let mut node: Node = pick_node(
            reachable.values().cloned().collect(),
            goal_node.clone(),
            field.clone(),
        );

        if node == goal_node {
            field[(goal_node.position.x / 32.) as usize][(goal_node.position.y / 32.) as usize]
                .path
                .push_back((
                    (goal_node.position.x / 32.) as u16,
                    (goal_node.position.y / 32.) as u16,
                ));
            return build_path(
                field[(goal_node.position.x / 32.) as usize][(goal_node.position.y / 32.) as usize]
                    .clone(),
            );
        }

        reachable.remove(&(node.position.x as usize, node.position.y as usize)); //???? maybe another func or change vec to hashmap
        explored.insert(
            (node.position.x as usize, node.position.y as usize),
            node.clone(),
        );

        let new_reachable_potential = Get_List(&mut graph_search, node.position, node.clone());
        //забыл убирать ноды уже изученые
        let mut new_reachable: Vec<Node> = Vec::new();
        for potential in 0..new_reachable_potential.len() {
            match explored.get(&(
                new_reachable_potential[potential].position.x as usize,
                new_reachable_potential[potential].position.y as usize,
            )) {
                Some(adk) => {}
                None => {
                    new_reachable.push(new_reachable_potential[potential].clone());
                }
            }
        }
        for adjacent in new_reachable {
            match reachable.get(&(adjacent.position.x as usize, adjacent.position.y as usize)) {
                Some(adj) => {}
                None => {
                    reachable.insert(
                        (adjacent.position.x as usize, adjacent.position.y as usize),
                        adjacent.clone(),
                    );
                }
            }

            if field[(node.position.x / 32.) as usize][(node.position.y / 32.) as usize].cost + 1
                < field[(adjacent.position.x / 32.) as usize][(adjacent.position.y / 32.) as usize]
                    .cost
            {
                field[(adjacent.position.x / 32.) as usize][(adjacent.position.y / 32.) as usize]
                    .path = field[(node.position.x / 32.) as usize]
                    [(node.position.y / 32.) as usize]
                    .path
                    .clone();
                let k = (
                    (node.position.x / 32.) as u16,
                    (node.position.y / 32.) as u16,
                );
                field[(adjacent.position.x / 32.) as usize][(adjacent.position.y / 32.) as usize]
                    .path
                    .push_back(k);
                field[(adjacent.position.x / 32.) as usize][(adjacent.position.y / 32.) as usize]
                    .cost =
                    field[(node.position.x / 32.) as usize][(node.position.y / 32.) as usize].cost
                        + 1;
            }
        }
    }
    return Vec::new(); //can't use return
}

fn create_new_graph(room: ResMut<crate::gamemap::LevelGenerator>, mut graph_search: ResMut<Graph>) {
    let grid = room.grid.clone(); //мб рефакторинг?
    for i in 1..grid.len() - 1 {
        for j in 1..grid[i].len() - 1 {
            if grid[i][j] == crate::gamemap::TileType::Floor {
                //NEED TO TRANSFER GRAPH TO MAIN FUNC, MAYBE CREATE AS RESOURCE? NEED TO CHECK CORNERS, IT SHOULD GO THROUGH THEM
                Add_Node_List(
                    &mut graph_search,
                    (i as u16, j as u16),
                    Node::new(crate::gamemap::TileType::Floor, Vec2::new(i as f32 * 32., j as f32 * 32.)),
                );
                //otdelnyy func
                let mut sub_grid: Vec<Vec<u8>> = vec![vec![0, 0, 0], vec![0, 0, 0], vec![0, 0, 0]];
                let mut sub_grid_i = 0;
                let mut sub_grid_j = 0;

                if (grid[i][j - 1] == crate::gamemap::TileType::Wall) & (grid[i - 1][j] == crate::gamemap::TileType::Wall) {
                    sub_grid[0][0] += 1;
                    sub_grid[0][1] += 1;
                    sub_grid[1][0] += 1;
                }
                if (grid[i - 1][j] == crate::gamemap::TileType::Wall) & (grid[i][j + 1] == crate::gamemap::TileType::Wall) {
                    sub_grid[0][2] += 1;
                    sub_grid[0][1] += 1;
                    sub_grid[1][2] += 1;
                }

                if (grid[i][j - 1] == crate::gamemap::TileType::Wall) & (grid[i + 1][j] == crate::gamemap::TileType::Wall) {
                    sub_grid[2][0] += 1;
                    sub_grid[1][0] += 1;
                    sub_grid[2][1] += 1;
                }
                if (grid[i + 1][j] == crate::gamemap::TileType::Wall) & (grid[i][j + 1] == crate::gamemap::TileType::Wall) {
                    sub_grid[2][2] += 1;
                    sub_grid[2][1] += 1;
                    sub_grid[1][2] += 1;
                }
                //otdelnyy func vinesti

                for k in i - 1..i + 2 {
                    for m in j - 1..j + 2 {
                        if ((k == i) & (m == j)) {
                            continue;
                        }
                        if (k == i - 1) & (m == j - 1) {
                        } else {
                            sub_grid_j += 1;
                        }

                        sub_grid_i = (sub_grid_i + sub_grid_j.clone() / 3) % 3;
                        sub_grid_j = sub_grid_j % 3;

                        if (sub_grid[sub_grid_i][sub_grid_j] > 0) {
                            continue;
                        }
                        if grid[k][m] == crate::gamemap::TileType::Floor {
                            Add_Node_To_List(
                                &mut graph_search,
                                (i as u16, j as u16),
                                Node::new(
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
    return graph_search;
}
fn build_path(node: cost_node) -> Vec<(u16, u16)> {
    let mut path: Vec<(u16, u16)> = Vec::new();
    for i in node.path {
        path.push(i);
    }
    //мб рефакторинг?
    return path;
}

fn pick_node(reachable: Vec<Node>, goal_node: Node, cost_grid: Vec<Vec<cost_node>>) -> Node {
    //doesn't work
    let mut min_cost: usize = usize::MAX;
    let mut best_node: Node = Node::new(crate::gamemap::TileType::Floor, Vec2::new(0., 0.));

    for node in reachable {
        let cost_to_start: usize = 10
            * cost_grid[(node.position.x / 32.) as usize][(node.position.y / 32.) as usize].cost
                as usize
            + 100; //change
        let cost_node_to_goal: usize = distance(&node, &goal_node);
        let total_cost: usize = cost_to_start + cost_node_to_goal;

        if min_cost > total_cost {
            min_cost = total_cost;
            best_node = node;
        }
    } //мб рефакторинг?
    return best_node;
}

fn distance(node1: &Node, node2: &Node) -> usize {
    return std::cmp::min(
        (node2.position.x - node1.position.x).abs() as usize
            + (node2.position.y - node1.position.y).abs() as usize,
        ((node1.position.x - node2.position.x).powf(2.)
            + (node1.position.y - node2.position.y).powf(2.))
        .sqrt() as usize,
    );
}
