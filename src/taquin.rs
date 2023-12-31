use bevy::{prelude::*, input::{keyboard::KeyboardInput, ButtonState}};
use rand::Rng;

use crate::{tile::{TileCoordinates, TileValue, EmptyTile, TileSelected, TileLerp}, AppState, TaquinSprites};

pub struct TaquinPlugin {
    pub(crate) size: i8
}

impl Plugin for TaquinPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<TaquinShuffled>()
            .add_event::<TaquinSolved>()
            .add_event::<TileMoved>()
            .insert_resource(Taquin::new(self.size))
            .init_resource::<TaquinSoundHandles>()
            .add_systems(Update, (move_tile_selection, (on_taquin_solved_play_tada, on_taquin_solved_reset_is_shuffled).chain().run_if(on_event::<TaquinSolved>())).run_if(in_state(AppState::Running)))
            .add_systems(Update, (move_selected_tile, shuffle).run_if(in_state(AppState::Running).and_then(not(any_with_component::<TileLerp>()))))
            .add_systems(Update, toggle_taquin_texture)
        ;
    }
}

#[derive(Event, Default)]
pub struct TaquinShuffled;

#[derive(Event, Default)]
pub struct TaquinSolved;

#[derive(Event, Default)]
pub struct TileMoved;

#[derive(Resource)]
struct TaquinSoundHandles {
    tada: Handle<AudioSource>,
}

impl FromWorld for TaquinSoundHandles {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Self {
            tada: asset_server.load("sounds/tada.ogg"),
        }
    }
}

#[derive(Resource, Default)]
pub struct Taquin {
    pub size: i8,
    pub tiles_nb: usize,
    pub tiles: Vec<Vec<TileValue>>,
    pub is_shuffled: bool,
}

impl Taquin {
    pub fn new(size: i8) -> Self {
        Self { size, tiles_nb: (size * size) as usize, tiles: vec![], is_shuffled: false }
    }

    pub fn get_next_selection_coordinates(&self, current_coordinates: &TileCoordinates, direction: KeyCode) -> TileCoordinates {
        let mut coordinates = *current_coordinates;
        match direction {
            KeyCode::Left => {
                loop {
                    coordinates.i -= 1;
                    if coordinates.i < 0 {
                        coordinates.i = self.size - 1;
                    }
                    if !self.tiles[coordinates.j as usize][coordinates.i as usize].is_empty(self.size) {
                        return coordinates
                    }
                }
            },
            KeyCode::Right => {
                loop {
                    coordinates.i += 1;
                    if coordinates.i >= self.size {
                        coordinates.i = 0;
                    }
                    if !self.tiles[coordinates.j as usize][coordinates.i as usize].is_empty(self.size) {
                        return coordinates
                    }
                }
            },
            KeyCode::Up => {
                loop {
                    coordinates.j -= 1;
                    if coordinates.j < 0 {
                        coordinates.j = self.size - 1;
                    }
                    if !self.tiles[coordinates.j as usize][coordinates.i as usize].is_empty(self.size) {
                        return coordinates
                    }
                }
            },
            KeyCode::Down => {
                loop {
                    coordinates.j += 1;
                    if coordinates.j >= self.size {
                        coordinates.j = 0;
                    }
                    if !self.tiles[coordinates.j as usize][coordinates.i as usize].is_empty(self.size) {
                        return coordinates
                    }
                }
            }
            _ => coordinates
        }
    }

    fn get_inversion_count(
        &self
    ) -> usize
    {
        let mut inversion_counter: usize = 0;
        let flat_tiles = self.tiles.iter().flatten().collect::<Vec<&TileValue>>();
        (0..(self.tiles_nb - 1)).for_each(|i| {
            ((i + 1)..self.tiles_nb).for_each(|j| {
                if flat_tiles[i].0 != self.tiles_nb as i8 && flat_tiles[j].0 != self.tiles_nb as i8 && flat_tiles[i] > flat_tiles[j] {
                    inversion_counter += 1;
                }
            })
        });
        return inversion_counter;
    }

