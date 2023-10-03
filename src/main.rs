use std::{f32::consts::PI, ops::Add};

use bevy::{prelude::*, render::{render_resource::{TextureFormat, TextureDimension, Extent3d}, mesh::VertexAttributeValues}};
use scene_hook::{SceneHook, HookPlugin};
use rand::Rng;


mod scene_hook;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(HookPlugin)
        .add_state::<AppState>()
        .init_resource::<Taquin>()
        .init_resource::<Markers>()
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
struct Taquin {
    size: i8,
    tiles_nb: usize,
    tiles: Vec<Vec<Tile>>
}

impl Taquin {
    fn get_next_selection_position(&self, current_position: &TilePosition, direction: KeyCode) -> TilePosition {
        let mut position = *current_position;
        match direction {
            KeyCode::Left => {
                loop {
                    position.i -= 1;
                    if position.i < 0 {
                        position.i = self.size - 1;
                    }
                    if !self.tiles[position.j as usize][position.i as usize].is_empty(self.size) {
                        return position
                    }
                }
            },
            KeyCode::Right => {
                loop {
                    position.i += 1;
                    if position.i >= self.size {
                        position.i = 0;
                    }
                    if !self.tiles[position.j as usize][position.i as usize].is_empty(self.size) {
                        return position
                    }
                }
            },
            KeyCode::Up => {
                loop {
                    position.j -= 1;
                    if position.j < 0 {
                        position.j = self.size - 1;
                    }
                    if !self.tiles[position.j as usize][position.i as usize].is_empty(self.size) {
                        return position
                    }
                }
            },
            KeyCode::Down => {
                loop {
                    position.j += 1;
                    if position.j >= self.size {
                        position.j = 0;
                    }
                    if !self.tiles[position.j as usize][position.i as usize].is_empty(self.size) {
                        return position
                    }
                }
            }
            _ => position
        }
    }
}

