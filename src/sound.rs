use crate::ron_helpers::{parse, trim_extension};
use bevy::{
    app::{App, Plugin},
    asset::AssetServer,
    audio::{AudioSource, AudioSourceBundle, PlaybackMode, PlaybackSettings},
    log::info,
    prelude::*,
};
use std::collections::HashMap;

pub struct SoundPlugin {}

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_sounds).add_systems(
            Update,
            (
                play_sfx.run_if(on_event::<PlaySFX>()),
                play_music.run_if(on_event::<PlayMusic>()),
            ),
        );
    }
}

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

#[derive(Event)]
pub struct PlaySFX {
    name: String,
}

#[derive(Event)]
pub struct PlayMusic {
    name: String,
}

#[derive(Component)]
pub struct NowPlaying {}

pub fn play_sfx(
    mut commands: Commands,
    mut events: EventReader<PlaySFX>,
    sound_resource: Res<SoundResource>,
) {
    for event in events.read() {
        if let Some(handle) = sound_resource.map.get(&event.name) {
            commands.spawn(AudioSourceBundle {
                source: handle.clone(),
                settings: PlaybackSettings {
                    mode: PlaybackMode::Despawn,
                    ..default()
                },
            });
        } else {
            info!("Sound not found: {}", event.name);
        }
    }
}

pub fn play_music(
    mut commands: Commands,
    mut events: EventReader<PlaySFX>,
    sound_resource: Res<SoundResource>,
    playing_query: Query<Entity, With<NowPlaying>>,
) {
    if !playing_query.is_empty() {
        commands.entity(playing_query.single()).despawn();
    }

    for event in events.read() {
        if let Some(handle) = sound_resource.map.get(&event.name) {
            commands
                .spawn(AudioSourceBundle {
                    source: handle.clone(),
                    settings: PlaybackSettings {
                        mode: PlaybackMode::Loop,
                        ..default()
                    },
                })
                .insert(NowPlaying {});
        }
    }
}
