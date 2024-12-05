use std::f32::consts::PI;

use bevy::prelude::*;

use crate::utils::get_random_index_with_weight;

pub struct ParticlesPlguin;

impl Plugin for ParticlesPlguin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnParticlesEvent>();
        app.add_event::<SpawnSingleParticleEvent>();

        app.add_systems(Update, (spawn_particles, spawn_single, move_particles));
    }
}

pub enum ParticlePattern {
    Burst {
        direction: Vec2,
        distance: f32,
        spread: f32,
    },
    Circle {
        radius: f32,
    },
}

#[derive(Event)]
pub struct SpawnParticlesEvent {
    pub pattern: ParticlePattern,
    pub position: Vec3,
    pub amount: u32,
    pub color: Color,
    pub speed: f32,
    pub rotate: bool,
}

#[derive(Event)]
struct SpawnSingleParticleEvent{
    texture_path: String,
    color: Color,
    position: Vec3,
    direction: Vec2,
    lifetime: f32,
    speed: f32,
    rotate: bool,
}

#[derive(Component)]
struct Particle {
    target_position: Vec2,
    speed: f32,
}

#[derive(Component)]
struct Rotate;

fn spawn_particles(
    mut ev_spawn_particles: EventReader<SpawnParticlesEvent>,
    mut ev_spawn_single: EventWriter<SpawnSingleParticleEvent>,
) {
    for ev in ev_spawn_particles.read() {
        match ev.pattern {
            ParticlePattern::Burst {direction, distance, spread} => {
                let offset = spread / (ev.amount as f32);
                for i in -(ev.amount as i32 / 2)..(ev.amount as i32 / 2) {
                    ev_spawn_single.send(SpawnSingleParticleEvent {
                        texture_path: "textures/dust.png".to_string(),
                        color: ev.color,
                        position: ev.position,
                        direction: Vec2::from_angle(direction.to_angle() + (i as f32 * offset)),
                        lifetime: distance,
                        speed: ev.speed,
                        rotate: ev.rotate,
                    });
                }
            },
            ParticlePattern::Circle {radius} => {
                let offset = (2.*PI)/(ev.amount as f32);
                for i in 0..ev.amount {
                    ev_spawn_single.send(SpawnSingleParticleEvent {
                        texture_path: "textures/dust.png".to_string(),
                        color: ev.color,
                        position: ev.position,
                        direction: Vec2::from_angle(i as f32 * offset),
                        lifetime: radius,
                        speed: ev.speed,
                        rotate: ev.rotate,
                    });
                }
            }
        }
    }
}

fn spawn_single(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut ev_spawn_signle: EventReader<SpawnSingleParticleEvent>,
) {
    for ev in ev_spawn_signle.read() {

        let mut particle = commands.spawn(SpriteBundle {
            texture: asset_server.load(ev.texture_path.clone()),
            sprite: Sprite {
                color: ev.color,
                ..default()
            },
            transform: Transform::from_translation(ev.position),
            ..default()
        });

        let layout = TextureAtlasLayout::from_grid(UVec2::splat(16), 3, 1, None, None);
        let texture_atlas_layout = texture_atlas_layouts.add(layout);
        particle.insert(TextureAtlas {
            layout: texture_atlas_layout,
            index: get_random_index_with_weight(vec![5, 2, 1]),
        });

        particle.insert(Particle {
            target_position: ev.position.truncate() + ev.direction * ev.lifetime,
            speed: ev.speed,
        });

        if ev.rotate {
            particle.insert(Rotate);
        }
    }
}

fn move_particles(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &Particle)>,
    time: Res<Time>,
) {
    for (entity, mut transform, particle) in query.iter_mut() {
        let z = transform.translation.z;
        transform.translation = transform.translation.lerp(particle.target_position.extend(z), particle.speed * time.delta_seconds());

        if transform.translation.truncate().distance(particle.target_position) <= 2. {
            commands.entity(entity).despawn();
        }
    }
}