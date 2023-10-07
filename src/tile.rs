use bevy::prelude::*;
use std::ops::Add;

use crate::AppState;

pub struct TilePlugin;

impl Plugin for TilePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .add_systems(Update, (
                on_tile_selected_changed, on_tile_selected_removal, move_tile
            ).run_if(in_state(AppState::Running)));
    }
}

#[derive(Component, Debug)]
pub struct EmptyTile;

#[derive(Component, Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct TileValue(pub i8);

impl TileValue {
    pub fn is_empty(&self, taquin_size: i8) -> bool {
        return self.0 == taquin_size * taquin_size;
    }
}

#[derive(Component, Debug)]
pub struct TileSelected;

#[derive(Component, Debug, PartialEq, Clone, Copy)]
pub struct TileCoordinates {
    pub i: i8,
    pub j: i8
}

impl TileCoordinates {
    pub fn new(i: i8, j:i8) -> Self {
        Self {i, j}
    }

    pub fn is_neighbour_of(&self, other: &TileCoordinates)-> bool {
        self.get_neighbours().contains(other)
    }

    fn get_neighbours(self) -> Vec<TileCoordinates>
    {
        vec![self + (1, 0), self + (0, 1), self + (-1, 0), self + (0, -1)]
    }
}

impl Add<(i8, i8)> for TileCoordinates {
    type Output = Self;

    fn add(self, other: (i8, i8)) -> Self {
        Self {
            i: self.i + other.0,
            j: self.j + other.1,
        }
    }
}

#[derive(Component, Debug)]
pub struct TileLerp(pub Vec3);

#[derive(Component, Debug, Default)]
pub struct TileAnimations {
    pub up: Handle<AnimationClip>,
    pub right: Handle<AnimationClip>,
    pub down: Handle<AnimationClip>,
    pub left: Handle<AnimationClip>,
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

fn move_tile(
    mut commands: Commands,
    mut tile_query: Query<(Entity, &mut Transform, &TileLerp)>, 
) {
    let Ok((entity, mut transform, tile_lerp)) = tile_query.get_single_mut() else {
        return;
    };

    transform.translation = transform.translation.lerp(tile_lerp.0, 0.25);

    if transform.translation.abs_diff_eq(tile_lerp.0, 0.01) {
        transform.translation = tile_lerp.0;
        commands.entity(entity).remove::<TileLerp>();
    }
}