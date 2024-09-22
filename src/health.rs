use bevy::prelude::*;
use crate::player::Player;
pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(PlayerHealth {
                current: 50,
                max: 100
            })
            .add_event::<HPGained>()
            .add_systems(Startup, spawn_ui)
            .add_systems(Update, (update_ui, pick_up_health));
    }
}

#[derive(Resource)]
pub struct PlayerHealth {
    max: u32,
    current: u32,
}

impl PlayerHealth {
    pub fn give(&mut self, value: u32) {
        if self.current + value >= self.max{
            self.current = self.max;
        }
        else {
            self.current += value;
        }
    }
}

#[derive(Component)]
struct HPBar;

#[derive(Event)]
pub struct HPGained;

#[derive(Component)]
pub struct HealthTank{
    pub hp: u32,
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
    player_hp: Res<PlayerHealth>,
    mut ev_hp_gained: EventReader<HPGained>,
) {

    for _ev in ev_hp_gained.read() {
        if let Ok(mut style) = bar_query.get_single_mut() {
            let percent = (player_hp.current as f32 / player_hp.max as f32) * 100.0; 
            style.width = Val::Percent(percent);
        }
    }
}

fn pick_up_health(
    mut commands: Commands,
    mut pot_query: Query<(&mut Transform, &HealthTank, Entity), Without<Player>>,
    mut player_health: ResMut<PlayerHealth>,  
    mut ev_hp_gained: EventWriter<HPGained>,
    player_query: Query<&Transform, With<Player>>,
) {
    if let Ok(player_transform) = player_query.get_single() {

        for (tank_transform, tank, tank_e) in tank_query.iter_mut() {

            let distance = tank_transform.translation.distance(player_transform.translation);

            if distance <= 24.0 { // радиус, с которого хп подбирается
                player_health.give(tank.hp);
                ev_hp_gained.send(HPGained);
                commands.entity(tank_e).despawn();
            }
        }
    }
}