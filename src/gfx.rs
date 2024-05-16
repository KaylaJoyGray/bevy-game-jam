use crate::ron_helpers::{parse, trim_extension};
use bevy::{prelude::*, render::camera::ScalingMode::WindowSize, window::PrimaryWindow};
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;

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
            .add_systems(
                Update,
                (
                    update_animations,
                    add_sprite_from_sprite_meta.after(update_animations),
                ),
            );

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
pub fn load_sprite_sheets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let config = parse::<
        Vec<(
            String,
            f32,
            usize,
            usize,
            Vec<(String, usize, usize, f32, AnimationType)>,
        )>,
    >("./assets/graphics/config.ron")
    .expect("Fatal: could not parse graphics/config.ron");

    let mut sprite_sheet_resource = SpriteSheetResource::new();
    let mut animation_resource = AnimationResource::new();

    config
        .iter()
        .for_each(|(sheet_name, tile_size, rows, columns, animations)| {
            // load sprite sheets
            let layout = TextureAtlasLayout::from_grid(
                Vec2::new(*tile_size, *tile_size),
                *columns,
                *rows,
                None,
                None,
            );

            let sprite_sheet_handle = SpriteSheetHandle {
                texture: asset_server.load(&format!("graphics/{}", *sheet_name)),
                layout: texture_atlas_layouts.add(layout),
            };

            sprite_sheet_resource.insert(trim_extension(sheet_name), sprite_sheet_handle);

            info!(
                "Loaded sprite sheet: {}, tile size: {}px, {} row(s), {} column(s)",
                sheet_name, tile_size, rows, columns
            );

            // load animations
            animations
                .iter()
                .for_each(|(anim_name, start, end, frame_time, animation_type)| {
                    let animation = Animation::new(
                        sheet_name.clone(),
                        (*start..=*end).collect(),
                        *frame_time,
                        animation_type.clone(),
                    );
                    animation_resource.insert(anim_name.clone(), animation);

                    info!("Loaded animation: {}", anim_name);
                });
        });

    commands.insert_resource(sprite_sheet_resource);
}

#[derive(Debug, Clone, PartialEq, Component)]
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

///
/// AnimationType
///
/// * Once: plays once and stops on the last frame
/// * Repeat: loops indefinitely
/// * Despawn: despawns the entity on completion
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub enum AnimationType {
    Once,
    Repeat,
    Despawn,
}

#[derive(Debug, Clone, Component)]
pub struct Animation {
    index: usize,
    sheet_name: String,
    frames: Vec<usize>,
    timer: Timer,
    animation_type: AnimationType,
    finished: bool,
}

impl Animation {
    pub fn new(
        sheet_name: String,
        frames: Vec<usize>,
        frame_time: f32,
        animation_type: AnimationType,
    ) -> Self {
        Animation {
            index: 0,
            sheet_name,
            frames,
            timer: Timer::from_seconds(frame_time, TimerMode::Once),
            animation_type,
            finished: false,
        }
    }

    fn advance_frame(&mut self) {
        if self.animation_type.eq(&AnimationType::Repeat) {
            self.index = (self.index + 1) % self.frames.len();
            self.timer.reset();
            return;
        }

        // non-repeating animation
        if self.index < self.frames.len() - 1 {
            self.index += 1;
            self.timer.reset();
        } else {
            self.finished = true;
        }
    }

    /// Advances the timer and returns the index of the current frame
    pub fn tick(&mut self, delta: f32) -> usize {
        self.timer.tick(Duration::from_secs_f32(delta));
        if self.timer.finished() {
            self.advance_frame();
        }
        self.frames[self.index].clone()
    }

    pub fn get_type(&self) -> AnimationType {
        self.animation_type.clone()
    }

    pub fn finished(&self) -> bool {
        self.finished
    }
}

#[derive(Debug, Resource)]
pub struct AnimationResource {
    map: HashMap<String, Animation>,
}

impl AnimationResource {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Insert a new Animation
    pub fn insert(&mut self, name: String, animation: Animation) {
        self.map.insert(name, animation.clone());
    }

    /// Get an Animation
    pub fn get(&self, name: &str) -> Option<Animation> {
        self.map.get(name).cloned()
    }
}

pub fn update_animations(
    mut commands: Commands,
    time: Res<Time<Virtual>>,
    mut query: Query<(Entity, &mut SpriteMeta, &mut Animation), With<SpriteAdded>>,
) {
    for (entity, mut sprite_meta, mut animation) in query.iter_mut() {
        let next_index = animation.tick(time.delta_seconds());
        if next_index.ne(&sprite_meta.index) {
            commands.entity(entity).remove::<SpriteAdded>();
            sprite_meta.index = next_index;
        }

        if animation.finished() {
            match animation.get_type() {
                AnimationType::Once => {
                    commands.entity(entity).remove::<Animation>();
                }
                AnimationType::Despawn => {
                    commands.entity(entity).despawn();
                }
                _ => {}
            }
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
