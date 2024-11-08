use bevy::prelude::{Component, Entity};
use bevy::utils::{HashMap};
use crate::uVec3::uVec3;

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
pub struct TileMap{
    pub running: bool,
    pub current_state: HashMap<uVec3, (Option<Entity>, i32)>,
    pub stable_current_state: HashMap<uVec3, (Option<Entity>, i32)>,
}

#[derive(Component)]
pub struct Tile;

#[derive(Component)]
pub struct ReferenceTile;

#[derive(Component)]
pub struct TilesCache{
    pub entities: Vec<Entity>
}

#[derive(Component)]
pub struct RefreshTimer{
    pub lastRefresh: u128,
    pub timeBetweenRefresh: u128,
}