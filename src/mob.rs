use bevy::prelude::*;
use avian2d::prelude::*;
use rand::Rng;

use crate::{gamemap::{LevelGenerator, TileType, ROOM_SIZE}, GameState};

pub struct MobPlugin;

impl Plugin for MobPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::InGame), debug_spawn_mobs)
            .add_systems(FixedUpdate, move_mobs.run_if(in_state(GameState::InGame)));
    }
}

#[derive(Component)]
pub struct Mob {
    pub path: Vec<(u16, u16)>, 
    pub needs_path: bool,
    speed: f32,
}

fn debug_spawn_mobs(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    room: Res<LevelGenerator>,
) {
    let grid = room.grid.clone();
    for i in 1..grid.len() - 1 {
        for j in 1..grid[i].len() - 1 {
            if grid[i][j] == TileType::Floor {
                let mut rng = rand::thread_rng();
                if rng.gen::<f32>() > 0.98 {
                    let mob = commands.spawn(SpriteBundle {
                        texture: asset_server.load("textures/player_placeholder.png"),
                        transform: Transform::from_xyz( (i as i32 * ROOM_SIZE) as f32, (j as i32 * ROOM_SIZE) as f32, 1.0),
                        ..default()
                    }).id();
                
                    commands.entity(mob)
                        .insert(RigidBody::Dynamic)
                        .insert(GravityScale(0.0))
                        .insert(LockedAxes::ROTATION_LOCKED)
                        .insert(Collider::circle(6.0))
                        .insert(LinearVelocity::ZERO)
                        .insert(Mob { path: vec![], needs_path: true, speed: 5000. });
                }
            }
        }
    }
}

fn move_mobs(
    mut mob_query: Query<(&mut LinearVelocity, &Transform, &mut Mob)>,
    time: Res<Time>,
) {
    for (mut linvel, transform, mut mob) in mob_query.iter_mut() {
        if mob.path.len() > 0 {
            mob.needs_path = false;
            //let mob_tile_pos = Vec2::new(((transform.translation.x - (ROOM_SIZE / 2) as f32) / ROOM_SIZE as f32).floor(), (transform.translation.y - (ROOM_SIZE / 2) as f32) / ROOM_SIZE as f32).floor();
            let direction = Vec2::new(mob.path[0].0 as f32 * 32. - transform.translation.x, mob.path[0].1 as f32 * 32. - transform.translation.y).normalize();

            linvel.0 = direction * mob.speed * time.delta_seconds();

            if transform.translation.truncate().distance(Vec2::new(mob.path[0].0 as f32 * 32., mob.path[0].1 as f32 * 32.)) <= 4. {
                mob.needs_path = true;
                mob.path.remove(0);
            }

        }
    }
}