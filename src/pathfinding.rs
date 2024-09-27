use std::collections::{HashMap, LinkedList};

//A* Pathfinding for enemies
use crate::{
    gamemap::{LevelGenerator, TileType, ROOM_SIZE},
    mob::Mob,
    player::Player,
    GameState,
};
use bevy::prelude::*;

pub struct PathfindingPlugin;

impl Plugin for PathfindingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Graph::default());
        app.add_systems(
            OnEnter(GameState::Loading),
            create_new_graph.after(crate::gamemap::spawn_map),
        )
        .add_systems(Update, a_pathfinding.run_if(in_state(GameState::InGame)));
    }
}
// структура для графа, ноды хранят в себе позицию и тип тайла, цена для поиска пути и путь в другой структуре
#[derive(Clone, PartialEq)]
pub struct Node {
    tile_type: crate::gamemap::TileType,
    position: Vec2,
}

impl Node {
    pub fn new(tile_type_new: TileType, position_new: Vec2) -> Self {
        Node {
            tile_type: tile_type_new,
            position: position_new,
        }
    }
}

// Граф - хэшмапа массивов нодов, так называемый лист смежности, в каждом массиве - список нодов в которые можно прийти из ноды ((u16,u16) - индексы в LevelGenerator)
#[derive(Resource)]
pub struct Graph {
    adj_list: HashMap<(u16, u16), Vec<Node>>,
}

impl Default for Graph {
    fn default() -> Graph {
        Graph {
            adj_list: HashMap::new(),
        }
    }
}
//добавляем массив в хэшмапу, добавляя саму нод внутрь, для будущего pathfinding-а,
//возможно стоит убрать добавление самого нода, нужно отрефакторить, вроде бы не используется
fn add_node_list(slf: &mut Graph, tup: (u16, u16), node: Node) {
    let mut vec = Vec::new();
    vec.push(node);
    slf.adj_list.insert(tup, vec);
}
//добавляем ноду по индексам в граф
fn add_node_to_list(slf: &mut Graph, tup: (u16, u16), node: Node) {
    let mut vec = slf.adj_list[&tup].clone();
    vec.push(node);
    slf.adj_list.insert(tup, vec);
}

//получение ноды где находится объект с помощью математики
fn get_node_where_object_is(slf: &mut Graph, vec: &Vec2) -> Node {
    let (i, j) = safe_get_pos(*vec, slf);
    for k in slf.adj_list[&(i, j)].clone() {
        if k.position == Vec2::new(vec.x, vec.y) {
            return k.clone();
        }
    }
    return slf.adj_list[&(i, j)][0].clone();
    //берем коорды, конвертим их в примерную вершину, в примерном векторе по идее - нужный нод нулевой, нужно посмотреть еще раз
}
//безопасное получение координат для нодов, если не существует узла по заданым координатам - смотрим, прошли ли мы достаточно чтобы встать в следующий нод
//нужно отдебажить, может происходить попадание в другой нод
fn safe_get_pos(vec: Vec2, slf: &Graph) -> (u16, u16) {
    let mut best = Vec2::new(0., 0.);
    let mut range: usize = u32::MAX as usize;
    for i in slf.adj_list.clone() {
        let temp_range = distance(
            &Node::new(TileType::Floor, vec),
            &Node::new(
                TileType::Floor,
                Vec2::new(
                    (i.0.0 as f32 * (ROOM_SIZE) as f32 + (ROOM_SIZE / 2) as f32).floor(),
                    (i.0.1 as f32 * (ROOM_SIZE) as f32 + (ROOM_SIZE / 2) as f32).floor(),
                ),
            ),
        );
        if (range > temp_range) {
            range = temp_range;
            best = Vec2::new(i.0.0 as f32, i.0.1 as f32);
        }
    }
    return (best.x.floor() as u16, best.y.floor() as u16);
}
// получение массива нодов, в функции удаляется нод с которым смежны остальные в массиве
fn get_list(slf: &Graph, vec: Vec2) -> Vec<Node> {
    let (i, j) = safe_get_pos(vec, slf);

    let ans = slf.adj_list[&(i, j)].clone();
    /*     for k in 0..ans.len() {
        if ans[k].position == node.position {
            ans.remove(k);

            return ans;
        }
    }*/
    return ans;
}
//структура нодов с ценами и путем, из них создается двумерная матрица по которой ищем и составляем путь
#[derive(Clone)]
struct CostNode {
    cost: u16,
    path: LinkedList<(u16, u16)>,
}
impl CostNode {
    fn new(cost_new: u16) -> Self {
        CostNode {
            cost: cost_new,
            path: LinkedList::new(),
        }
    }
    fn change_cost(&mut self, cost_new: u16) {
        self.cost = cost_new;
    }
}

