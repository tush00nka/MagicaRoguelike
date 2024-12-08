use avian2d::math::PI;

//bundle for melee only mobs
use crate::{
    blank_spell::SpawnBlankEvent,
    elements::{ElementResistance, ElementType},
    health::Health,
    level_completion::PortalManager,
    mobs::mob::*,
    stun::Stun,
    GameLayer, Timer,
};
use {
    avian2d::prelude::*, bevy::prelude::*, seldom_state::trigger::Done, std::cmp::Ordering,
    std::time::Duration,
};
#[derive(Bundle)]
pub struct OrbitalBundle {
    mob_bundle: MobBundle,
    orbital: Orbital,
}

#[derive(Component, Clone)]
pub struct BusyOrbital;

#[derive(Component, Clone)]
pub struct FreeOrbital;

impl MobBundle {
    pub fn air_elemental() -> Self {
        Self {
            phys_bundle: PhysicalBundle {
                collision_layers: CollisionLayers::new(
                    GameLayer::Enemy,
                    [
                        GameLayer::Projectile,
                        GameLayer::Player,
                    ],
                ),
                ..default()
            },
            resistance: ElementResistance {
                elements: vec![ElementType::Air],
                resistance_percent: vec![0, 0, 0, 80, 0],
            },
            mob_type: MobType::AirElemental,
            mob: Mob::new(1),
            health: Health::new(30),
            ..default()
        }
    }
}

impl OrbitalBundle {
    pub fn air_elemental() -> Self {
        Self {
            mob_bundle: MobBundle::air_elemental(),
            orbital: Orbital {
                time_to_live: Timer::new(Duration::from_millis(10000), TimerMode::Once),
                is_eternal: false,
                speed: 2000.,
                parent: None,
            },
        }
    }
}
#[derive(Component)]
pub struct OrbitalCount{
    pub orbital_array: Vec<Entity>,
}
pub fn air_elemental_movement<Side: Component>(
    mut commands: Commands,
    mut airel_query: Query<
        (Entity, &mut LinearVelocity, &mut Transform, &mut Orbital),
        (
            Without<Stun>,
            Without<Teleport>,
            Without<RaisingFlag>,
            With<Side>,
            With<FreeOrbital>,
        ),
    >,
    target_query: Query<(Entity, &Transform), (With<Side>, Without<Orbital>)>,
    time: Res<Time>,
    mut orbital_count_query: Query<&mut OrbitalCount>,
) {
    for (air_e, mut lin_vel, mut air_transform, mut orbital) in airel_query.iter_mut() {
        if target_query.iter().len() <= 0 {
            return;
        }
        let sorted_targets: Vec<(Entity, &Transform)> = target_query
            .iter()
            .sort_by::<&Transform>(|item1, item2| {
                if item1.translation.distance(air_transform.translation)
                    < item2.translation.distance(air_transform.translation)
                {
                    return Ordering::Less;
                } else if item1.translation.distance(air_transform.translation)
                    > item2.translation.distance(air_transform.translation)
                {
                    return Ordering::Greater;
                }

                return Ordering::Equal;
            })
            .collect();

        let (target_e, target_transform) = sorted_targets[0];

        let direction = (target_transform.translation - air_transform.translation)
            .truncate()
            .normalize();

        lin_vel.0 = direction * orbital.speed * time.delta_seconds();

        if air_transform
            .translation
            .distance(target_transform.translation)
            < 28.
        {
            air_transform.translation = target_transform.translation;
            commands.entity(target_e).push_children(&[air_e]);
            
            orbital.parent = Some(Box::new(target_e));
//            if orbital_count_query.contains(target_e){
//                orbital_count_query.get_mut(target_e).unwrap().orbital_array.push(air_e);
//            }else{
//                commands.entity(target_e).insert(OrbitalCount{orbital_array: vec![air_e],});
//            }
            commands.entity(air_e).insert(Done::Success);
        }
    }
}

pub fn clear_orbitals(mob_query: Query<&Orbital>, mut commands: Commands, mut parent_query: Query<(Entity, &mut OrbitalCount)>){
    for (parent_e, mut orbital_count) in parent_query.iter_mut(){
        if orbital_count.orbital_array.len() == 0{
            commands.entity(parent_e).remove::<OrbitalCount>();
            continue;
        }
        for i in 0..orbital_count.orbital_array.len(){
            if !mob_query.contains(orbital_count.orbital_array[i]){
                orbital_count.orbital_array[i] = orbital_count.orbital_array[orbital_count.orbital_array.len() - 1];
                orbital_count.orbital_array.pop();
                println!("cleared");
                break;
            }
        }
    }
} 

pub fn rotate_orbital<Side: Component>(
    mut orbital_query: Query<(Entity,&mut Orbital, &mut Transform), (With<Side>, With<BusyOrbital>)>,
    parent_query: Query<&Children, (With<Side>, Without<Orbital>)>,
    time: Res<Time>,
    mut commands: Commands, 
) {
    for (orbital_e,mut orbital, mut transform_orb) in orbital_query.iter_mut() {
        if !parent_query.contains(*(orbital.parent.clone().unwrap())){
            orbital.parent = None;
        }
        match &orbital.parent {
            Some(parent) => {
                let orbitals = parent_query
                    .get(*parent.clone())
                    .unwrap();
                
                let count = orbitals.iter().len();
                let mut multiplier = 0;
                for orbitals_e in orbitals.iter(){
                    multiplier += 1;
                    if *orbitals_e == orbital_e{
                        break;
                    }
                }
                let pos_new = //transform_orb.translation
                    //.truncate()
                    Vec2::from_angle(PI * multiplier as f32 * time.elapsed_seconds() / count as f32  ) * 32.;

                transform_orb.translation = Vec3::new(pos_new.x, pos_new.y,0.);
            } // radius
            None => {commands.entity(orbital_e).insert(Done::Success);}
        };
    }
}

pub fn timer_tick_orbital<Side: Component>(
    mut airel_query: Query<
        (Entity, &GlobalTransform, &mut Orbital),
        (
            Without<Stun>,
            Without<Teleport>,
            Without<RaisingFlag>,
            With<Side>,
            With<BusyOrbital>,
        ),
    >,
    time: Res<Time>,
    mut commands: Commands,
    mut spawn_blank_ev: EventWriter<SpawnBlankEvent>,
    mut portal_manager: ResMut<PortalManager>,
) {
    for (elemental_e, pos, mut orbital) in airel_query.iter_mut() {
        if orbital.is_eternal {
            continue;
        }
        orbital.time_to_live.tick(time.delta());
        if orbital.time_to_live.just_finished() {
            
            let is_friendly: bool;
            if std::any::type_name::<Side>() == std::any::type_name::<Enemy>() {
                is_friendly = false;
                portal_manager.pop_mob();
            }
            else { is_friendly = true; }

            spawn_blank_ev.send(SpawnBlankEvent {
                range: 18.,
                position: pos.translation(),
                speed: 4.5,
                is_friendly,
            });
            commands.entity(elemental_e).insert(Done::Success);
            commands.entity(elemental_e).despawn();
        }
    }
}
