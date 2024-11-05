use std::collections::VecDeque;
use bevy::asset::{Assets, Handle};
use bevy::input::ButtonInput;
use bevy::input::mouse::MouseWheel;
use bevy::math::Vec3;
use bevy::prelude::{default, Camera, Camera2dBundle, ColorMaterial, Commands, Entity, EventReader, GlobalTransform, KeyCode, Mesh, MouseButton, OrthographicProjection, Query, Rectangle, Res, ResMut, Time, Transform, Window, With};
use bevy::reflect::Array;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use bevy::utils::HashSet;
use bevy::window::{PresentMode, PrimaryWindow};
use crate::{CAMERA_SPEED, INVISIBLE, WHITE, ZOOM_MULTIPLIER};
use crate::components;
use crate::components::*;
use crate::uVec3::uVec3;

pub fn setup_camera(mut commands: Commands){
    let mut proj = OrthographicProjection::default();
    proj.near = -1.;
    proj.far = 5.;
    proj.scale = 0.5;
    commands.spawn((
        Camera2dBundle{
            transform: Transform::from_xyz(0.0,0.0,0.0),
            projection: proj,
            ..default()
        },
        components::MainCamera,
    ));
}

pub fn setup_batching(
    mut commands: Commands,
    mut meshAssets: ResMut<Assets<Mesh>>,
    mut materialAssets: ResMut<Assets<ColorMaterial>>
) {

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshAssets.add(Rectangle::default()).into(),
            transform: Transform::from_xyz(0.,0.,-7.).with_scale(Vec3::splat(1.)),
            material: materialAssets.add(WHITE),
            ..default()
        },
        ReferenceTile
    ));

}

pub fn camera_mouvement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut mouse_wheel_input: EventReader<MouseWheel>,
    mut camera_query: Query<(&mut Transform, &mut OrthographicProjection), With<components::MainCamera>>,
    time: Res<Time>
){
    if let Ok((mut transform, mut projection)) = camera_query.get_single_mut(){
        let mut direction = Vec3::ZERO;

        if keyboard_input.pressed(KeyCode::KeyW) {
            direction += Vec3::new(0.,1.,0.);
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            direction += Vec3::new(0.,-1.,0.);
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            direction += Vec3::new(1.,0.,0.);
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            direction += Vec3::new(-1.,0.,0.);
        }

        if direction.length() > 0.0 {
            direction = direction.normalize();
        }

        transform.translation += direction * CAMERA_SPEED * time.delta_seconds();

        let mut wheel: f32 = 0.0;

        for i in mouse_wheel_input.read(){
            wheel += i.y;
        }

        let mut zoom = projection.scale;
        if wheel != 0.0 {
            zoom -= (wheel * 0.01) * zoom * ZOOM_MULTIPLIER;

            if zoom < 0.01 {
                zoom = 0.01;
            }
            projection.scale = zoom;
        }
        // println!("{}", zoom);
    }
}

pub fn tile_placement(
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<components::MainCamera>>,
    mut tilemap_query: Query<&mut TileMap>,
    mouse_input: Res<ButtonInput<MouseButton>>
){
    let mut key: MouseButton = MouseButton::Forward;
    if(mouse_input.pressed(MouseButton::Left) && !mouse_input.pressed(MouseButton::Right)){
        key = MouseButton::Left;
    } else if(mouse_input.pressed(MouseButton::Right) && !mouse_input.pressed(MouseButton::Left)){
        key = MouseButton::Right;
    }
    if(key == MouseButton::Left){
        if let Ok(mut tileMap) = tilemap_query.get_single_mut(){
            let (camera, global_transform) = camera_query.single();
            let window = window_query.single();
            if let Some(mut world_position) = window.cursor_position()
                .and_then(|cursor| camera.viewport_to_world(global_transform, cursor))
                .map(|ray| ray.origin.truncate()){
                world_position.x = world_position.x.round();
                world_position.y = world_position.y.round();
                tileMap.current_state.insert(uVec3::new(world_position.x as i32, world_position.y as i32, 0));
            }
        }
    } else if key == MouseButton::Right{
        if let Ok(mut tileMap) = tilemap_query.get_single_mut(){
            let (camera, global_transform) = camera_query.single();
            let window = window_query.single();
            if let Some(mut world_position) = window.cursor_position()
                .and_then(|cursor| camera.viewport_to_world(global_transform, cursor))
                .map(|ray| ray.origin.truncate()){
                world_position.x = world_position.x.round();
                world_position.y = world_position.y.round();
                tileMap.current_state.remove(&uVec3::new(world_position.x as i32, world_position.y as i32, 0));
            }
        }
    }
}

