//Бандл для особых мобов
//пока что только вор - ворует предметы и убегает с карты, если ударить выкидывает все что поднял
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
use avian2d::prelude::*;
use bevy::prelude::*;
use seldom_state::prelude::*;
use std::time::Duration;

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
}

#[derive(PartialEq, Clone, Component)]
pub enum ItemPickedFlag {
    Some(ItemPicked),
    None,
}

#[derive(PartialEq, Clone, Component)]
pub struct PickupItem {
    pub item_type: ItemPicked,
    pub item_name: Option<ItemType>,
}

#[derive(Component)]
pub enum OnDeathEffect {
    CircleAttack,
}

#[derive(Component)]
pub enum OnHitEffect {
    DropItemFromBag,
}
#[derive(Event)]
pub struct PushItemQueryEvent {
    pub thief_entity: Entity,
    pub item: PickupItem,
}

#[derive(Component)]
pub struct PickupItemQueue {
    pub item_queue: Vec<Option<PickupItem>>,
    pub amount_of_obstacles: u8,
}

impl PickupItemQueue {
    pub fn empty_queue(&mut self) {
        for i in 0..self.item_queue.len() {
            self.item_queue[i] = None;
        }
    }

    fn push_obstacle(&mut self) -> bool {
        self.amount_of_obstacles += 1;
        if self.amount_of_obstacles >= 8 {
            return true;
        }
        return false;
    }

    fn get_len(&mut self) -> i32 {
        let mut len = 0;
        for i in self.item_queue.iter() {
            match i {
                None => break,
                Some(_) => len += 1,
            };
        }
        return len;
    }

    fn push_item(&mut self, item: PickupItem) {
        let len = self.item_queue.len();

        self.item_queue[len - 1] = None;
        for i in (1..len).rev() { //?????
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
    on_hit_effect: OnHitEffect,
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
            exp_loot: MobLoot { orbs: 1 },
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
                update_path_timer: Timer::new(Duration::from_millis(200), TimerMode::Repeating),
                ..default()
            },
            items: PickupItemQueue::default(),
            on_death_attack: OnDeathEffect::CircleAttack,
            on_hit_effect: OnHitEffect::DropItemFromBag,
        }
    }
}

pub fn thief_collide(
    mut commands: Commands,
    mut thief_query: Query<(Entity, &mut PickupItemQueue, &CollidingEntities, &Transform)>,
    interactable_query: Query<
        Entity,
        (
            Or<(With<Item>, With<Obstacle>, With<HealthTank>, With<ExpTank>)>,
            Without<Corpse>,
        ),
    >,
    exp_tank_rush_query: Query<&ExpTankRush, With<PickupItemQueue>>,
    obstacle_rush_query: Query<&ObstacleRush, With<PickupItemQueue>>,
    item_rush_query: Query<&ItemRush, With<PickupItemQueue>>,
    health_tank_rush_query: Query<&HPTankRush, With<PickupItemQueue>>,

    mut item_push_ev: EventWriter<PushItemQueryEvent>,
    hp_tank_query: Query<&HealthTank>,
    exp_tank_query: Query<&ExpTank>,
    item_query: Query<&Item>,

    mut ev_mob_death: EventWriter<MobDeathEvent>,
) {
    for (mob_e, mut obstacle_count, colliding_e, transform) in thief_query.iter_mut() {
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

                if item_query.contains(interactable_e) {
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

                commands.entity(interactable_e).despawn();
                
                let mut check = false;

                if is_obstacle {
                    check = obstacle_count.push_obstacle();
                }

                if health_tank_rush_query.contains(mob_e) {
                    commands.entity(mob_e).remove::<HPTankRush>();
                    check = obstacle_count.push_obstacle();
                }

                if exp_tank_rush_query.contains(mob_e) {
                    commands.entity(mob_e).remove::<ExpTankRush>();
                    check = obstacle_count.push_obstacle();
                }

                if item_rush_query.contains(mob_e) {
                    commands.entity(mob_e).remove::<ItemRush>();
                    check = obstacle_count.push_obstacle();
                }

                if obstacle_rush_query.contains(mob_e) {
                    commands.entity(mob_e).remove::<ObstacleRush>();
                    check = obstacle_count.push_obstacle();
                }

                if check {
                    commands.entity(mob_e).despawn();
                    
                    ev_mob_death.send(MobDeathEvent {
                        mob_unlock_tag: "lurker.png".to_string(),
                        orbs: 0,
                        pos: transform.translation,
                        dir: Vec3::ZERO,
                        is_spawned: false,
                    });
                    
                    break;
                }
                commands.entity(mob_e).insert(Done::Success);

                break;
            }
        }
    }
}

pub fn item_queue_update(
    mut item_push_ev: EventReader<PushItemQueryEvent>,
    mut thief_query: Query<(Entity, &mut PickupItemQueue)>,
    mut commands: Commands,
) {
    for ev in item_push_ev.read() {
        match thief_query.get_mut(ev.thief_entity) {
            Ok((entity, mut val)) => {
                val.push_item(ev.item.clone());
                if val.get_len() == 5 {
                    commands.entity(entity).despawn();
                }
            }
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
            Or<(With<HealthTank>, With<ExpTank>, With<Item>, With<Obstacle>)>,
        ),
    >,
    transforms_thief: Query<&Transform, With<PickupItemQueue>>,
) -> Option<Option<Entity>> {
    if transforms.iter().len() == 0 {
        return None;
    }

    let sorted_targets: Vec<(Entity, &Transform)> = transforms
        .iter()
        .sort_by::<&Transform>(|item1, item2| {
            item1
                .translation
                .distance(transforms_thief.get(entity).unwrap().translation)
                .total_cmp(
                    &item2
                        .translation
                        .distance(transforms_thief.get(entity).unwrap().translation),
                )
        })
        .collect();

    if sorted_targets.len() == 0 {
        return None;
    }

    let (nearest_target, _) = sorted_targets[0];

    return Some(Some(nearest_target));
}

pub fn pick_item_to_steal(
    In(entity): In<Entity>,
    steal_query: Query<&PickTargetForSteal>,
    hp_tank_query: Query<&HealthTank>,
    exp_tank_query: Query<&ExpTank>,
    item_query: Query<&Item>,
) -> Option<Option<ItemPicked>> {
    let Ok(steal_target) = steal_query.get(entity) else {
        return None;
    };
    match steal_target.target {
        None => {
            return None;
        }
        Some(target_e) => {
            if hp_tank_query.contains(target_e) {
                return Some(Some(ItemPicked::HPTank));
            }

            if exp_tank_query.contains(target_e) {
                return Some(Some(ItemPicked::EXPTank));
            }

            if item_query.contains(target_e) {
                return Some(Some(ItemPicked::Item));
            }

            return Some(Some(ItemPicked::Obstacle));

            //    return None;
        } // Check whether the target is within range. If it is, return `Ok` to trigger!
    };
}

pub fn set_state_thief(mut commands: Commands, mut thief_query: Query<(Entity, &ItemPickedFlag)>) {
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