    pub fn get_empty_tile_coordinates(&self) -> TileCoordinates
    {
        let mut ret_i = 0;
        let mut ret_j = 0;

        self.tiles.iter().enumerate().for_each(|(j, row)| {
           row.iter().enumerate().for_each(|(i, tile)|  {
                if tile.0 == self.tiles_nb as i8 {
                    ret_i = i;
                    ret_j = j;
                }
           })
        });

        TileCoordinates::new(ret_i as i8, ret_j as i8)
    }

    pub fn is_solvable(&self) -> bool {
        let inversion_count = self.get_inversion_count();
        let empty_tile_coordinates = self.get_empty_tile_coordinates();

        if self.size & 1 == 1 {
            return inversion_count & 1 == 0;
        }
    
        if empty_tile_coordinates.j & 1 == 1 {
            return inversion_count & 1 == 0;
        }
    
        inversion_count & 1 == 1
    }

    pub fn is_solved(&self) -> bool {
        self.tiles.iter()
            .flatten()
            .collect::<Vec<&TileValue>>()
            .windows(2)
            .filter(|a| {
                a.get(1).is_some() && a[0] > a[1] 
            })
            .count() == 0
    }

    pub fn swap_tiles(&mut self, a: TileCoordinates, b: TileCoordinates) {
        let temp_tile = self.tiles[a.j as usize][a.i as usize];
        self.tiles[a.j as usize][a.i as usize] = self.tiles[b.j as usize][b.i as usize];
        self.tiles[b.j as usize][b.i as usize] = temp_tile;
    }
}

fn move_tile_selection(
    selected_tile_query: Query<(Entity, &TileCoordinates), With<TileSelected>>,
    tiles_query: Query<(Entity, &TileCoordinates), Without<TileSelected>>,
    taquin : Res<Taquin>,
    mut commands: Commands,
    mut keyboard_input_events: EventReader<KeyboardInput>
) {
    let Ok((selected_tile_entity, selected_tile_coordinates)) = selected_tile_query.get_single() else {
        return;
    };

    let Some(KeyboardInput {key_code: Some(key_code), state : ButtonState::Released, ..}) = keyboard_input_events.read().next() else {
        return;
    };

    let selected_tile_new_coordinates = taquin.get_next_selection_coordinates(selected_tile_coordinates, *key_code);
    if selected_tile_new_coordinates != *selected_tile_coordinates {
        for (tile_entity, tile_coordinates) in tiles_query.iter() {
            if *tile_coordinates == selected_tile_new_coordinates {
                commands.entity(selected_tile_entity).remove::<TileSelected>();
                commands.entity(tile_entity).insert(TileSelected);
            }
        }
    }

}

fn move_selected_tile(
    mut commands: Commands,
    mut selected_tile_query: Query<(Entity, &Transform, &mut TileCoordinates), (With<TileSelected>, Without<EmptyTile>)>,
    mut empty_tile_query: Query<(&mut Transform, &mut TileCoordinates), (With<EmptyTile>, Without<TileSelected>)>,
    keyboard_input: Res<Input<KeyCode>>,
    mut taquin : ResMut<Taquin>,
    mut solved_events: EventWriter<TaquinSolved>,
    mut tile_moved_events: EventWriter<TileMoved>,
) {
    if !keyboard_input.just_released(KeyCode::Space) {
        return;
    }
    let Ok((mut empty_tile_transform, mut empty_tile_coords)) = empty_tile_query.get_single_mut() else {
        return;
    };
    let Ok((entity, selected_tile_transform, mut selected_tile_coords)) = selected_tile_query.get_single_mut() else {
        return;
    };

    if selected_tile_coords.is_neighbour_of(empty_tile_coords.as_ref()) {
        std::mem::swap(empty_tile_coords.as_mut(), selected_tile_coords.as_mut());
        taquin.swap_tiles(*selected_tile_coords, *empty_tile_coords);
        commands.entity(entity).insert(TileLerp(empty_tile_transform.translation));
        empty_tile_transform.translation = selected_tile_transform.translation;
        tile_moved_events.send_default();
    }

    if taquin.is_solved() {
        solved_events.send_default();
    }
}

fn shuffle(
    mut taquin : ResMut<Taquin>,
    mut shuffle_events: EventWriter<TaquinShuffled>,
    keyboard_input: Res<Input<KeyCode>>,
    mut tiles_query: Query<(&mut Transform, &mut TileCoordinates)>,
) {
    if !keyboard_input.just_released(KeyCode::R) {
        return;
    }

    loop {
        if do_shuffle(taquin.as_mut(), &mut tiles_query) == true {
            taquin.is_shuffled = true;
            shuffle_events.send_default();
            break;
        }
    }
}