pub fn setup_simulation(
    mut commands: Commands,
){
    commands.spawn((
        TileMap{
            running: false,
            current_state: HashSet::new(),
            state_stack: VecDeque::new(),
        }
    ));
}


//TODO multithread this shit and make it despawn tiles instead of lifting
pub fn display_tilemap(
    mut refresh_timer_query: Query<&mut RefreshTimer>,
    mut tilemap_query: Query<&TileMap>,
    mut tile_query: Query<(Entity, &Tile, &mut Transform)>,
    mut reference_tile_query: Query<(&ReferenceTile, &Mesh2dHandle, &Handle<ColorMaterial>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    time: Res<Time>
){
    let mut refresh_timer = refresh_timer_query.single_mut();
    // println!("{} {} {}", refresh_timer.lastRefresh, refresh_timer.timeBetweenRefresh, time.elapsed().as_millis());
    if(refresh_timer.timeBetweenRefresh != 0 && refresh_timer.lastRefresh + refresh_timer.timeBetweenRefresh >= time.elapsed().as_millis()){
        return;
    }
    refresh_timer.lastRefresh = time.elapsed().as_millis();

    let tileMap = tilemap_query.single();
    let tileMapSize = tileMap.current_state.len();

    let mut tiles = tile_query.iter_mut();
    let tilesSize = tiles.len();

    let mut index = 0;

    let mut currTile = tiles.next();

    let (reft, refmesh, refmat) = reference_tile_query.single();
    
    for pos in &tileMap.current_state{
        if(!currTile.is_none()){
            let (entity, tile, mut transform) = currTile.unwrap();
            transform.translation = pos.toVec3();
            // let color_mat = materials.get_mut(mat).unwrap();
            // color_mat.color = WHITE;
            currTile = tiles.next();
        } else if(index < tileMapSize){
            commands.spawn((
                MaterialMesh2dBundle {
                    mesh: refmesh.clone(),
                    transform: Transform::from_translation(pos.toVec3()).with_scale(Vec3::splat(1.)),
                    material: refmat.clone(),
                    ..default()
                },
                Tile
            ));

        }
        index += 1;
    }

    if(index < tilesSize){
        let unusedPercentage = ((tilesSize-index) / tilesSize) * 100;
        for i in 0..(tilesSize-index){
            if(!currTile.is_none()){
                let (entity, tile, mut transform) = currTile.unwrap();
                if(unusedPercentage > 20 && (((tilesSize-index) / tilesSize) * 100) > 80){
                    commands.entity(entity).despawn();
                } else {
                    transform.translation = Vec3::new(0.,0.,-7.);
                    // let color_mat = materials.get_mut(mat).unwrap();
                    // color_mat.color = Color::linear_rgba(1.,0.,0.,1.);
                }
                currTile = tiles.next();
            }
        }
    }

    // println!("showing: {}/{} in {}ms", index, tilesSize, time.delta().as_millis());
}

pub fn setup_tiles_cache(mut commands: Commands){
    commands.spawn(TilesCache{
        entities: Vec::new()
    });
}

pub fn display_cube_material(
    mut tile_query: Query<(&Handle<ColorMaterial>, &mut Transform), With<Tile>>,
    mut materials: ResMut<Assets<ColorMaterial>>
){
    for (mat, transform) in tile_query.iter(){
        if(transform.translation.x == 1.0){
            let color_mat = materials.get_mut(mat).unwrap();
            color_mat.color = INVISIBLE;
        }
    }
}

pub fn setup_refresh_timer(
    mut commands: Commands,
    time: Res<Time>,
){
    commands.spawn(
        RefreshTimer{
            lastRefresh: time.elapsed().as_millis(),
            timeBetweenRefresh: 0,
        }
    );
}


pub fn run_simulation(
    mut tilemap_query: Query<&mut TileMap>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
){
    let mut tileMap = tilemap_query.single_mut();
    if(keyboard_input.just_pressed(KeyCode::Space)){
        tileMap.running = !tileMap.running;
        if(tileMap.running){
            println!("Simulation started");
        } else {
            println!("Simulation stopped");
        }
    }
    if(tileMap.running){
        let mut newTileMap: HashSet<uVec3> = tileMap.current_state.clone();
        //TODO multithread the inside of this loop
        for tile in tileMap.current_state.iter(){
            checkArround(tile, &tileMap.current_state, &mut newTileMap);
        }
        //
        tileMap.current_state = newTileMap;
    }
}

pub fn checkArround(pos: &uVec3, tileMap: &HashSet<uVec3>, newTileMap: &mut HashSet<uVec3>){
    let mut count = 0;
    for i in -1..2{
        for j in -1..2{
            if(!tileMap.contains(&uVec3::new(pos.x + i, pos.y + j, 0))){
                let mut countArround = 0;
                for k in -1..2{
                    for l in -1..2{
                        if(!(k==0 && l==0) && tileMap.contains(&uVec3::new(pos.x + i + k, pos.y + j + l, 0))){
                            countArround += 1;
                        }
                    }
                }
                if(countArround == 3){
                    newTileMap.insert(uVec3::new(pos.x + i, pos.y + j, 0));
                }
            }
            if(!(i==0 && j==0) && tileMap.contains(&uVec3::new(pos.x + i, pos.y + j, 0))){
                count += 1;
            }
        }
    }
    if(count < 2 || count > 3){
        newTileMap.remove(&uVec3::new(pos.x, pos.y, 0));
    } else {
        newTileMap.insert(uVec3::new(pos.x, pos.y, 0));
    }
}


pub fn place_patterns(
    mut tilemap_query: Query<&mut TileMap>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
){
    if(keyboard_input.just_pressed(KeyCode::KeyB)){
        let mut tileMap = tilemap_query.single_mut();

        let benchmark = [
            [ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ],
            [ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ],
            [ 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0 ],
            [ 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0 ],
            [ 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0 ],
            [ 0, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 1, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0 ],
            [ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ],
            [ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ],
            [ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ],
            [ 0, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 1, 1, 0, 0, 1, 0, 0, 1, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 1, 0, 0, 1, 0, 0, 1, 1, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0 ],
            [ 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0 ],
            [ 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0 ],
            [ 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0 ],
            [ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ],
            [ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ],
            [ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ],
            [ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ],
            [ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ],
            [ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ],
            [ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ],
            [ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ],
            [ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ],
            [ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ],
            [ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 1, 0, 1, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ],
            [ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ],
            [ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ],
            [ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ],
            [ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ],
            [ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ],
            [ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ],
        ];

        for i in 0..30 {
            for j in 0..57{
                if(benchmark[i][j] == 1){
                    tileMap.current_state.insert(uVec3::new(i as i32, j as i32, 0));
                }
            }
        }
    }
    if(keyboard_input.just_pressed(KeyCode::KeyG)){
        let mut tileMap = tilemap_query.single_mut();

        let glider = [
            [0,0,1],
            [1,0,1],
            [0,1,1],
        ];

        for i in 0..3 {
            for j in 0..3{
                if(glider[i][j] == 1){
                    tileMap.current_state.insert(uVec3::new(i as i32, j as i32, 0));
                }
            }
        }
    }
    if(keyboard_input.just_pressed(KeyCode::Backspace)){
        let mut tileMap = tilemap_query.single_mut();
        tileMap.current_state.clear();
    }




}

pub fn toggle_vsync(input: Res<ButtonInput<KeyCode>>, mut windows: Query<&mut Window>){
    if input.just_pressed(KeyCode::KeyV){
        let mut window = windows.single_mut();

        window.present_mode = if matches!(window.present_mode, PresentMode::AutoVsync) {
            PresentMode::AutoNoVsync
        }else{
            PresentMode::AutoVsync
        };
        println!("PRESENT MODE : {:?}", window.present_mode)
    }
}