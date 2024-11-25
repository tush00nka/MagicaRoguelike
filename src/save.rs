use bevy::prelude::*;
use bevy_common_assets::json::JsonAssetPlugin;

use std::{fs::File, io::Write};

use crate::GameState;

pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(JsonAssetPlugin::<Save>::new(&["json"]));

        app.add_systems(PreStartup, load);
        app.add_systems(OnEnter(GameState::GameOver), save);
        
    }
}

#[derive(serde::Deserialize, serde::Serialize, Asset, TypePath)]
pub struct Save {
    pub seen_items: Vec<String>,
    pub seen_mobs: Vec<String>,
    pub seen_spells: Vec<String>
}

#[derive(Resource)]
pub struct SaveHandle(pub Handle<Save>); 

fn load(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    println!("asasd");
    commands.insert_resource(SaveHandle(asset_server.load("save.json")));
}

fn save(
    saves: Res<Assets<Save>>,
    handle: Res<SaveHandle>,
) {
    let save = saves.get(handle.0.id()).unwrap();

    let json_string = serde_json::to_string(save).expect("[E] Couldn't save to JSON!!");

    let mut save_file = File::create("assets/save.json").expect("[E] Couldn't open save file!!");
    save_file.write_all(json_string.as_bytes()).expect("[E] Couldn't write to file!!");
}