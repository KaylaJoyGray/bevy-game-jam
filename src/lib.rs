use bevy::prelude::*;
use wasm_bindgen::prelude::*;
mod gfx;
mod ron_helpers;

#[wasm_bindgen(start)]
pub fn start() {
    App::new().run()
}