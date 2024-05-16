use crate::ron_helpers::{parse, trim_extension};
use bevy::{prelude::*, render::camera::ScalingMode::WindowSize, window::PrimaryWindow};
use std::collections::HashMap;

pub struct GFXPlugin {
    pub snap_camera: bool, // snaps camera to the entity with HasCameraFocus (must be a single entity)
}

impl Default for GFXPlugin {
    fn default() -> Self {
        GFXPlugin { snap_camera: false }
    }
}

impl Plugin for GFXPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (load_sprite_sheets, spawn_camera))
            .add_systems(Update, add_sprite_from_sprite_meta);

        if self.snap_camera {
            app.add_systems(Update, snap_camera_to_focus);
        }
    }
}

/// Important: this is the sprite size before window scaling is applied
pub const SPRITE_SIZE: f32 = 1.0;

#[derive(Debug, Clone)]
pub struct SpriteSheetHandle {
    pub texture: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
}

#[derive(Debug, Resource)]
pub struct SpriteSheetResource {
    map: HashMap<String, SpriteSheetHandle>,
}

impl SpriteSheetResource {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Insert a new SpriteSheetHandle
    pub fn insert(&mut self, name: String, handle: SpriteSheetHandle) {
        self.map.insert(name, handle.clone());
    }

    /// Get a SpriteSheetHandle
    pub fn get(&self, name: &str) -> Option<SpriteSheetHandle> {
        self.map.get(name).cloned()
    }
}

///
/// load_sprite_sheets: Bevy system
///
/// This system scans the graphics folder for sprite sheets and loads the resources
/// to the asset server
///
pub fn load_sprite_sheets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let config = parse::<Vec<(String, f32, i32, i32)>>("./assets/graphics/config.ron")
        .expect("Fatal: could not parse graphics/config.ron");

    let mut sprite_sheet_resource = SpriteSheetResource::new();

    config.iter().for_each(|data| {
        let layout = TextureAtlasLayout::from_grid(
            Vec2::new(data.1, data.1),
            data.2 as usize,
            data.3 as usize,
            None,
            None,
        );

        let sprite_sheet_handle = SpriteSheetHandle {
            texture: asset_server.load(&format!("graphics/{}", data.0)),
            layout: texture_atlas_layouts.add(layout),
        };

        sprite_sheet_resource
            .insert(trim_extension(&data.0), sprite_sheet_handle);

        info!(
            "Loaded sprite sheet: {}, size: {}px, {} row(s), {} column(s)",
            data.0, data.1, data.3, data.2
        );
    });

    commands.insert_resource(sprite_sheet_resource);
}

#[derive(Debug, Clone, Component)]
pub struct SpriteMeta {
    pub index: usize,
    pub sheet_name: String,
}

impl Default for SpriteMeta {
    fn default() -> Self {
        SpriteMeta {
            index: 0,
            sheet_name: "default".to_string(),
        }
    }
}

#[derive(Debug, Component)]
pub struct SpriteAdded {}

///
/// add_sprite_from_sprite_meta: Bevy system
///
/// This system finds SpriteMeta components that do not have a sprite sheet bundle added yet,
/// and adds the bundle. The SpriteMeta component contains only an identifier and an index, so
/// this system needs to run for anything to be displayed on the screen
///
pub fn add_sprite_from_sprite_meta(
    mut commands: Commands,
    query: Query<
        (Entity, &SpriteMeta),
        (
            With<SpriteMeta>,
            With<GlobalTransform>,
            Without<SpriteAdded>,
        ),
    >,
    sprite_sheet_resource: Res<SpriteSheetResource>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    for (entity, sprite) in query.iter() {
        let handle = sprite_sheet_resource.get(sprite.sheet_name.as_str());

        if let Some(handle) = handle {
            commands
                .entity(entity)
                .insert(SpriteSheetBundle {
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(
                            SPRITE_SIZE * window.single().scale_factor(),
                            SPRITE_SIZE * window.single().scale_factor(),
                        )),
                        color: Color::rgb(1.0, 1.0, 1.0), // needed for shading to work properly
                        ..default()
                    },
                    texture: handle.texture.clone(),
                    atlas: TextureAtlas {
                        layout: handle.layout.clone(),
                        index: sprite.index,
                    },
                    ..default()
                })
                .insert(SpriteAdded {});
        } else {
            warn!("Warning: no sprite sheet named {} found", sprite.sheet_name);
        }
    }
}

#[derive(Debug, Component)]
pub struct MainCamera {}

#[derive(Debug, Component)]
pub struct HasCameraFocus {}

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        MainCamera {},
        Camera2dBundle {
            projection: OrthographicProjection {
                near: -1000.0,
                far: 1000.0,
                scaling_mode: WindowSize(16.0), // 16 pixels per game unit
                ..default()
            },
            camera: Camera {
                clear_color: ClearColorConfig::from(Color::rgb(0.0, 0.0, 0.0)),
                ..default()
            },
            ..default()
        },
    ));
}

pub fn snap_camera_to_focus(
    mut query_camera: Query<&mut Transform, With<MainCamera>>,
    query_focus: Query<&GlobalTransform, With<HasCameraFocus>>,
) {
    for mut transform in query_camera.iter_mut() {
        transform.translation = query_focus.single().translation();
    }
}
