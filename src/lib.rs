use bevy::prelude::*;
use wasm_bindgen::prelude::*;
mod gfx;
mod ron_helpers;
mod sound;

// TODO: OTD: Organize SFX and sound into plugins. Add Player entity/start controls.

#[wasm_bindgen(start)]
pub fn start() {
    App::new().add_plugins(DefaultPlugins).run()
}
