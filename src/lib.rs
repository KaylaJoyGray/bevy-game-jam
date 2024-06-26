use bevy::prelude::*;
use wasm_bindgen::prelude::*;
mod gfx;
mod ron_helpers;
mod sound;

// TODO OTD: Start building example game

#[wasm_bindgen(start)]
pub fn start() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            gfx::GFXPlugin { snap_camera: false },
            sound::SoundPlugin {},
        ))
        .run()
}
