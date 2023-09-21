use std::f32::consts::PI;

use bevy::{prelude::*, render::{render_resource::{TextureFormat, TextureDimension, Extent3d}, mesh::VertexAttributeValues}, asset::LoadedFolder};
use scene_hook::{SceneHook, HookPlugin};


mod scene_hook;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(HookPlugin)
        .add_state::<AppState>()
        .init_resource::<AppConfig>()
        .init_resource::<Markers>()
        .add_systems(OnEnter(AppState::Setup), setup_scene)
        .add_systems(Update, setup_markers.run_if(in_state(AppState::Setup)))
        .add_systems(Update, check_setup_finished.run_if(in_state(AppState::Setup)))
        .add_systems(OnEnter(AppState::SetupTiles), setup_tiles)
        .add_systems(Update, rotate.run_if(in_state(AppState::Running)))
        .run();
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
enum AppState {
    #[default]
    Setup,
    SetupTiles,
    Running,
}

#[derive(Resource)]
struct AppConfig {
    taquin_size: u8
}

impl FromWorld for AppConfig {
    fn from_world(_world: &mut World) -> Self {
        AppConfig { taquin_size: 4 }
    }
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

#[derive(Component)]
struct Marker;

#[derive(Resource, Default)]
pub struct Markers {
    pub tl : Option<Vec3>,
    pub tr : Option<Vec3>,
    pub bl : Option<Vec3>,
    pub br : Option<Vec3>,
}

impl Markers {
    pub fn is_ready(&self) -> bool {
        self.tl.is_some() && self.tr.is_some() && self.bl.is_some() && self.br.is_some()
    }
}

#[derive(Component)]
struct Tile;

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

fn setup_markers(
    mut markers: ResMut<Markers>,
    query: Query<(Entity, &Name, &GlobalTransform), With<Marker>>
) {
    for (entity, name, global_transform) in query.iter() {
        match name.as_str() {
            "TL" => markers.tl = Some(global_transform.translation()),
            "TR" => markers.tr = Some(global_transform.translation()),
            "BL" => markers.bl = Some(global_transform.translation()),
            "BR" => markers.br = Some(global_transform.translation()),
            _ => (),
        };
    }
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
    app_config : Res<AppConfig>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let taquin_inner_width = markers.tr.unwrap().x - markers.tl.unwrap().x;
    let taquin_inner_height = markers.tr.unwrap().y - markers.br.unwrap().y;
    let tile_width = taquin_inner_width / app_config.taquin_size as f32;
    let tile_height = taquin_inner_height / app_config.taquin_size as f32;
    let tile_width_ratio = tile_width / taquin_inner_width;
    let tile_height_ratio = tile_height / taquin_inner_height;
    let origin = markers.tl.unwrap();
    
    for i in 0..app_config.taquin_size {
        for j in 1..=app_config.taquin_size {
            let mut block = Mesh::from(shape::Quad::new(Vec2::new(tile_width, tile_height)));
            if let Some(attr) = block.attribute_mut(Mesh::ATTRIBUTE_UV_0) {
                *attr = VertexAttributeValues::Float32x2(vec![
                    [0. + i as f32 * tile_width_ratio, j as f32 * tile_height_ratio],
                    [0. + i as f32 * tile_width_ratio, (j - 1) as f32 * tile_height_ratio],
                    [(i + 1) as f32 * tile_width_ratio, (j - 1) as f32 * tile_height_ratio],
                    [(i + 1) as f32 * tile_width_ratio, j as f32 * tile_height_ratio],
                ])
            }
            let block_handle = meshes.add(block);
            commands.spawn((PbrBundle {
                mesh: block_handle,
                material: materials.add(StandardMaterial {
                    base_color_texture: Some(taquin_sprite_handles.bevy.clone()),
                    double_sided: true,
                    cull_mode: None,
                    ..default()
                }),
                transform: Transform::from_translation(Vec3 { x: origin.x + i as f32 * tile_width + tile_width / 2. as f32, y: origin.y - j as f32 * tile_height + tile_height / 2., z: 0.75 }),
                ..default()
            }, Tile));
        }

        next_state.set(AppState::Running);
    }
}

fn rotate(mut query: Query<(&mut Transform), With<Tile>>, time: Res<Time>) {
    for mut transform in &mut query {
        transform.rotate_y(time.delta_seconds());
        transform.rotate_x(time.delta_seconds());
    }
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
