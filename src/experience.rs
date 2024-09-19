use bevy::prelude::*;

pub struct ExperiencePlugin;

impl Plugin for ExperiencePlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(PlayerExperience {
                current: 0,
                to_lv_up: 100,
                lv: 1,
            })
            .add_event::<ExpGained>()
            .add_systems(Startup, spawn_ui)
            .add_systems(Update, update_ui);
    }
}

#[derive(Resource)]
pub struct PlayerExperience {
    pub current: u32,
    to_lv_up: u32,
    lv: u32,
}

impl PlayerExperience {
    pub fn give(&mut self, value: u32) {
        if self.current + value >= self.to_lv_up {
            self.lv += 1;
            self.current = self.current + value - self.to_lv_up; 
            self.to_lv_up += 50;
        }
        else {
            self.current += value;
        }
    }
}

#[derive(Component)]
struct ExpBar;

#[derive(Event)]
pub struct ExpGained;

fn spawn_ui(
    mut commands: Commands,
) {

    commands.spawn(ImageBundle {
        image: UiImage::solid_color(Color::hsl(35.0, 0.5, 1.0)),
        style: Style {
            width: Val::Px(96.0*2.0),
            height: Val::Px(24.0),
            ..default()
        },
        ..default()
    }).with_children(|parent| { 
        parent.spawn(ImageBundle {
            image: UiImage::solid_color(Color::hsl(25.0, 1.0, 0.5)),
            style: Style {
                width: Val::Percent(0.0),
                height: Val::Px(24.0),
                ..default()
            },
            ..default()
            }).insert(ExpBar); 
        }
    );

}

fn update_ui(
    mut bar_query: Query<&mut Style, With<ExpBar>>, 
    player_exp: Res<PlayerExperience>,
    mut ev_exp_gained: EventReader<ExpGained>,
) {

    for _ev in ev_exp_gained.read() {
        if let Ok(mut style) = bar_query.get_single_mut() {
            let percent = (player_exp.current as f32 / player_exp.to_lv_up as f32) * 100.0; 
            style.width = Val::Percent(percent);
        }
    }
}