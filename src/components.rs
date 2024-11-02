use std::collections::VecDeque;
use bevy::prelude::{Component, Entity};
use bevy::utils::HashSet;
use crate::uVec3::uVec3;

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
pub struct TileMap{
    pub running: bool,
    pub current_state: HashSet<uVec3>,
    pub state_stack: VecDeque<HashSet<uVec3>>,
}

#[derive(Component)]
pub struct Tile;

#[derive(Component)]
pub struct TilesCache{
    pub entities: Vec<Entity>
}