//система Pathifinding-а, самописный A* используя средства беви, перекидываю граф, очереди мобов и игрока, после чего ищу от позиций мобов путь до игрока
fn a_pathfinding(
    player_query: Query<&Transform, With<Player>>, //don't use globalTransform, please
    mut mob_query: Query<(&Transform, &mut Mob), Without<Player>>,
    mut graph_search: ResMut<Graph>,
) {
    for (mob_transform, mut mob) in mob_query.iter_mut() {
        if mob.needs_path {
            //получаем позицию игрока
            if let Ok(player) = player_query.get_single() {
                //создаем нод где стоит моб
                let start_node = Node::new(
                    TileType::Floor,
                    Vec2::new(
                        mob_transform.translation.x.floor(),
                        mob_transform.translation.y.floor(),
                    ),
                );

                let mut field: Vec<Vec<CostNode>> = Vec::new();

                //задаем поле с ценами, ставим их как большое число, чтобы потом пересчитывать во время работы алгоритма
                for i in 0..ROOM_SIZE {
                    field.push(Vec::new());
                    for _ in 0..ROOM_SIZE {
                        field[i as usize].push(CostNode::new(u8::MAX as u16));
                    }
                }

                //делаем на основе позиции игрока goal_node
                let goal_node = get_node_where_object_is(
                    &mut graph_search,
                    &Vec2::new(player.translation.x.floor(), player.translation.y.floor()),
                );

                //задаем нод где стоит моб нулевой ценой
                field[((mob_transform.translation.x.floor() as usize - (ROOM_SIZE / 2) as usize) / (ROOM_SIZE) as usize)][((mob_transform.translation.y.floor() as usize- (ROOM_SIZE / 2) as usize)/ (ROOM_SIZE) as usize)].change_cost(0);

                //создаем хэшмапы для пройденных нодов и доступных
                let mut reachable = HashMap::new();
                let mut explored = HashMap::new();

                //добавляем нод с мобом в хэшмапу для доступных нодов
                reachable.insert(
                    (
                        start_node.position.x as usize,
                        start_node.position.y as usize,
                    ),
                    start_node,
                );
                while reachable.len() > 0 {
                    //функция выбора нода описана ниже
                    let node: Node = pick_node(
                        reachable.values().cloned().collect(),
                        goal_node.clone(),
                        field.clone(),
                    );

                    //нужно придумать что делать если нашли целевой нод, в теории путь можно сохранять в структуру к мобам?
                    if node == goal_node {
                        field[((goal_node.position.x - (ROOM_SIZE / 2) as f32) / (ROOM_SIZE) as f32) as usize]
                            [((goal_node.position.y - (ROOM_SIZE / 2) as f32) / (ROOM_SIZE) as f32) as usize]
                            .path
                            .push_back((
                                ((goal_node.position.x - (ROOM_SIZE / 2) as f32) / (ROOM_SIZE) as f32) as u16,
                                ((goal_node.position.y - (ROOM_SIZE / 2) as f32) / (ROOM_SIZE) as f32) as u16,
                            ));

                        mob.path = build_path(
                            field[((goal_node.position.x - (ROOM_SIZE / 2) as f32) / (ROOM_SIZE) as f32) as usize]
                                [((goal_node.position.y - (ROOM_SIZE / 2) as f32) / (ROOM_SIZE) as f32) as usize]
                                .clone(),
                        );

                        break;
                    }

                    //записываем что мы прошли нод и убираем из доступных
                    reachable.remove(&(node.position.x as usize, node.position.y as usize));
                    explored.insert(
                        (node.position.x as usize, node.position.y as usize),
                        node.clone(),
                    );

                    //берем ноды в которые можно прийти
                    let new_reachable_potential = get_list(&mut graph_search, node.position);
                    let mut new_reachable: Vec<Node> = Vec::new();

                    //записываем ноды, которые не были еще посещены и записываем их в new_reachable
                    for potential in 0..new_reachable_potential.len() {
                        match explored.get(&(
                            new_reachable_potential[potential].position.x as usize,
                            new_reachable_potential[potential].position.y as usize,
                        )) {
                            Some(_) => {}
                            None => {
                                new_reachable.push(new_reachable_potential[potential].clone());
                            }
                        }
                    }

                    //проходим по new_reachable, если нод уже есть в reachable, не добавляем
                    for adjacent in new_reachable {
                        match reachable
                            .get(&(adjacent.position.x as usize, adjacent.position.y as usize))
                        {
                            Some(_) => {}
                            None => {
                                reachable.insert(
                                    (adjacent.position.x as usize, adjacent.position.y as usize),
                                    adjacent.clone(),
                                );
                            }
                        }

                        //если цена нода больше чем цена текущего нода + 1, меняем кост в field, перерасчитываем путь
                        if field[((node.position.x - (ROOM_SIZE / 2) as f32) / (ROOM_SIZE) as f32) as usize]
                            [((node.position.y - (ROOM_SIZE / 2) as f32) / (ROOM_SIZE) as f32) as usize]
                            .cost
                            + 1
                            < field[((adjacent.position.x - (ROOM_SIZE / 2) as f32) / (ROOM_SIZE ) as f32) as usize]
                                [((adjacent.position.y - (ROOM_SIZE / 2) as f32) / (ROOM_SIZE) as f32) as usize]
                                .cost
                        {
                            field[((adjacent.position.x - (ROOM_SIZE / 2) as f32) / (ROOM_SIZE) as f32) as usize]
                                [((adjacent.position.y - (ROOM_SIZE / 2) as f32) / (ROOM_SIZE) as f32) as usize]
                                .path = field[((node.position.x - (ROOM_SIZE / 2) as f32) / (ROOM_SIZE) as f32) as usize]
                                [((node.position.y - (ROOM_SIZE / 2) as f32) / (ROOM_SIZE) as f32) as usize]
                                .path
                                .clone();

                            let k = (
                                ((node.position.x - (ROOM_SIZE / 2) as f32) / (ROOM_SIZE) as f32) as u16,
                                ((node.position.y - (ROOM_SIZE / 2) as f32) / (ROOM_SIZE) as f32) as u16,
                            );

                            field[((adjacent.position.x - (ROOM_SIZE / 2) as f32) / (ROOM_SIZE) as f32) as usize]
                                [((adjacent.position.y - (ROOM_SIZE / 2) as f32) / (ROOM_SIZE) as f32) as usize]
                                .path
                                .push_back(k);

                            field[((adjacent.position.x - (ROOM_SIZE / 2) as f32) / (ROOM_SIZE) as f32) as usize]
                                [((adjacent.position.y - (ROOM_SIZE / 2) as f32) / (ROOM_SIZE) as f32) as usize]
                                .cost = field[((node.position.x - (ROOM_SIZE / 2) as f32) / (ROOM_SIZE) as f32) as usize]
                                [((node.position.y - (ROOM_SIZE / 2) as f32) / (ROOM_SIZE) as f32) as usize]
                                .cost
                                + 1;
                        }
                    }
                }
            }
        }
    }
}

