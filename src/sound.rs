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
        app.add_event::<PlaySFX>()
            .add_event::<PlayMusic>()
            .add_event::<StopMusic>()
            .add_systems(Startup, load_sounds)
            .add_systems(
                Update,
                (
                    play_sfx.run_if(on_event::<PlaySFX>()),
                    play_music.run_if(on_event::<PlayMusic>()),
                    stop_music.run_if(on_event::<StopMusic>()),
                ),
            );
    }
}

#[derive(Debug, Resource)]
pub struct SoundResource {
    map: HashMap<String, Handle<AudioSource>>,
}

impl SoundResource {
    pub fn new() -> Self {
        SoundResource {
            map: HashMap::new(),
        }
    }

    /// Insert a new Handle<AudioSource>
    pub fn insert(&mut self, name: String, handle: Handle<AudioSource>) {
        self.map.insert(name, handle.clone());
    }

    /// Get a Handle<AudioSource>
    pub fn get(&self, name: &str) -> Option<Handle<AudioSource>> {
        self.map.get(name).cloned()
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
        let handle: Handle<AudioSource> = asset_server.load(data);

        sound_resource.insert(trim_extension(&data), handle);

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

#[derive(Event)]
pub struct StopMusic {}

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
            warn!("Sound not found: {}", event.name);
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

pub fn stop_music(mut commands: Commands, playing_query: Query<Entity, With<NowPlaying>>) {
    if !playing_query.is_empty() {
        commands.entity(playing_query.single()).despawn();
    }
}
