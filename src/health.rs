use avian2d::prelude::Collision;
use bevy::{prelude::*, transform::commands};
use crate::{player::{self, Player}, GameState};
pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<PlayerHPGained>()
            .add_systems(OnEnter(GameState::InGame), spawn_ui)
            .add_systems(Update, (update_ui, pick_up_health).run_if(in_state(GameState::InGame)));
    }
}

#[derive(Component)]
pub struct Health {
    pub max: u32,
    pub current: u32,
}

impl Health {
    pub fn give(&mut self, value: u32) {
        if self.current + value >= self.max {
            self.current = self.max;
        }
        else {
            self.current += value;
        }
    }
    pub fn take(&mut self, value: u32,) {
        if self.current - value > 0 {
            self.current -= value;
        }
        else {
        }
    }
}

#[derive(Component)]
struct HPBar;

#[derive(Event)]
pub struct PlayerHPGained;

#[derive(Component)]
pub struct HealthTank{
    pub hp: u32,
}

#[derive(Event)]
struct DeathEvent(Entity);
fn player_death(
    mut commands: Commands,
    health_query: Query<(Entity, &Health)>,
) {
    for (player, health) in health_query.iter() {
        if health.current <= 0 {
            commands.entity(player).despawn();
        }
    }
}

fn spawn_ui(
    mut commands: Commands,
) {

    commands.spawn(ImageBundle { // фон полоски ХП
        image: UiImage::solid_color(Color::hsl(0.0, 1.0, 0.1)),
        style: Style {
            width: Val::Px(96.0*2.0),
            height: Val::Px(24.0),
            left: Val::Px(0.0),
            top: Val::Px(20.0),
            ..default()
        },
        ..default()
    }).with_children(|parent| { // сама полоска ХП
        parent.spawn(ImageBundle {
            image: UiImage::solid_color(Color::hsl(0.0, 1.0, 0.4)),
            style: Style {
                width: Val::Percent(50.0),
                height: Val::Px(24.0),
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                ..default()
            },
            ..default()
            }).insert(HPBar); 
        }
    );

}

fn update_ui(
    mut bar_query: Query<&mut Style, With<HPBar>>, 
    player_hp_query: Query<&Health, With <Player>>,
    mut ev_hp_gained: EventReader<PlayerHPGained>,
) {

    for _ev in ev_hp_gained.read() {
        if let Ok(mut style) = bar_query.get_single_mut() {
            for health in player_hp_query.iter() {
                let percent = (health.current as f32 /health.max as f32) * 100.0; 
                style.width = Val::Percent(percent);
            }
        }
    }
}

fn init_player_health(
    mut commands: Commands,
    player_query: Query<Entity, With<Player>>,
) {
    if let Ok(player_e) = player_query.get_single(){
        commands.entity(player_e).insert(Health{max: 100, current: 50});
    }
}

fn pick_up_health(
    mut commands: Commands,
    tank_query: Query<(Entity, &HealthTank)>,
    mut player_hp_query: Query<(&Player, &mut Health)>,
    mut ev_hp_gained: EventWriter<PlayerHPGained>,
    mut ev_collision: EventReader<Collision>,
) {
    for Collision(contacts) in ev_collision.read() {
        if tank_query.contains(contacts.entity2) && player_hp_query.contains(contacts.entity1) {
            for (tank_e, tank) in tank_query.iter() {
                if contacts.entity2 == tank_e {
                    for (player, mut health) in player_hp_query.iter_mut() {
                        health.give(tank.hp);
                        health.take(2 * tank.hp);
                    }
                    ev_hp_gained.send(PlayerHPGained);
                    commands.entity(tank_e).despawn();
                }
            }
        }
    }
}