use bevy::prelude::*;
use rand::Rng;

use crate::{elements::ElementType, gamemap::{ROOM_SIZE, TILE_SIZE}, projectile::SpawnProjectileEvent};

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

    if boss.attack_cooldown.just_finished() {
        ev_boss_attack.send(BossAttackEvent(BossAttackType::Wall(WALL_DIRECTIONS[rand::thread_rng().gen_range(0..4)])));
    }
}

fn perform_attack(
    mut ev_boss_attack: EventReader<BossAttackEvent>,
    mut ev_spawn_projectile: EventWriter<SpawnProjectileEvent>,
) {
    for ev in ev_boss_attack.read() {
        match ev.0 {
            BossAttackType::Wall(direction)=> {
                for i in (ROOM_SIZE/2-7)..(ROOM_SIZE/2+8) {
                    let pos = match direction {
                        Vec2::NEG_Y => Vec3::new(i as f32 * TILE_SIZE, (ROOM_SIZE/2+7) as f32 * TILE_SIZE, 1.0),
                        Vec2::Y => Vec3::new(i as f32 * TILE_SIZE, (ROOM_SIZE/2-7) as f32 * TILE_SIZE, 1.0),
                        Vec2::NEG_X => Vec3::new((ROOM_SIZE/2+7) as f32 * TILE_SIZE, i as f32 * TILE_SIZE, 1.0),
                        Vec2::X => Vec3::new((ROOM_SIZE/2-7) as f32 * TILE_SIZE, i as f32 * TILE_SIZE, 1.0),
                        _ => Vec3::ZERO,
                    };

                    ev_spawn_projectile.send(SpawnProjectileEvent {
                        texture_path: "textures/earthquake.png".to_string(),
                        color: ElementType::Fire.color(),
                        translation: pos,
                        angle: direction.to_angle(),
                        radius: 1.0,
                        speed: 50.0,
                        damage: 20,
                        element: ElementType::Fire,
                        is_friendly: false,
                    });
                }
            }
        }
    }
}