impl FromWorld for Taquin {
    fn from_world(_world: &mut World) -> Self {
        Taquin { size: 2, tiles_nb: 4, tiles: vec![] }
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

#[derive(Component, Debug, Clone, Copy, PartialEq, PartialOrd)]
struct Tile(i8);

impl Tile {
    fn is_empty(&self, taquin_size: i8) -> bool {
        return self.0 == taquin_size * taquin_size;
    }
}

#[derive(Component, Debug)]
struct TileSelected;

#[derive(Component, Debug, PartialEq, Clone, Copy)]
struct TilePosition {
    i: i8,
    j: i8
}

impl TilePosition {
    fn new(i: i8, j:i8) -> Self {
        Self {i, j}
    }

    fn is_neighbour_of(&self, other: &TilePosition)-> bool {
        self.get_neighbours().contains(other)
    }

    fn get_neighbours(self) -> Vec<TilePosition>
    {
        vec![self + (1, 0), self + (0, 1), self + (-1, 0), self + (0, -1)]
    }
}

impl Add<(i8, i8)> for TilePosition {
    type Output = Self;

    fn add(self, other: (i8, i8)) -> Self {
        Self {
            i: self.i + other.0,
            j: self.j + other.1,
        }
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
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut taquin : ResMut<Taquin>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let taquin_inner_width = markers.tr.unwrap().x - markers.tl.unwrap().x;
    let taquin_inner_height = markers.tr.unwrap().y - markers.br.unwrap().y;
    let tile_width = taquin_inner_width / taquin.size as f32;
    let tile_height = taquin_inner_height / taquin.size as f32;
    let tile_width_ratio = tile_width / taquin_inner_width;
    let tile_height_ratio = tile_height / taquin_inner_height;
    let origin = markers.tl.unwrap();

    taquin.tiles = (0..taquin.size).map(|j| {
        (0..taquin.size).map(|i| {
            let translation = Vec3 { 
                x: origin.x + i as f32 * tile_width + tile_width / 2. as f32, 
                y: origin.y - j as f32 * tile_height - tile_height / 2., 
                z: 0.75
            };
            if i == taquin.size - 1 && j == taquin.size - 1 {
                commands.spawn((Transform::from_translation(translation), EmptyTile, TilePosition::new(i, j)));
                return Tile(taquin.size * taquin.size);
            }
            let mut block = Mesh::from(shape::Quad::new(Vec2::new(tile_width, tile_height)));
            if let Some(attr) = block.attribute_mut(Mesh::ATTRIBUTE_UV_0) {
                *attr = VertexAttributeValues::Float32x2(vec![
                    [0. + i as f32 * tile_width_ratio, (j + 1) as f32 * tile_height_ratio],
                    [0. + i as f32 * tile_width_ratio, j  as f32 * tile_height_ratio],
                    [(i + 1) as f32 * tile_width_ratio, j  as f32 * tile_height_ratio],
                    [(i + 1) as f32 * tile_width_ratio, (j + 1) as f32 * tile_height_ratio],
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
            }, TilePosition::new(i, j)));
            if i == 0 && j == 0 {
                tile_command.insert(TileSelected);
            }
            Tile(j * taquin.size + i + 1)
        }).collect()
    }).collect();
    
    next_state.set(AppState::Running);
}

fn tile_selection_toggle(
    query: Query<(&Handle<StandardMaterial>, &TilePosition), Changed<TileSelected>>,
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
    selected_tile_query: Query<(Entity, &TilePosition), With<TileSelected>>,
    tiles_query: Query<(Entity, &TilePosition), Without<TileSelected>>,
    keyboard_input: Res<Input<KeyCode>>,
    taquin : Res<Taquin>,
    mut commands: Commands,
) {
    let Ok((selected_tile_entity, selected_tile_index)) = selected_tile_query.get_single() else {
        return;
    };
    let mut selected_tile_new_position = *selected_tile_index;

    if keyboard_input.just_released(KeyCode::Left) {
        selected_tile_new_position = taquin.get_next_selection_position(selected_tile_index, KeyCode::Left)
    } else if keyboard_input.just_released(KeyCode::Right) {
        selected_tile_new_position = taquin.get_next_selection_position(selected_tile_index, KeyCode::Right)
    } else if keyboard_input.just_released(KeyCode::Up) {
        selected_tile_new_position = taquin.get_next_selection_position(selected_tile_index, KeyCode::Up)
    } else if keyboard_input.just_released(KeyCode::Down) {
        selected_tile_new_position = taquin.get_next_selection_position(selected_tile_index, KeyCode::Down)
    }

    for (tile_entity, tile_position) in tiles_query.iter() {
        if *tile_position == selected_tile_new_position {
            commands.entity(selected_tile_entity).remove::<TileSelected>();
            commands.entity(tile_entity).insert(TileSelected);
        }
    }
}

fn move_selected_tile(
    mut selected_tile_query: Query<(&mut Transform, &mut TilePosition), (With<TileSelected>, Without<EmptyTile>)>,
    mut empty_tile_query: Query<(&mut Transform, &mut TilePosition), (With<EmptyTile>, Without<TileSelected>)>,
    keyboard_input: Res<Input<KeyCode>>,
    mut taquin : ResMut<Taquin>,
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

    if selected_tile_index.is_neighbour_of(empty_tile_index.as_ref()) {
        std::mem::swap(empty_tile_transform.as_mut(), selected_tile_transform.as_mut());
        std::mem::swap(empty_tile_index.as_mut(), selected_tile_index.as_mut());

        let temp_tile = taquin.tiles[selected_tile_index.j as usize][selected_tile_index.i as usize];
        taquin.tiles[selected_tile_index.j as usize][selected_tile_index.i as usize] = taquin.tiles[empty_tile_index.j as usize][empty_tile_index.i as usize];
        taquin.tiles[empty_tile_index.j as usize][empty_tile_index.i as usize] = temp_tile;
    }
}

fn randomize_tiles(
    mut taquin : ResMut<Taquin>,
    keyboard_input: Res<Input<KeyCode>>,
    mut tiles_query: Query<(&mut Transform, &mut TilePosition, Option<&EmptyTile>)>,
) {
    if !keyboard_input.just_released(KeyCode::R) {
        return;
    }

    let mut rng = rand::thread_rng();
    let mut empty_tile_position: Option<TilePosition> = None;
    for _i in 0..64 {
        let n1: usize = rng.gen_range(0..taquin.tiles_nb as usize);
        let n2: usize = rng.gen_range(0..taquin.tiles_nb as usize);
        if n1 == n2 {
            continue;
        }
        let mut tiles_iter = tiles_query.iter_mut();
        if let (Some(mut tile1), Some(mut tile2)) = (tiles_iter.nth(n1), tiles_iter.nth(n2)) {
            std::mem::swap(tile1.0.as_mut(), tile2.0.as_mut());
            std::mem::swap(tile1.1.as_mut(), tile2.1.as_mut());
    
            if tile1.2.is_some() {
                empty_tile_position = Some(tile1.1.to_owned())
            } else if tile2.2.is_some() {
                empty_tile_position = Some(tile2.1.to_owned())
            }
            
            let temp_tile = taquin.tiles[tile1.1.j as usize][tile1.1.i as usize];
            taquin.tiles[tile1.1.j as usize][tile1.1.i as usize] = taquin.tiles[tile2.1.j as usize][tile2.1.i as usize];
            taquin.tiles[tile2.1.j as usize][tile2.1.i as usize] = temp_tile;
        }
    }

    println!("EMPTY POSITION : {:?}", empty_tile_position);
    if let Some(position) = empty_tile_position {
        if !is_solvable(taquin.as_ref(), position) {
            println!("PAS SOLVABLE");
        } else {
            println!("SOLVABLE");
        }
    }
}

fn get_inversion_count(
    taquin: &Taquin
) -> usize
{
    let mut inversion_counter: usize = 0;
    let flat_tiles = taquin.tiles.iter().flatten().collect::<Vec<&Tile>>();
    (0..(taquin.tiles_nb - 1)).for_each(|i| {
        ((i + 1)..taquin.tiles_nb).for_each(|j| {
            println!("COMPARING {:?} > {:?} RESULT {:?}", flat_tiles[i], flat_tiles[j], flat_tiles[i] > flat_tiles[j]);
            if flat_tiles[i].0 != taquin.tiles_nb as i8 && flat_tiles[j].0 != taquin.tiles_nb as i8 && flat_tiles[i] > flat_tiles[j] {
                inversion_counter += 1;
            }
        })
    });
    return inversion_counter;
}

fn is_solvable(
    taquin: &Taquin,
    empty_tile_position: TilePosition
) -> bool {
    println!("{:?}", taquin.tiles);
    let inversion_count = get_inversion_count(taquin);
    println!("Nombre inversion {:?}", inversion_count);

    if taquin.size & 1 == 1 {
        return empty_tile_position.j & 1 == 0;
    }

    if empty_tile_position.j & 1 == 1 {
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

    use crate::{Tile, is_solvable, Taquin, TilePosition};

    #[test]
    fn test_is_solvable() {
        let mut app = App::new();

        app.world.insert_resource(Taquin {
            size: 2,
            tiles_nb: 4,
            tiles: vec![vec![Tile(1), Tile(2)], vec![Tile(3), Tile(4)]]
        });
        assert_eq!(is_solvable(app.world.resource::<Taquin>(), TilePosition::new(1, 1)), true);

        app.world.insert_resource(Taquin {
            size: 2,
            tiles_nb: 4,
            tiles: vec![vec![Tile(4), Tile(1)], vec![Tile(2), Tile(3)]]
        });
        assert_eq!(is_solvable(app.world.resource::<Taquin>(), TilePosition::new(0, 0)), true);
    }

    
    #[test]
    fn test_is_not_solvable() {
        let mut app = App::new();

        app.world.insert_resource(Taquin {
            size: 2,
            tiles_nb: 4,
            tiles: vec![vec![Tile(2), Tile(1)], vec![Tile(3), Tile(4)]]
        });
        assert_eq!(is_solvable(app.world.resource::<Taquin>(), TilePosition::new(1, 1)), false);
    }
}