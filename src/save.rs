use bevy::prelude::*;
use bevy_common_assets::json::JsonAssetPlugin;

pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(JsonAssetPlugin::<Save>::new(&["json"]));

        app.add_systems(PreStartup, load);
        
    }
}

#[derive(serde::Deserialize, Asset, TypePath)]
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