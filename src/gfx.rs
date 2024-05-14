use crate::ron_config::{parse, trim_extension};
use bevy::{prelude::*, render::camera::ScalingMode::WindowSize, window::PrimaryWindow};
use regex::Regex;
use ron::de::from_reader;
use std::collections::HashMap;

/// Important: this is the sprite size before window scaling is applied
pub const SPRITE_SIZE: f32 = 2.0;
const LAYER_HEIGHT: f32 = SPRITE_SIZE;

#[derive(Debug, Clone)]
pub struct SpriteSheetHandle {
    pub texture: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
}

#[derive(Debug, Resource)]
pub struct SpriteSheetResource {
    sheets: HashMap<String, SpriteSheetHandle>,
}

impl SpriteSheetResource {
    pub fn new() -> Self {
        Self {
            sheets: HashMap::new(),
        }
    }

    pub fn insert(&mut self, name: String, sprite: SpriteSheetHandle) {
        self.sheets.insert(name, sprite);
    }

    pub fn get(&self, name: &str) -> Option<&SpriteSheetHandle> {
        self.sheets.get(name)
    }
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

        sprite_sheet_resource.insert(trim_extension(&data.0), sprite_sheet_handle);

        info!(
            "Loaded sprite sheet: {}, size: {}px, {} row(s), {} column(s)",
            data.0, data.1, data.3, data.2
        );
    });

    commands.insert_resource(sprite_sheet_resource);
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

            //commands.entity(entity).push_children(&[sprite]);
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
                scaling_mode: WindowSize(8.0), // 8 pixels per game unit
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
    mut query_camera: Query<(&mut OrthographicProjection, &mut Transform), With<MainCamera>>,
    query_focus: Query<&GlobalTransform, With<HasCameraFocus>>,
) {
    let focus_z = query_focus.single().translation().z;

    let min = (focus_z - 20.).floor(); // view is 10 blocks, 2 units each
    let max = focus_z.ceil();

    for (mut projection, mut transform) in query_camera.iter_mut() {
        projection.near = min;
        projection.far = max;
        transform.translation = query_focus.single().translation();
    }
}
