# Bevy 2D Game Jam Template

This is a quick template to make loading resources and some other basic tasks easier.

To add graphics or sounds, drop the files in the correct folder and edit the `config.ron`

The handles are then available through SpriteSheetResource and SoundResource and can be looked up by name.  
The name is just the filename with the extension stripped.

### Events:
`PlaySFX` plays a sound once and then despawns  
`PlayMusic` plays a sound on loop indefinitely  
`StopMusic` stops the current song