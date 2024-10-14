use bevy::prelude::*;

pub struct LoadingScreenUIPlugin;
use crate::{
    GameState,
};
impl Plugin for LoadingScreenUIPlugin{
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Loading),load_loading_screen)
        .add_systems(Update, display_loading_screen);
    }
}

#[derive(Component)]
struct LoadingScreen;

// Spawns the necessary components for the loading screen.
fn load_loading_screen(mut commands: Commands) {
    let text_style = TextStyle {
        font_size: 80.0,
        ..default()
    };
    // Spawn the UI that will make up the loading screen.
    commands
        .spawn((
            NodeBundle {
                background_color: BackgroundColor(Color::BLACK),
                style: Style {
                    height: Val::Percent(100.0),
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ..default()
            },
            LoadingScreen,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_sections([TextSection::new(
                "Loading...",
                text_style.clone(),
            )]));
        });
}

fn display_loading_screen(
    mut loading_screen: Query<&mut Visibility, With<LoadingScreen>>,
    current_state: Res<State<GameState>>,
) {
    match current_state.get() {
        GameState::Loading => {
            *loading_screen.get_single_mut().unwrap() = Visibility::Visible;
        }
        _ => *loading_screen.get_single_mut().unwrap() = Visibility::Hidden,
    };
}