//! A module for shared resources that don't have a special home yet

use bevy::prelude::*;

use crate::item_plugin::ItemType;

#[derive(Component, Copy, Clone)]
pub struct OldLoc(pub f32, pub f32);

#[derive(Component, Hash, Eq, PartialEq, Clone, Debug)]
pub struct Terrain(pub usize, pub usize);

#[derive(Component)]
pub struct DynamicPos;

#[derive(Component)]
pub struct Camera;

#[derive(Component)]
pub struct Indestructible;

#[derive(Debug, Copy, Clone)]
pub struct TileSettings {
    pub height: usize,
    pub copse: bool,
    pub kind: TileType,
    pub item: Option<ItemType>,
}
impl Default for TileSettings {
    fn default() -> Self {
        TileSettings {
            height: 0,
            copse: false,
            kind: TileType::Base,
            item: None,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum TileType {
    Base,
    B,
}

pub fn type_to_colour(s: &TileType) -> Color {
    match s {
        TileType::Base => Color::rgb(0.1, 0.5, 0.1),
        TileType::B => Color::rgb(0.3, 0.3, 0.3),
    }
}
