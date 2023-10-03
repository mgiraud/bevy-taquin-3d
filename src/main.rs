use std::f32::consts::PI;

use bevy::{prelude::*, render::{render_resource::{TextureFormat, TextureDimension, Extent3d}, mesh::VertexAttributeValues}};
use scene_hook::{SceneHook, HookPlugin};
use rand::Rng;


mod scene_hook;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(HookPlugin)
        .add_state::<AppState>()
        .init_resource::<AppConfig>()
        .init_resource::<Markers>()
        .init_resource::<Tiles>()
        .add_systems(OnEnter(AppState::Setup), setup_scene)
        .add_systems(Update, setup_markers.run_if(in_state(AppState::Setup)))
        .add_systems(Update, check_setup_finished.run_if(in_state(AppState::Setup)))
        .add_systems(OnEnter(AppState::SetupTiles), setup_tiles)
        .add_systems(Update, (
            tile_selection_toggle, move_tile_selection, react_on_removal, move_selected_tile
        ).run_if(in_state(AppState::Running)))
        .add_systems(Update, randomize_tiles)
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
    taquin_size: i8,
    tiles_nb: i8,
}

impl FromWorld for AppConfig {
    fn from_world(_world: &mut World) -> Self {
        AppConfig { taquin_size: 2, tiles_nb: 4 }
    }
}

#[derive(Resource, Debug)]
struct Tiles (Vec<Tile>);

impl FromWorld for Tiles {
    fn from_world(_world: &mut World) -> Self {
        Tiles(vec![])
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

#[derive(Component, Debug)]
struct EmptyTile;

#[derive(Component, Debug)]
struct Tile(i8);

#[derive(Component, Debug)]
struct TileSelected;

#[derive(Component, Debug)]
struct TileIndex(i8);


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
    query: Query<(&Name, &GlobalTransform), With<Marker>>
) {
    for (name, global_transform) in query.iter() {
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
    mut tiles: ResMut<Tiles>,
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
            let translation = Vec3 { 
                x: origin.x + i as f32 * tile_width + tile_width / 2. as f32, 
                y: origin.y - j as f32 * tile_height + tile_height / 2., 
                z: 0.75
            };
            
            let tile_index = (j - 1) * app_config.taquin_size + i;

            if i == app_config.taquin_size - 1 && j == app_config.taquin_size {
                commands.spawn((Transform::from_translation(translation), EmptyTile, TileIndex(tile_index)));
                tiles.0.push(Tile(0));
                continue;
            }
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
            let mut tile_command = commands.spawn((PbrBundle {
                mesh: block_handle,
                material: materials.add(StandardMaterial {
                    base_color_texture: Some(taquin_sprite_handles.bevy.clone()),
                    double_sided: true,
                    cull_mode: None,
                    ..default()
                }),
                transform: Transform::from_translation(translation),
                ..default()
            }, TileIndex(tile_index)));
            if i == 0 && j == 1 {
                tile_command.insert(TileSelected);
            }
            tiles.0.push(Tile(tile_index + 1))
        }

        next_state.set(AppState::Running);
    }
}

fn tile_selection_toggle(
    query: Query<(&Handle<StandardMaterial>, &TileIndex), Changed<TileSelected>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    for (material, position) in &query {
        info!("{:?} changed: {:?}", material, position);
        if let Some(material) = materials.get_mut(material) {
            material.emissive = Color::RED;
        } 
    }
}

fn react_on_removal(
    mut removed: RemovedComponents<TileSelected>, mut query: Query<&Handle<StandardMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    for entity in removed.read() {
        if let Ok(material_handle) = query.get_mut(entity) {
            if let Some(material) = materials.get_mut(material_handle) {
                material.emissive = Color::BLACK;
            } 
        }
    }
}

fn move_tile_selection(
    empty_tile_query: Query<&TileIndex, (With<EmptyTile>, Without<TileSelected>)>,
    selected_tile_query: Query<(Entity, &TileIndex), With<TileSelected>>,
    tiles_query: Query<(Entity, &TileIndex), Without<TileSelected>>,
    keyboard_input: Res<Input<KeyCode>>,
    app_config : Res<AppConfig>,
    mut commands: Commands,
) {
    let Ok((selected_tile_entity, selected_tile_index)) = selected_tile_query.get_single() else {
        return;
    };
    let Ok(empty_tile_index) = empty_tile_query.get_single() else {
        return;
    };
    let mut selected_tile_new_position = selected_tile_index.0;

    if keyboard_input.just_released(KeyCode::Left) {
        selected_tile_new_position -= 1;
        if selected_tile_new_position == empty_tile_index.0 {
            selected_tile_new_position -= 1;
        }
    } else if keyboard_input.just_released(KeyCode::Right) {
        selected_tile_new_position += 1;
        if selected_tile_new_position == empty_tile_index.0 {
            selected_tile_new_position += 1;
        }
    } else if keyboard_input.just_released(KeyCode::Up) {
        selected_tile_new_position -= app_config.taquin_size;
        if selected_tile_new_position == empty_tile_index.0 {
            selected_tile_new_position -= app_config.taquin_size;
        }
    } else if keyboard_input.just_released(KeyCode::Down) {
        selected_tile_new_position += app_config.taquin_size;
        if selected_tile_new_position == empty_tile_index.0 {
            selected_tile_new_position += app_config.taquin_size;
        }
    }

    if selected_tile_new_position < 0 {
        selected_tile_new_position = app_config.taquin_size.pow(2) - ((selected_tile_new_position + 1) % app_config.taquin_size) - 1;
        if selected_tile_new_position == empty_tile_index.0 {
            selected_tile_new_position -= 1;
        }
    } else if selected_tile_new_position > app_config.taquin_size.pow(2) - 1 {
        selected_tile_new_position = selected_tile_new_position % app_config.taquin_size;
    }

    for (tile_entity, tile_position) in tiles_query.iter() {
        if tile_position.0 == selected_tile_new_position {
            commands.entity(selected_tile_entity).remove::<TileSelected>();
            commands.entity(tile_entity).insert(TileSelected);
        }
    }
}

