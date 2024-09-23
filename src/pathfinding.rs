//A* Pathfinding for enemies
use crate::player::Player;
use bevy::prelude::*;

pub struct PathfindingPlugin;

impl Plugin for PathfindingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Graph::default());
        app.add_systems(PostStartup, create_new_graph)
            .add_systems(FixedUpdate, a_pathfinding);
    }
}
// структура для графа, ноды хранят в себе позицию и тип тайла, цена для поиска пути и путь в другой структуре
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

// Граф - хэшмапа массивов нодов, так называемый лист смежности, в каждом массиве - список нодов в которые можно прийти из ноды ((u16,u16) - индексы в LevelGenerator)
#[derive(Resource)]
pub struct Graph {
    adj_list: std::collections::HashMap<(u16, u16), Vec<Node>>,
}

impl Default for Graph {
    fn default() -> Graph {
        Graph {
            adj_list: std::collections::HashMap::new(),
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
    let i = vec.x / 32.;
    let j = vec.y / 32.; //мб рефакторинг?

    return slf.adj_list[&(i as u16, j as u16)][0].clone();
    //берем коорды, конвертим их в примерную вершину, в примерном векторе по идее - нужный нод нулевой, нужно посмотреть еще раз
}
// получение массива нодов, в функции удаляется нод с которым смежны остальные в массиве
fn get_list(slf: &Graph, vec: Vec2, node: Node) -> Vec<Node> {
    let i = vec.x / 32.;
    let j = vec.y / 32.;

    let mut ans = slf.adj_list[&(i as u16, j as u16)].clone();
    for k in 0..ans.len() {
        if ans[k].position == node.position {
            ans.remove(k);

            return ans;
        }
    }
    return ans;
}
//структура нодов с ценами и путем, из них создается двумерная матрица по которой ищем и составляем путь
#[derive(Clone)]
struct CostNode {
    cost: u16,
    path: std::collections::LinkedList<(u16, u16)>,
}
impl CostNode {
    fn new(cost_new: u16) -> Self {
        CostNode {
            cost: cost_new,
            path: std::collections::LinkedList::new(),
        }
    }
    fn change_cost(&mut self, cost_new: u16) {
        self.cost = cost_new;
    }
}
//система Pathifinding-а, самописный A* используя средства беви, перекидываю граф, очереди мобов и игрока, после чего ищу от позиций мобов путь до игрока
fn a_pathfinding(
    mut player_query: Query<&Transform, With<Player>>, //maybe use globalTransform?
    //    mob_query: // Query<(&Transform, With<Mob>)>, change when add mobs
    //    Vec<Vec2>,
    mut graph_search: ResMut<Graph>,
) {
    let mob_query: Vec<Vec2> = vec![Vec2::new(64., 64.), Vec2::new(96., 96.)]; //затычка
    for mob in mob_query {
        //получаем позицию игрока
        if let Ok(player) = player_query.get_single_mut() {
            //создаем нод где стоит моб
            let start_node = Node::new(crate::gamemap::TileType::Floor, Vec2::new(mob.x, mob.y));

            let mut field: Vec<Vec<CostNode>> = Vec::new();

            //задаем поле с ценами, ставим их как большое число, чтобы потом пересчитывать во время работы алгоритма
            for i in 0..9 {
                field.push(Vec::new());
                for _ in 0..10 {
                    field[i].push(CostNode::new(u8::MAX as u16));
                }
            }

            //делаем на основе позиции игрока goal_node
            let goal_node = get_node_where_object_is(
                &mut graph_search,
                &Vec2::new(player.translation.x, player.translation.y),
            );

            //задаем нод где стоит моб нулевой ценой
            field[(mob.x / 32.) as usize][(mob.y / 32.) as usize].change_cost(0);

            //создаем хэшмапы для пройденных нодов и доступных
            let mut reachable = std::collections::HashMap::new();
            let mut explored = std::collections::HashMap::new();

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

                //нужно придумать что делать если нашли целевой нод, в теории путь можно сохранять в структуру к мобам? или в ресурс
                if node == goal_node {
                    field[(goal_node.position.x / 32.) as usize]
                        [(goal_node.position.y / 32.) as usize]
                        .path
                        .push_back((
                            (goal_node.position.x / 32.) as u16,
                            (goal_node.position.y / 32.) as u16,
                        ));
                    break;
                    /*
                    return build_path(
                        field[(goal_node.position.x / 32.) as usize]
                            [(goal_node.position.y / 32.) as usize]
                            .clone(),
                    );*/
                }

                //записываем что мы прошли нод и убираем из доступных
                reachable.remove(&(node.position.x as usize, node.position.y as usize));
                explored.insert(
                    (node.position.x as usize, node.position.y as usize),
                    node.clone(),
                );

                //берем ноды в которые можно прийти
                let new_reachable_potential =
                    get_list(&mut graph_search, node.position, node.clone());
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
                    if field[(node.position.x / 32.) as usize][(node.position.y / 32.) as usize]
                        .cost
                        + 1
                        < field[(adjacent.position.x / 32.) as usize]
                            [(adjacent.position.y / 32.) as usize]
                            .cost
                    {
                        field[(adjacent.position.x / 32.) as usize]
                            [(adjacent.position.y / 32.) as usize]
                            .path = field[(node.position.x / 32.) as usize]
                            [(node.position.y / 32.) as usize]
                            .path
                            .clone();

                        let k = (
                            (node.position.x / 32.) as u16,
                            (node.position.y / 32.) as u16,
                        );

                        field[(adjacent.position.x / 32.) as usize]
                            [(adjacent.position.y / 32.) as usize]
                            .path
                            .push_back(k);

                        field[(adjacent.position.x / 32.) as usize]
                            [(adjacent.position.y / 32.) as usize]
                            .cost = field[(node.position.x / 32.) as usize]
                            [(node.position.y / 32.) as usize]
                            .cost
                            + 1;
                    }
                }
            }
            //     return Vec::new(); //can't use return
        }
    }
}

