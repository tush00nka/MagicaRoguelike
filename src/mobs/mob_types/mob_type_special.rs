//bundle for melee only mobs
use avian2d::prelude::*;
use bevy::prelude::*;
use seldom_state::prelude::*;

use crate::{
    elements::{ElementResistance, ElementType},
    exp_tank::ExpTank,
    health_tank::HealthTank,
    item::{Item, ItemType},
    mobs::mob::*,
    obstacles::*,
    pathfinding::Pathfinder,
    GameLayer,
};

#[derive(Component, Clone, Default)]
pub struct ObstacleRush;

#[derive(Component, Clone)]
pub struct ItemRush;

#[derive(PartialEq, Clone)]
pub enum ItemPicked {
    HPTank,
    EXPTank,
    Item,
    Obstacle,
    None,
}

#[derive(PartialEq, Clone, Component)]
pub enum ItemPickedFlag {
    Some(ItemPicked),
    None,
}

#[derive(PartialEq, Clone, Component)]
pub struct PickupItem {
    item_type: ItemPicked,
    item_name: Option<ItemType>,
}

#[derive(Component)]
pub enum OnDeathEffect {
    CircleAttack,
}

#[derive(Event)]
pub struct PushItemQueryEvent {
    pub thief_entity: Entity,
    pub item: PickupItem,
}

#[derive(Component)]
pub struct PickupItemQueue {
    item_queue: Vec<Option<PickupItem>>,
    amount_of_obstacles: u8,
}

impl PickupItemQueue {
    fn empty_queue(&mut self) {
        for i in 0..self.item_queue.len() {
            self.item_queue[i] = None;
        }
    }

    fn push_obstacle(&mut self) {
        self.amount_of_obstacles += 1;
        if self.amount_of_obstacles >= 8 {
            self.empty_queue();
        }
    }

    fn push_item(&mut self, item: PickupItem) {
        let len = self.item_queue.len();

        self.item_queue[len - 1] = None;
        for i in (1..len - 1).rev() {
            self.item_queue[i] = self.item_queue[i - 1].clone();
        }

        self.item_queue[0] = Some(item);
    }
}

impl Default for PickupItemQueue {
    fn default() -> Self {
        Self {
            amount_of_obstacles: 0,
            item_queue: vec![None, None, None, None, None],
        }
    }
}
#[derive(Bundle)]
pub struct ThiefBundle {
    mob_bundle: MobBundle,
    path_finder: Pathfinder,
    items: PickupItemQueue,
    on_death_attack: OnDeathEffect,
}

impl MobBundle {
    fn thief() -> Self {
        Self {
            phys_bundle: PhysicalBundle {
                collision_layers: CollisionLayers::new(
                    GameLayer::Enemy,
                    [
                        GameLayer::Wall,
                        GameLayer::Projectile,
                        GameLayer::Shield,
                        GameLayer::Friend,
                        GameLayer::Player,
                        GameLayer::Interactable,
                    ],
                ),
                ..default()
            },
            resistance: ElementResistance {
                elements: vec![
                    ElementType::Earth,
                    ElementType::Air,
                    ElementType::Fire,
                    ElementType::Water,
                ],
                resistance_percent: vec![30, 30, 30, 30, 0],
            },
            mob_type: MobType::Thief,
            loot: MobLoot { orbs: 1 },
            ..default()
        }
    }
}

impl Default for ThiefBundle {
    fn default() -> Self {
        Self {
            mob_bundle: MobBundle::thief(),
            path_finder: Pathfinder {
                speed: 3200.,
                ..default()
            },
            items: PickupItemQueue::default(),
            on_death_attack: OnDeathEffect::CircleAttack,
        }
    }
}

pub fn thief_collide(
    mut commands: Commands,
    mut thief_query: Query<(Entity, &mut PickupItemQueue, &CollidingEntities)>,
    interactable_query: Query<
        Entity,
        (
            Or<(With<Item>, With<Obstacle>, With<HealthTank>, With<ExpTank>)>,
            Without<Corpse>,
        ),
    >,
    mut item_push_ev: EventWriter<PushItemQueryEvent>,
    hp_tank_query: Query<&HealthTank>,
    exp_tank_query: Query<&ExpTank>,
    item_query: Query<&Item>,
) {
    for (mob_e, mut obstacle_count, colliding_e) in thief_query.iter_mut() {
        for interactable_e in interactable_query.iter() {
            if colliding_e.contains(&interactable_e) {
                let mut is_obstacle: bool = true;
                if hp_tank_query.contains(interactable_e) {
                    item_push_ev.send(PushItemQueryEvent {
                        thief_entity: mob_e,
                        item: PickupItem {
                            item_type: ItemPicked::HPTank,
                            item_name: None,
                        },
                    });

                    is_obstacle = !is_obstacle;
                }

                if exp_tank_query.contains(interactable_e) {
                    item_push_ev.send(PushItemQueryEvent {
                        thief_entity: mob_e,
                        item: PickupItem {
                            item_type: ItemPicked::EXPTank,
                            item_name: None,
                        },
                    });

                    is_obstacle = !is_obstacle;
                }

                if hp_tank_query.contains(interactable_e) {
                    let item_type = item_query.get(interactable_e).unwrap();
                    item_push_ev.send(PushItemQueryEvent {
                        thief_entity: mob_e,
                        item: PickupItem {
                            item_type: ItemPicked::Item,
                            item_name: Some(item_type.item_type),
                        },
                    });

                    is_obstacle = !is_obstacle;
                }

                if is_obstacle {
                    obstacle_count.push_obstacle();
                }

                commands.entity(interactable_e).despawn();
                commands.entity(mob_e).insert(Done::Success);
                break;
            }
        }
    }
}