//система создания графа как листа смежности, граф идет как ресурс, мб стоит проверить, что с ним все нормально и он меняется и сохраняется
fn create_new_graph(
    room: Res<LevelGenerator>,
    mut graph_search: ResMut<Graph>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    //берем мапу с LevelGenerator, потом надо будет вынести ее оттуда в отдельную структуру
    let grid = room.grid.clone();

    for i in 1..grid.len() - 1 {
        for j in 1..grid[i].len() - 1 {
            //когда находим тайл пола, создаем новый массив в хэшмапе с этим нодом внутри, надо рефакторить, мб не стоит добавлять нод
            if grid[i][j] == TileType::Floor {
                add_node_list(
                    &mut graph_search,
                    (i as u16, j as u16),
                    Node::new(
                        TileType::Floor,
                        Vec2::new(
                            i as f32 * (ROOM_SIZE) as f32 + (ROOM_SIZE / 2) as f32,
                            j as f32 * (ROOM_SIZE) as f32 + (ROOM_SIZE / 2) as f32,
                        ),
                    ),
                );
                //otdelnyy func
                let mut sub_grid: Vec<Vec<u8>> = vec![vec![0, 0, 0], vec![0, 0, 0], vec![0, 0, 0]];
                let mut sub_grid_i = 0;
                let mut sub_grid_j = 0;

                if grid[i][j - 1] == TileType::Wall {
                    sub_grid[0][0] += 1;
                    sub_grid[1][0] += 1;
                    sub_grid[2][0] += 1;
                }
                if grid[i][j + 1] == TileType::Wall {
                    sub_grid[0][2] += 1;
                    sub_grid[1][2] += 1;
                    sub_grid[2][2] += 1;
                }
                if grid[i - 1][j] == TileType::Wall {
                    sub_grid[0][0] += 1;
                    sub_grid[0][1] += 1;
                    sub_grid[0][2] += 1;
                }
                if grid[i + 1][j] == TileType::Wall {
                    sub_grid[2][0] += 1;
                    sub_grid[2][1] += 1;
                    sub_grid[2][2] += 1;
                }
                //otdelnyy func vinesti

                //цикл где мы добавляем в массив соседние ноды, если к ним можно пройти (не стены и тайл не закрыт стенами по диагонали)
                for k in i - 1..i + 2 {
                    for m in j - 1..j + 2 {
                        //смотрим, если стены закрывают диагональ, то не добавляем их в граф смежности
                        if (k == i) & (m == j) {
                            sub_grid_j += 1;
                            continue;
                        }
                        if (k == i - 1) & (m == j - 1) {
                        } else {
                            sub_grid_j += 1;
                        }

                        sub_grid_i = (sub_grid_i + sub_grid_j.clone() / 3) % 3;
                        sub_grid_j = sub_grid_j % 3;

                        if sub_grid[sub_grid_i][sub_grid_j] > 0 {
                            continue;
                        }

                        //добавляем в список ноду если она тайл пола
                        if grid[k][m] == TileType::Floor {
                            add_node_to_list(
                                &mut graph_search,
                                (i as u16, j as u16),
                                Node::new(
                                    TileType::Floor,
                                    Vec2::new(
                                        k as f32 * (ROOM_SIZE) as f32 + (ROOM_SIZE / 2) as f32,
                                        m as f32 * (ROOM_SIZE) as f32 + (ROOM_SIZE / 2) as f32,
                                    ),
                                ),
                            );
                        }
                    }
                }
            }
        }
    }
    print!("Graph is generated");
    game_state.set(GameState::InGame);
}
// ФУНКЦИЯ ПОСТРОЕНИЯ ПУТИ, НУЖНО РЕШИТЬ ЧТО С НЕЙ ДЕЛАТЬ, КУДА СОХРАНЯТЬ ПУТЬ
fn build_path(node: CostNode) -> Vec<(u16, u16)> {
    let mut path: Vec<(u16, u16)> = Vec::new();
    for i in node.path {
        path.push(i);
    }
    path.remove(0); // удаляем точку, в которой уже стоит моб на момент создания пути
    return path;
}

