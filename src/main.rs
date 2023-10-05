use std::{f32::consts::PI, env};

use bevy::{prelude::*, render::{render_resource::{TextureFormat, TextureDimension, Extent3d}, mesh::VertexAttributeValues}};
use gui::GuiPlugin;
use marker::{Markers, Marker, setup_markers};
use scene_hook::{SceneHook, HookPlugin};
use taquin::{Taquin, TaquinPlugin};
use tile::{EmptyTile, TileCoordinates, TileValue, TileSelected, TilePlugin};


mod scene_hook;
mod taquin;
mod tile;
mod gui;
mod marker;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(HookPlugin)
        .add_plugins(TilePlugin)
        .add_plugins(GuiPlugin)
        .add_plugins(TaquinPlugin {size: args.get(1).unwrap_or(&"3".to_string()).parse::<i8>().unwrap_or(3)})
        .add_state::<AppState>()
        .init_resource::<Markers>()
        .add_systems(OnEnter(AppState::Setup), setup_scene)
        .add_systems(Update, setup_markers.run_if(in_state(AppState::Setup)))
        .add_systems(Update, check_setup_finished.run_if(in_state(AppState::Setup)))
        .add_systems(OnEnter(AppState::SetupTiles), setup_tiles)
        .run();
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
pub enum AppState {
    #[default]
    Setup,
    SetupTiles,
    Running,
}

#[derive(Component)]
struct FrameScene;


#[derive(Resource)]
struct TaquinSprites {
    bevy: Handle<Image>,
    rust: Handle<Image>
}

struct TaquinSpritesLoaded {
    bevy: bool,
    rust: bool
}

impl TaquinSpritesLoaded {
    fn is_ready(&self) -> bool {
        self.bevy && self.rust
    }
}

impl FromWorld for TaquinSpritesLoaded {
    fn from_world(_world: &mut World) -> Self {
        TaquinSpritesLoaded { bevy: false, rust: false }
    }
}

fn setup_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    let debug_material = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(uv_debug_texture())),
        ..default()
    });

    commands.insert_resource(TaquinSprites {
        bevy: asset_server.load("textures/taquin/bevy.png"),
        rust: asset_server.load("textures/taquin/rust.png")
    });
        
    commands.spawn((SceneBundle {
        scene: asset_server.load("models/frame.glb#Scene0"),
        transform: Transform::from_rotation(Quat::from_axis_angle(Vec3::Y, PI)),
        ..default()
    }, FrameScene, SceneHook::new(|entity, commands| {
        match entity.get::<Name>().map(|t|t.as_str()) {
            Some("TL") | Some("TR") | Some("BL") | Some("BR") => commands.insert(Marker),
            _ => commands,
        };
    })));


    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 9000.0,
            range: 100.,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(8.0, 16.0, 8.0),
        ..default()
    });

    // ground plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane::from_size(50.0).into()),
        material: debug_material,
        ..default()
    });

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 30., 40.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
        ..default()
    });
}


fn check_setup_finished(
    taquin_sprite_folder: ResMut<TaquinSprites>,
    mut events: EventReader<AssetEvent<Image>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut taquin_sprites_loaded: Local<TaquinSpritesLoaded>,
    markers: Res<Markers>
) {

    for event in events.read() {
        if event.is_loaded_with_dependencies(&taquin_sprite_folder.bevy) {
            taquin_sprites_loaded.bevy = true;
        } else if event.is_loaded_with_dependencies(&taquin_sprite_folder.rust) {
            taquin_sprites_loaded.rust = true;
        }
    }

    if markers.is_ready() && taquin_sprites_loaded.is_ready() {
        next_state.set(AppState::SetupTiles);
    }
}

fn setup_tiles(
    mut commands: Commands,
    taquin_sprite_handles: Res<TaquinSprites>,
    markers: Res<Markers>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut taquin : ResMut<Taquin>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let tile_width = markers.inner_width() / taquin.size as f32;
    let tile_height = markers.inner_height() / taquin.size as f32;
    let tile_ratio = 1. / taquin.size as f32;
    let origin = markers.tl;

    taquin.tiles = (0..taquin.size).map(|j| {
        (0..taquin.size).map(|i| {
            let translation = Vec3 { 
                x: origin.x + i as f32 * tile_width + tile_width / 2., 
                y: origin.y - j as f32 * tile_height - tile_height / 2., 
                z: 0.75
            };
            let value = j * taquin.size + i + 1;
            let name = Name::new("Tile-".to_string() + value.to_string().as_str());
            if i == taquin.size - 1 && j == taquin.size - 1 {
                commands.spawn((Transform::from_translation(translation), EmptyTile, TileCoordinates::new(i, j)));
                return TileValue(taquin.size * taquin.size);
            }
            let mut block = Mesh::from(shape::Quad::new(Vec2::new(tile_width, tile_height)));
            if let Some(attr) = block.attribute_mut(Mesh::ATTRIBUTE_UV_0) {
                *attr = VertexAttributeValues::Float32x2(vec![
                    [0. + i as f32 * tile_ratio, (j + 1) as f32 * tile_ratio],
                    [0. + i as f32 * tile_ratio, j  as f32 * tile_ratio],
                    [(i + 1) as f32 * tile_ratio, j  as f32 * tile_ratio],
                    [(i + 1) as f32 * tile_ratio, (j + 1) as f32 * tile_ratio],
                ]);
            }
            let mut tile_command = commands.spawn((PbrBundle {
                    mesh: meshes.add(block),
                    material: materials.add(StandardMaterial {
                        base_color_texture: Some(taquin_sprite_handles.bevy.clone()),
                        double_sided: true,
                        cull_mode: None,
                        ..default()
                    }),
                    transform: Transform::from_translation(translation),
                    ..default()
                }, 
                TileCoordinates::new(i, j), 
                name,
                AnimationPlayer::default(), 
            ));
            if i == 0 && j == 0 {
                tile_command.insert(TileSelected);
            }
            TileValue(value)
        }).collect()
    }).collect();

    next_state.set(AppState::Running);
}

/// Creates a colorful test pattern
fn uv_debug_texture() -> Image {
    const TEXTURE_SIZE: usize = 8;

    let mut palette: [u8; 32] = [
        255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255, 102, 255,
        198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255,
    ];

    let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
        palette.rotate_right(4);
    }

    Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,
    )
}