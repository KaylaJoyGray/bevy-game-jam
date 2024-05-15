use crate::ron_helpers::{parse, trim_extension};
use bevy::asset::AssetServer;
use bevy::log::info;
use bevy::prelude::{AudioSource, Commands, Handle, Res, Resource};
use bevy::utils::HashMap;

#[derive(Debug, Resource)]
pub struct SoundResource {
    pub map: HashMap<String, Handle<AudioSource>>,
}

impl SoundResource {
    pub fn new() -> Self {
        SoundResource {
            map: HashMap::new(),
        }
    }
}

///
/// load_sounds: Bevy system
///
/// This system scans the graphics folder for sprite sheets and loads the resources
/// to the asset server
///
pub fn load_sounds(mut commands: Commands, asset_server: Res<AssetServer>) {
    let config = parse::<Vec<String>>("./assets/sounds/config.ron")
        .expect("Fatal: could not parse sounds/config.ron");

    let mut sound_resource = SoundResource::new();

    config.iter().for_each(|data| {
        let handle = asset_server.load(data);

        sound_resource.map.insert(trim_extension(&data), handle);

        info!("Loaded sound file: {}", data);
    });

    commands.insert_resource(sound_resource);
}