fn move_selected_tile(
    mut selected_tile_query: Query<(&mut Transform, &mut TileIndex), (With<TileSelected>, Without<EmptyTile>)>,
    mut empty_tile_query: Query<(&mut Transform, &mut TileIndex), (With<EmptyTile>, Without<TileSelected>)>,
    keyboard_input: Res<Input<KeyCode>>,
    app_config : Res<AppConfig>,
    mut tiles: ResMut<Tiles>,
) {
    if !keyboard_input.just_released(KeyCode::Space) {
        return;
    }
    let Ok((mut empty_tile_transform, mut empty_tile_index)) = empty_tile_query.get_single_mut() else {
        return;
    };
    let Ok((mut selected_tile_transform, mut selected_tile_index)) = selected_tile_query.get_single_mut() else {
        return;
    };
    let diff = empty_tile_index.0 - selected_tile_index.0;
    let mut diff_checker: Vec<i8> = vec![];
    if selected_tile_index.0 % app_config.taquin_size != 0 {
        diff_checker.push(-1);
    }
    if selected_tile_index.0 % app_config.taquin_size != app_config.taquin_size - 1 {
        diff_checker.push(1);
    }
    if selected_tile_index.0 + app_config.taquin_size < app_config.tiles_nb {
        diff_checker.push(app_config.taquin_size);
    }
    if selected_tile_index.0 - app_config.taquin_size >= 0 {
        diff_checker.push(-app_config.taquin_size);
    }
    
    if diff_checker.contains(&diff) {
        let temp_transform = *empty_tile_transform;
        *empty_tile_transform = *selected_tile_transform;
        *selected_tile_transform = temp_transform;

        let temp_index = empty_tile_index.0;
        empty_tile_index.0 = selected_tile_index.0;
        selected_tile_index.0 = temp_index;

        tiles.0.swap(selected_tile_index.0 as usize, empty_tile_index.0 as usize);
    }
}

fn randomize_tiles(
    app_config : Res<AppConfig>,
    keyboard_input: Res<Input<KeyCode>>,
    mut tiles: ResMut<Tiles>,
    mut tiles_query: Query<(&mut Transform, &mut TileIndex, Option<&EmptyTile>)>,
) {
    if !keyboard_input.just_released(KeyCode::R) {
        return;
    }

    let mut rng = rand::thread_rng();
    let mut empty_tile_index: Option<i8> = None;
    for _i in 0..64 {
        let n1: usize = rng.gen_range(0..app_config.tiles_nb as usize);
        let n2: usize = rng.gen_range(0..app_config.tiles_nb as usize);
        if n1 == n2 {
            continue;
        }
        let mut tiles_iter = tiles_query.iter_mut();
        if let (Some(mut tile1), Some(mut tile2)) = (tiles_iter.nth(n1), tiles_iter.nth(n2)) {
            let temp_transform = *(tile1.0);
            *tile1.0 = *tile2.0;
            *tile2.0 = temp_transform;
    
            let temp_index = tile1.1.0;
            tile1.1.0 = tile2.1.0;
            tile2.1.0 = temp_index;
            if tile1.2.is_some() {
                empty_tile_index = Some(tile1.1.0)
            } else if tile2.2.is_some() {
                empty_tile_index = Some(tile2.1.0)
            }
            tiles.0.swap(tile1.1.0 as usize, tile2.1.0 as usize)
        }
    }

    if let Some(index) = empty_tile_index {
        if !is_solvable(tiles.as_mut(), app_config.as_ref(),index) {
            println!("PAS SOLVABLE");
        } else {
            println!("SOLVABLE");
        }
    }
}

fn get_inversion_count(
    tiles: &Tiles,
) -> usize
{
    let mut inversion_counter: usize = 0;
    let tile_1_iter = tiles.0.iter().take(tiles.0.len() - 1);
    for (i, tile_1) in tile_1_iter.enumerate() {
        let tile_index_1 = tile_1.0;
        for tile_2 in tiles.0.iter().skip(i + 1) {
            let tile_index_2 = tile_2.0;
            if tile_index_1 != 0 && tile_index_2 != 0 && tile_index_1 > tile_index_2 {
                inversion_counter += 1;
            }
        }
    }
    return inversion_counter;
}

fn is_solvable<'a>(
    tiles: &'a Tiles,
    app_config: &'a AppConfig,
    empty_tile_index: i8
) -> bool {
    println!("{:?}", tiles);
    let inversion_count = get_inversion_count(tiles);
    println!("Nombre inversion {:?}", inversion_count);

    if app_config.taquin_size & 1 == 1 {
        return inversion_count & 1 == 0;
    }

    if empty_tile_index & 1 == 1 {
        return inversion_count & 1 == 0;
    }

    return inversion_count & 1 == 1;
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

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use crate::{Tile, is_solvable, AppConfig, Tiles};

    #[test]
    fn test_is_solvable() {
        let mut app = App::new();

        app.world.insert_resource(Tiles(vec![Tile(1), Tile(2), Tile(3), Tile(0)]));
        app.world.insert_resource(AppConfig {
            taquin_size: 2,
            tiles_nb: 4,
        });
        assert_eq!(is_solvable(app.world.resource::<Tiles>(), app.world.resource::<AppConfig>(), 3), true);
    }
}