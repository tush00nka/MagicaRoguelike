use bevy::prelude::*;

use crate::{
    experience::{ExpGained, PlayerExperience},
    player::Player
};

pub struct ExpOrbPlugin;

impl Plugin for ExpOrbPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<SpawnExpOrbEvent>()
            .add_systems(Update, (spawn_particles, drop_particles, move_particles));
    }
}

#[derive(Component)]
pub struct ExpOrb {
    pub exp: u32,
}  

#[derive(Component)]
pub struct ExpOrbDrop {
    pub drop_destination: Vec3,
}

#[derive(Event)]
pub struct SpawnExpOrbEvent {
    pub pos: Vec3,
    pub destination: Vec3,
}

fn spawn_particles(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut ev_spawn: EventReader<SpawnExpOrbEvent>,
) {
    for ev in ev_spawn.read() {
        commands.spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(2.0, 2.0, 2.0),
                ..default()
            },
            texture: asset_server.load("textures/exp_particle.png"),
            transform: Transform::from_translation(ev.pos),
            ..default()
        })
        .insert(ExpOrb { exp: 5 })
        .insert(ExpOrbDrop { drop_destination: ev.destination });
    }
}

fn drop_particles( // грубо говоря, анимация вылетания опыта из колбы
    mut commands: Commands,
    mut orb_query: Query<(&mut Transform, &ExpOrbDrop, Entity)>,
    time: Res<Time>,
) {
    for (mut orb_transform, orb, orb_e) in orb_query.iter_mut() {
        orb_transform.translation = orb_transform.translation.lerp(orb.drop_destination, time.delta_seconds() * 10.0);

        if orb_transform.translation.distance(orb.drop_destination) <= 0.1 { // когда санимировалось, убираем компонент, который за это отвечает
            commands.entity(orb_e).remove::<ExpOrbDrop>();
        }
    }
}

fn move_particles(
    mut commands: Commands,
    mut orb_query: Query<(&mut Transform, &ExpOrb, Entity), (Without<Player>, Without<ExpOrbDrop>)>,
    mut player_experience: ResMut<PlayerExperience>,  
    mut ev_exp_gained: EventWriter<ExpGained>,
    player_query: Query<&Transform, With<Player>>,
    time: Res<Time>,
) {
    if let Ok(player_transform) = player_query.get_single() {

        for (mut orb_transform, orb, orb_e) in orb_query.iter_mut() {

            let distance = orb_transform.translation.distance(player_transform.translation);

            if distance <= 96.0 { // радиус, с которого опыт начинает притягиваться

                let direction = Vec3::new(player_transform.translation.x, player_transform.translation.y, orb_transform.translation.z);
                //orb_transform.translation = orb_transform.translation.lerp(direction, time.delta_seconds() * (100.0 /  distance));
                orb_transform.translation = orb_transform.translation.move_towards(direction, time.delta_seconds() * (5000.0 /  distance));
                if distance <= 4.0 { // опыт считается поднятым
                    player_experience.give(orb.exp);
                    ev_exp_gained.send(ExpGained);
                    commands.entity(orb_e).despawn();
                }
            }
        }
    }
}
