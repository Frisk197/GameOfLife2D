use bevy::prelude::{Component, Entity};
use bevy::utils::{HashMap};
use crate::uVec3::uVec3;

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
pub struct TileMap{
    pub running: bool,
    pub current_state: HashMap<uVec3, (Option<Entity>, i32)>,   // map qui est mise à jour toute les frames
    pub stable_current_state: HashMap<uVec3, (Option<Entity>, i32)>,  // map qui contiens les pixels n'ayant pas été mis à jour depuis plus de X itérations. ces pixels sont "réveillé" (transféré vers l'autre hashmap) quand il y a une mise à jour d'un pixel à proximité.
}

#[derive(Component)]
pub struct Tile;

#[derive(Component)]
pub struct InTileMap;

#[derive(Component)]
pub struct Stable;

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