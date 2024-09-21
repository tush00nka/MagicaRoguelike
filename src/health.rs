use bevy::prelude::*;

pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(PlayerHealth {
                current: 100,
                max: 100
            })
            .add_event::<HPGained>()
            .add_systems(Startup, spawn_ui)
            .add_systems(Update, update_ui);
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

fn spawn_ui(
    mut commands: Commands,
) {

    commands.spawn(ImageBundle { // фон полоски ХП
        image: UiImage::solid_color(Color::hsl(0.0, 1.0, 0.1)),
        style: Style {
            width: Val::Px(96.0*2.0),
            height: Val::Px(24.0),
            ..default()
        },
        ..default()
    }).with_children(|parent| { // сама полоска ХП
        parent.spawn(ImageBundle {
            image: UiImage::solid_color(Color::hsl(0.0, 1.0, 0.4)),
            style: Style {
                width: Val::Percent(0.0),
                height: Val::Px(24.0),
                left: Val::Px(2.0),
                top: Val::Px(2.0),
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