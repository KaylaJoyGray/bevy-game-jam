use bevy::prelude::{Asset, AudioSource, Decodable, Handle, Resource};
use bevy::utils::HashMap;

#[derive(Debug, Resource)]
pub struct SoundResource<Source = AudioSource>
where
    Source: Asset + Decodable,
{
    pub map: HashMap<String, Handle<Source>>,
}

impl SoundResource {
    pub fn new() -> Self {
        SoundResource {
            map: HashMap::new(),
        }
    }
}
