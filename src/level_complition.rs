use avian2d::prelude::Collision;
use bevy::prelude::*;

pub struct LevelCompletionPlugin;

impl Plugin for LevelCompletionPlugin{
    fn build(&self, app: &mut App){

    }
}

#[derive(Event)]
pub struct PortalEvent{
    
}