//функция выбора нода, эвристика, учитывает только расстояние до цели и длину пути,
//можно добавить что-то, например кастомные тайлы пола, по которым не будут хотеть ходить мобы
fn pick_node(reachable: Vec<Node>, goal_node: Node, cost_grid: Vec<Vec<CostNode>>) -> Node {
    let mut min_cost: usize = usize::MAX;
    let mut best_node: Node = Node::new(TileType::Floor, Vec2::new(0., 0.));

    for node in reachable {
        //цена пути (учет кол-ва пройденных нодов, можно здесь подумать покрутить параметры)
        let cost_to_start: usize = 10
            * cost_grid[((node.position.x - (ROOM_SIZE / 2) as f32) / (ROOM_SIZE) as f32) as usize]
                [((node.position.y - (ROOM_SIZE / 2) as f32) / (ROOM_SIZE) as f32) as usize]
                .cost as usize
            + 100;

        //добавление цены за дистанцию до нода
        let cost_node_to_goal: usize = distance(&node, &goal_node);
        let total_cost: usize = cost_to_start + cost_node_to_goal;

        if min_cost > total_cost {
            min_cost = total_cost;
            best_node = node;
        }
    }
    return best_node;
}

//функция расчета дистанции для функции выбора нода
fn distance(node1: &Node, node2: &Node) -> usize { // maybe this comment back
    return std::cmp::min(
        (node2.position.x - node1.position.x).abs() as usize
            + (node2.position.y - node1.position.y).abs() as usize,
        ((node1.position.x - node2.position.x).powf(2.)
            + (node1.position.y - node2.position.y).powf(2.))
        .sqrt() as usize);
}
