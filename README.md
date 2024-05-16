# Bevy 2D Game Jam Template

This is a quick template to make loading resources and some other basic tasks easier.
Targets WASM.

To add graphics or sounds, drop the files in the correct folder and edit the `config.ron`

The handles are then available through SpriteSheetResource and SoundResource, and can be looked up by filename (without the extension).

### Sprites:
Add a `SpriteMeta` component and the components needed to display a sprite will be added automatically.

Note: the entity will also need a `GlobalTransform` which can be added with Bevy's `TransformBundle` to give  
the sprite a location on the screen.

The size of the sprite in *virtual pixels* can be set by changing the `SPRITE_SIZE` constant. Every sprite is the same size (which is adequate for a tile-based game).

### Animations:

Attach an `Animation` component to add an animation:  
`Animation::new(frames: Vec<SpriteMeta>, frame_time: f32, animation_type: AnimationType)`

### Events:
`PlaySFX` plays a sound once and then despawns  
`PlayMusic` plays a sound on loop indefinitely  
`StopMusic` stops the current song