fn do_shuffle(taquin : &mut Taquin, tiles_query: &mut Query<(&mut Transform, &mut TileCoordinates)>) -> bool {
    let mut rng = rand::thread_rng();
    for _i in 0..taquin.tiles_nb.pow(2) {
        let n1: usize = rng.gen_range(0..taquin.tiles_nb as usize);
        let n2: usize = rng.gen_range(0..taquin.tiles_nb as usize);
        if n1 == n2 {
            continue;
        }
        let mut tiles_iter = tiles_query.iter_mut();
        if let (Some(mut tile1), Some(mut tile2)) = (tiles_iter.nth(n1), tiles_iter.nth(n2)) {
            std::mem::swap(tile1.0.as_mut(), tile2.0.as_mut());
            std::mem::swap(tile1.1.as_mut(), tile2.1.as_mut());
            taquin.swap_tiles(*tile1.1, *tile2.1);
        }
    }

    !taquin.is_solved() && taquin.is_solvable()
}

fn on_taquin_solved_play_tada(
    taquin: Res<Taquin>,
    mut commands: Commands,
    handles: Res<TaquinSoundHandles>
) {
    if !taquin.is_shuffled {
        return;
    }

    commands.spawn(AudioBundle {
        source: handles.tada.clone(),
        settings: PlaybackSettings::DESPAWN,
    });
}

fn on_taquin_solved_reset_is_shuffled(
    mut taquin: ResMut<Taquin>
) {
    taquin.is_shuffled = false;
}

fn toggle_taquin_texture(
    keyboard_input: Res<Input<KeyCode>>,
    taquin_sprite_handles: Res<TaquinSprites>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    materials_query: Query<&Handle<StandardMaterial>, With<TileCoordinates>>,
) {
    if !keyboard_input.just_released(KeyCode::T) {
        return;
    }
    for material_handle in materials_query.iter() {
        let Some(material) = materials.get_mut(material_handle) else {
            return;
        };
    
        if material.base_color_texture == Some(taquin_sprite_handles.bevy.clone()) {
            material.base_color_texture = Some(taquin_sprite_handles.rust.clone());
        } else {
            material.base_color_texture = Some(taquin_sprite_handles.bevy.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use crate::{TileValue, Taquin};

    #[test]
    fn test_is_solvable() {
        let mut app = App::new();

        app.world.insert_resource(Taquin {
            size: 2,
            tiles_nb: 4,
            tiles: vec![vec![TileValue(1), TileValue(2)], vec![TileValue(3), TileValue(4)]],
            is_shuffled: true,
        });
        assert_eq!(app.world.resource::<Taquin>().is_solvable(), true);

        app.world.insert_resource(Taquin {
            size: 2,
            tiles_nb: 4,
            tiles: vec![vec![TileValue(4), TileValue(3)], vec![TileValue(2), TileValue(1)]],
            is_shuffled: true,
        });
        assert_eq!(app.world.resource::<Taquin>().is_solvable(), true);

        app.world.insert_resource(Taquin {
            size: 2,
            tiles_nb: 4,
            tiles: vec![vec![TileValue(2), TileValue(3)], vec![TileValue(1), TileValue(4)]],
            is_shuffled: true,
        });
        assert_eq!(app.world.resource::<Taquin>().is_solvable(), true);
    }

    
    #[test]
    fn test_is_not_solvable() {
        let mut app = App::new();

        app.world.insert_resource(Taquin {
            size: 2,
            tiles_nb: 4,
            tiles: vec![vec![TileValue(2), TileValue(1)], vec![TileValue(3), TileValue(4)]],
            is_shuffled: true,
        });
        assert_eq!(app.world.resource::<Taquin>().is_solvable(), false);

        app.world.insert_resource(Taquin {
            size: 2,
            tiles_nb: 4,
            tiles: vec![vec![TileValue(4), TileValue(1)], vec![TileValue(2), TileValue(3)]],
            is_shuffled: true,
        });
        assert_eq!(app.world.resource::<Taquin>().is_solvable(), false);
    }
}