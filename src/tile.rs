use bevy::prelude::*;
use std::ops::Add;

use crate::AppState;

pub struct TilePlugin;

impl Plugin for TilePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.
            add_systems(Update, (
                on_tile_selected_changed, on_tile_selected_removal
            ).run_if(in_state(AppState::Running)));
    }
}

#[derive(Component, Debug)]
pub struct EmptyTile;

#[derive(Component, Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Tile(pub i8);

impl Tile {
    pub fn is_empty(&self, taquin_size: i8) -> bool {
        return self.0 == taquin_size * taquin_size;
    }
}

#[derive(Component, Debug)]
pub struct TileSelected;

#[derive(Component, Debug, PartialEq, Clone, Copy)]
pub struct TilePosition {
    pub i: i8,
    pub j: i8
}

impl TilePosition {
    pub fn new(i: i8, j:i8) -> Self {
        Self {i, j}
    }

    pub fn is_neighbour_of(&self, other: &TilePosition)-> bool {
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

fn on_tile_selected_changed(
    query: Query<&Handle<StandardMaterial>, Changed<TileSelected>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    for material in &query {
        if let Some(material) = materials.get_mut(material) {
            material.emissive = Color::RED;
        } 
    }
}

fn on_tile_selected_removal(
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