pub fn item_queue_update(
    mut item_push_ev: EventReader<PushItemQueryEvent>,
    mut thief_query: Query<&mut PickupItemQueue>,
) {
    for ev in item_push_ev.read() {
        match thief_query.get_mut(ev.thief_entity) {
            Ok(mut val) => val.push_item(ev.item.clone()),
            Err(error) => {
                println!("{}", error);
                continue;
            }
        };
    }
}

pub fn nearest_interactable(
    In(entity): In<Entity>,
    transforms: Query<
        (Entity, &Transform),
        (
            Without<Corpse>,
            Or<(With<HealthTank>, With<ExpTank>, With<Item>, With<Obstacle>, With<PickupItemQueue>)>,
        ),
    >,
) -> Option<Option<Entity>> {
    
    if transforms.iter().len() == 0{
        return None;
    }

    let sorted_targets: Vec<(Entity, &Transform)> = transforms
        .iter()
        .sort_by::<&Transform>(|item1, item2| {
            item1
                .translation
                .distance(transforms.get(entity).unwrap().1.translation)
                .total_cmp(
                    &item2
                        .translation
                        .distance(transforms.get(entity).unwrap().1.translation),
                )
        })
        .collect();

    if sorted_targets.len() < 2 {
        return None;
    }

    let (nearest_target, _) = sorted_targets[1];

    return Some(Some(nearest_target));
}

pub fn pick_item_to_steal(
    In(entity): In<Entity>,
    steal_query: Query<&PickTargetForSteal>,
    hp_tank_query: Query<&HealthTank>,
    exp_tank_query: Query<&ExpTank>,
    obstacle_query: Query<&Obstacle, Without<Corpse>>,
    item_query: Query<&Item>,
) -> Result<i8, i8> {
    let Ok(steal_target) = steal_query.get(entity) else {
        return Err(ItemPicked::None as i8);
    };
    match steal_target.target {
        None => return Err(4),
        Some(target_e) => {
            if hp_tank_query.contains(target_e) {
                return Ok(ItemPicked::HPTank as i8);
            }

            if exp_tank_query.contains(target_e) {
                return Ok(ItemPicked::EXPTank as i8);
            }

            if item_query.contains(target_e) {
                return Ok(ItemPicked::Item as i8);
            }

            if obstacle_query.contains(target_e) {
                return Ok(ItemPicked::Obstacle as i8);
            }

            return Err(ItemPicked::None as i8);
        } // Check whether the target is within range. If it is, return `Ok` to trigger!
    };
}

pub fn set_state_thief(
    mut commands: Commands,
    mut thief_query: Query<(Entity, &ItemPickedFlag)>,
) {
    for (thief_e, item_picked) in thief_query.iter_mut() {
        match item_picked {
            ItemPickedFlag::None => commands.entity(thief_e).insert(Done::Failure),
            ItemPickedFlag::Some(target_e) => {
                match target_e {
                    ItemPicked::HPTank => {
                        commands.entity(thief_e).insert(HPTankRush);
                    }
                    ItemPicked::EXPTank => {
                        commands.entity(thief_e).insert(ExpTankRush);
                    }
                    ItemPicked::Item => {
                        commands.entity(thief_e).insert(ItemRush);
                    }
                    ItemPicked::Obstacle => {
                        commands.entity(thief_e).insert(ObstacleRush);
                    }
                    _ => {
                        commands.entity(thief_e).insert(Done::Failure);
                        continue;
                    }
                }
                commands.entity(thief_e).insert(Done::Success)
            }
        };
    }
}

#[derive(Component, Clone)]
pub struct PickTargetForSteal {
    pub target: Option<Entity>,
}

#[derive(Component, Clone)]
pub struct SearchingInteractableFlag;

#[derive(Component, Clone)]
pub struct ExpTankRush;

#[derive(Component, Clone)]
pub struct HPTankRush;