use std::f32::consts::PI;

use bevy::prelude::*;
use rand::Rng;

use crate::{elements::ElementType, gamemap::{ROOM_SIZE, TILE_SIZE}, player::Player, projectile::SpawnProjectileEvent};

pub struct BossBehavoiurPlugin;

impl Plugin for BossBehavoiurPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<BossAttackEvent>()
            .add_systems(Update, (charge_attack, perform_attack));
    }
}

#[derive(Component)]
pub struct Boss {
    pub attack_cooldown: Timer,
}

const WALL_DIRECTIONS: [Vec2; 4] = [
        Vec2 {x: 1.0, y: 0.0},
        Vec2 {x: -1.0, y: 0.0},
        Vec2 {x: 0.0, y: 1.0},
        Vec2 {x: 0.0, y: -1.0},
    ];

enum BossAttackType {
    Wall(Vec2),
    Radial(usize, f32),
}

#[derive(Event)]
struct BossAttackEvent(BossAttackType);

fn charge_attack(
    mut boss_query: Query<&mut Boss>,
    time: Res<Time>,
    mut ev_boss_attack: EventWriter<BossAttackEvent>,
) {
    let Ok(mut boss) = boss_query.get_single_mut() else {
        return;
    }; 

    boss.attack_cooldown.tick(time.delta());

    let mut rng = rand::thread_rng();

    if boss.attack_cooldown.just_finished() {
        match rng.gen_range(0..2) {
            0 => {
                ev_boss_attack.send(BossAttackEvent(BossAttackType::Wall(WALL_DIRECTIONS[rng.gen_range(0..4)])));
            },
            1 => {
                ev_boss_attack.send(BossAttackEvent(BossAttackType::Radial(rng.gen_range(8..16), 96.0)));
            },
            _ => {}
        }
    }
}

fn perform_attack(
    mut ev_boss_attack: EventReader<BossAttackEvent>,
    mut ev_spawn_projectile: EventWriter<SpawnProjectileEvent>,

    player_query: Query<&Transform, With<Player>>,
) {
    for ev in ev_boss_attack.read() {
        let element: ElementType = rand::random();

        match ev.0 {
            BossAttackType::Wall(direction) => {
                let to_skip = rand::thread_rng().gen_range((ROOM_SIZE/2-7)..(ROOM_SIZE/2+8));

                for i in (ROOM_SIZE/2-7)..(ROOM_SIZE/2+8) {
                    if i == to_skip {
                        continue;
                    }

                    let position = match direction {
                        Vec2::NEG_Y => Vec3::new(i as f32 * TILE_SIZE, (ROOM_SIZE/2+7) as f32 * TILE_SIZE, 1.0),
                        Vec2::Y => Vec3::new(i as f32 * TILE_SIZE, (ROOM_SIZE/2-7) as f32 * TILE_SIZE, 1.0),
                        Vec2::NEG_X => Vec3::new((ROOM_SIZE/2+7) as f32 * TILE_SIZE, i as f32 * TILE_SIZE, 1.0),
                        Vec2::X => Vec3::new((ROOM_SIZE/2-7) as f32 * TILE_SIZE, i as f32 * TILE_SIZE, 1.0),
                        _ => Vec3::ZERO,
                    };

                    ev_spawn_projectile.send(SpawnProjectileEvent {
                        texture_path: "textures/earthquake.png".to_string(),
                        color: element.color(),
                        translation: position,
                        angle: direction.to_angle(),
                        collider_radius: 1.0,
                        speed: 75.0,
                        damage: 20,
                        element,
                        is_friendly: false,
                        trajectory: crate::projectile::Trajectory::Straight,
                    });
                }
            },
            BossAttackType::Radial(amount, radius) => {
                let Ok(player_transform) = player_query.get_single() else {
                    return;
                };

                let offset = 2.0*PI / (amount as f32);

                let to_skip = rand::thread_rng().gen_range(0..amount);

                for i in 0..amount {
                    if i == to_skip {
                        continue;
                    }

                    let direction = -Vec2::from_angle(i as f32 * offset);
                    let position = (player_transform.translation.truncate() - direction * radius).extend(1.0);

                    ev_spawn_projectile.send(SpawnProjectileEvent {
                        texture_path: "textures/fireball.png".to_string(),
                        color: element.color(),
                        translation: position,
                        angle: direction.to_angle(),
                        collider_radius: 1.0,
                        speed: 50.0,
                        damage: 20,
                        element,
                        is_friendly: false,
                        trajectory: crate::projectile::Trajectory::Straight,
                    });
                }
            }
        }
    }
}