//система создания графа как листа смежности, граф идет как ресурс, мб стоит проверить, что с ним все нормально и он меняется и сохраняется
fn create_new_graph(room: Res<crate::gamemap::LevelGenerator>, mut graph_search: ResMut<Graph>) {
    //берем мапу с LevelGenerator, потом надо будет вынести ее оттуда в отдельную структуру
    let grid = room.grid.clone();

    for i in 1..grid.len() - 1 {
        for j in 1..grid[i].len() - 1 {
            //когда находим тайл пола, создаем новый массив в хэшмапе с этим нодом внутри, надо рефакторить, мб не стоит добавлять нод
            if grid[i][j] == crate::gamemap::TileType::Floor {
                add_node_list(
                    &mut graph_search,
                    (i as u16, j as u16),
                    Node::new(
                        crate::gamemap::TileType::Floor,
                        Vec2::new(i as f32 * 32., j as f32 * 32.),
                    ),
                );
                //otdelnyy func
                let mut sub_grid: Vec<Vec<u8>> = vec![vec![0, 0, 0], vec![0, 0, 0], vec![0, 0, 0]];
                let mut sub_grid_i = 0;
                let mut sub_grid_j = 0;

                if (grid[i][j - 1] == crate::gamemap::TileType::Wall)
                    & (grid[i - 1][j] == crate::gamemap::TileType::Wall)
                {
                    sub_grid[0][0] += 1;
                    sub_grid[0][1] += 1;
                    sub_grid[1][0] += 1;
                }
                if (grid[i - 1][j] == crate::gamemap::TileType::Wall)
                    & (grid[i][j + 1] == crate::gamemap::TileType::Wall)
                {
                    sub_grid[0][2] += 1;
                    sub_grid[0][1] += 1;
                    sub_grid[1][2] += 1;
                }

                if (grid[i][j - 1] == crate::gamemap::TileType::Wall)
                    & (grid[i + 1][j] == crate::gamemap::TileType::Wall)
                {
                    sub_grid[2][0] += 1;
                    sub_grid[1][0] += 1;
                    sub_grid[2][1] += 1;
                }
                if (grid[i + 1][j] == crate::gamemap::TileType::Wall)
                    & (grid[i][j + 1] == crate::gamemap::TileType::Wall)
                {
                    sub_grid[2][2] += 1;
                    sub_grid[2][1] += 1;
                    sub_grid[1][2] += 1;
                }
                //otdelnyy func vinesti

                //цикл где мы добавляем в массив соседние ноды, если к ним можно пройти (не стены и тайл не закрыт стенами по диагонали)
                for k in i - 1..i + 2 {
                    for m in j - 1..j + 2 {
                        //смотрим, если стены закрывают диагональ, то не добавляем их в граф смежности
                        if (k == i) & (m == j) {
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
                        if grid[k][m] == crate::gamemap::TileType::Floor {
                            add_node_to_list(
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
}
/* ФУНКЦИЯ ПОСТРОЕНИЯ ПУТИ, НУЖНО РЕШИТЬ ЧТО С НЕЙ ДЕЛАТЬ, КУДА СОХРАНЯТЬ ПУТЬ
fn build_path(node: CostNode) -> Vec<(u16, u16)> {
    let mut path: Vec<(u16, u16)> = Vec::new();
    for i in node.path {
        path.push(i);
    }
    return path;
}
*/
//функция выбора нода, эвристика, учитывает только расстояние до цели и длину пути,
//можно добавить что-то, например кастомные тайлы пола, по которым не будут хотеть ходить мобы
fn pick_node(reachable: Vec<Node>, goal_node: Node, cost_grid: Vec<Vec<CostNode>>) -> Node {
    let mut min_cost: usize = usize::MAX;
    let mut best_node: Node = Node::new(crate::gamemap::TileType::Floor, Vec2::new(0., 0.));

    for node in reachable {
        //цена пути (учет кол-ва пройденных нодов, можно здесь подумать покрутить параметры)
        let cost_to_start: usize = 10
            * cost_grid[(node.position.x / 32.) as usize][(node.position.y / 32.) as usize].cost
                as usize
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
fn distance(node1: &Node, node2: &Node) -> usize {
    return std::cmp::min(
        (node2.position.x - node1.position.x).abs() as usize
            + (node2.position.y - node1.position.y).abs() as usize,
        ((node1.position.x - node2.position.x).powf(2.)
            + (node1.position.y - node2.position.y).powf(2.))
        .sqrt() as usize,
    );
}