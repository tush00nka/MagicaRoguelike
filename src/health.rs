use avian2d::prelude::Collision;
use bevy::prelude::*;
use crate::{player::Player, GameState};
pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<PlayerHPGained>()
            .add_event::<DeathEvent>()
            .add_systems(OnExit(GameState::MainMenu), spawn_ui)
            .add_systems(Update, (update_ui, pick_up_health, death).run_if(in_state(GameState::Hub)))
            .add_systems(Update, (update_ui, pick_up_health, death).run_if(in_state(GameState::InGame)));
    }
}

#[derive(Component)]
pub struct Health {
    pub max: i32,
    pub current: i32,
}

impl Health {
    pub fn heal(&mut self, value: i32) {
        if self.current + value >= self.max {
            self.current = self.max;
        }
        else {
            self.current += value;
        }
    }
    pub fn damage(&mut self, value: i32,) {
        self.current -= value;
    }
}

#[derive(Component)]
pub struct HPBarUI;

#[derive(Component)]
struct HPBar;

#[derive(Event)]
pub struct PlayerHPGained;

#[derive(Component)]
pub struct HealthTank{
    pub hp: i32,
}

#[derive(Event)]
pub struct DeathEvent(pub Entity);

fn death(
    mut commands: Commands,
    mut ev_death: EventReader<DeathEvent>
) {
    for ev in ev_death.read(){
        commands.entity(ev.0).despawn();
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
    })
    .insert(HPBarUI)
    .with_children(|parent| { // сама полоска ХП
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
                let percent = (health.current as f32 / health.max as f32) * 100.0; 
                style.width = Val::Percent(percent);
            }
        }
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

        let tank_e: Option<Entity>;

        if tank_query.contains(contacts.entity2) && player_hp_query.contains(contacts.entity1) {
            tank_e = Some(contacts.entity2);
        }
        else if tank_query.contains(contacts.entity1) && player_hp_query.contains(contacts.entity2) {
            tank_e = Some(contacts.entity1);
        }
        else {
            tank_e = None;
        }

        for (candiate_e, tank) in tank_query.iter() {
            if tank_e.is_some() && tank_e.unwrap() == candiate_e {
                for (_player, mut health) in player_hp_query.iter_mut() {
                    health.heal(tank.hp);
                }
                ev_hp_gained.send(PlayerHPGained);
                commands.entity(tank_e.unwrap()).despawn();
            }
        }

    }
}