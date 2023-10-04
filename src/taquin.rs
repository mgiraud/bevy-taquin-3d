use bevy::{prelude::*, input::{keyboard::KeyboardInput, ButtonState}};
use rand::Rng;

use crate::{tile::{TilePosition, Tile, EmptyTile, TileSelected}, AppState};

pub struct TaquinPlugin {
    pub(crate) size: i8
}

impl Plugin for TaquinPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(Taquin::new(self.size))
            .add_systems(Update, (move_tile_selection,  move_selected_tile, randomize_tiles).run_if(in_state(AppState::Running)));
    }
}

#[derive(Resource, Default)]
pub struct Taquin {
    pub size: i8,
    pub tiles_nb: usize,
    pub tiles: Vec<Vec<Tile>>
}

impl Taquin {
    pub fn new(size: i8) -> Self {
        Self { size, tiles_nb: (size * size) as usize, tiles: vec![] }
    }

    pub fn get_next_selection_position(&self, current_position: &TilePosition, direction: KeyCode) -> TilePosition {
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

    fn get_inversion_count(
        &self
    ) -> usize
    {
        let mut inversion_counter: usize = 0;
        let flat_tiles = self.tiles.iter().flatten().collect::<Vec<&Tile>>();
        (0..(self.tiles_nb - 1)).for_each(|i| {
            ((i + 1)..self.tiles_nb).for_each(|j| {
                if flat_tiles[i].0 != self.tiles_nb as i8 && flat_tiles[j].0 != self.tiles_nb as i8 && flat_tiles[i] > flat_tiles[j] {
                    inversion_counter += 1;
                }
            })
        });
        return inversion_counter;
    }

    pub fn get_empty_tile_position(&self) -> TilePosition
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

        TilePosition::new(ret_i as i8, ret_j as i8)
    }

    pub fn is_solvable(&self) -> bool {
        let inversion_count = self.get_inversion_count();
        let empty_tile_position = self.get_empty_tile_position();

        if self.size & 1 == 1 {
            return empty_tile_position.j & 1 == 0;
        }
    
        if empty_tile_position.j & 1 == 1 {
            return inversion_count & 1 == 0;
        }
    
        return inversion_count & 1 == 1;
    }

    pub fn is_solved(&self) -> bool {
        self.tiles.iter()
            .flatten()
            .collect::<Vec<&Tile>>()
            .chunks(2)
            .filter(|a| { a[0] > a[1] })
            .count() == 0
    }

    pub fn swap_tiles(&mut self, a: TilePosition, b: TilePosition) {
        let temp_tile = self.tiles[a.j as usize][a.i as usize];
        self.tiles[a.j as usize][a.i as usize] = self.tiles[b.j as usize][b.i as usize];
        self.tiles[b.j as usize][b.i as usize] = temp_tile;
    }
}

fn move_tile_selection(
    selected_tile_query: Query<(Entity, &TilePosition), With<TileSelected>>,
    tiles_query: Query<(Entity, &TilePosition), Without<TileSelected>>,
    taquin : Res<Taquin>,
    mut commands: Commands,
    mut keyboard_input_events: EventReader<KeyboardInput>
) {
    let Ok((selected_tile_entity, selected_tile_position)) = selected_tile_query.get_single() else {
        return;
    };

    for event in keyboard_input_events.read() {
        let (Some(key_code), ButtonState::Released) = (event.key_code, event.state) else {
            continue;
        };
        let selected_tile_new_position = taquin.get_next_selection_position(selected_tile_position, key_code);
        if selected_tile_new_position != *selected_tile_position {
            for (tile_entity, tile_position) in tiles_query.iter() {
                if *tile_position == selected_tile_new_position {
                    commands.entity(selected_tile_entity).remove::<TileSelected>();
                    commands.entity(tile_entity).insert(TileSelected);
                }
            }
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
        taquin.swap_tiles(*selected_tile_index, *empty_tile_index);
    }

    if taquin.is_solved() {
        println!("SOLVED !");
    }
}

fn randomize_tiles(
    mut taquin : ResMut<Taquin>,
    keyboard_input: Res<Input<KeyCode>>,
    mut tiles_query: Query<(&mut Transform, &mut TilePosition)>,
) {
    if !keyboard_input.just_released(KeyCode::R) {
        return;
    }

    let mut rng = rand::thread_rng();
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
            taquin.swap_tiles(*tile1.1, *tile2.1);
        }
    }

    if !taquin.is_solvable() {
        println!("PAS SOLVABLE");
    } else {
        println!("SOLVABLE");
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use crate::{Tile, Taquin};

    #[test]
    fn test_is_solvable() {
        let mut app = App::new();

        app.world.insert_resource(Taquin {
            size: 2,
            tiles_nb: 4,
            tiles: vec![vec![Tile(1), Tile(2)], vec![Tile(3), Tile(4)]]
        });
        assert_eq!(app.world.resource::<Taquin>().is_solvable(), true);

        app.world.insert_resource(Taquin {
            size: 2,
            tiles_nb: 4,
            tiles: vec![vec![Tile(4), Tile(3)], vec![Tile(2), Tile(1)]]
        });
        assert_eq!(app.world.resource::<Taquin>().is_solvable(), true);

        app.world.insert_resource(Taquin {
            size: 2,
            tiles_nb: 4,
            tiles: vec![vec![Tile(2), Tile(3)], vec![Tile(1), Tile(4)]]
        });
        assert_eq!(app.world.resource::<Taquin>().is_solvable(), true);
    }

    
    #[test]
    fn test_is_not_solvable() {
        let mut app = App::new();

        app.world.insert_resource(Taquin {
            size: 2,
            tiles_nb: 4,
            tiles: vec![vec![Tile(2), Tile(1)], vec![Tile(3), Tile(4)]]
        });
        assert_eq!(app.world.resource::<Taquin>().is_solvable(), false);

        app.world.insert_resource(Taquin {
            size: 2,
            tiles_nb: 4,
            tiles: vec![vec![Tile(4), Tile(1)], vec![Tile(2), Tile(3)]]
        });
        assert_eq!(app.world.resource::<Taquin>().is_solvable(), false);
    }
}