use std::collections::VecDeque;
use std::process::Command;
use bevy::asset::{Assets, Handle};
use bevy::input::ButtonInput;
use bevy::input::mouse::MouseWheel;
use bevy::math::Vec3;
use bevy::prelude::{default, Camera, Camera2dBundle, ColorMaterial, Commands, Entity, EventReader, GlobalTransform, In, KeyCode, Mesh, MouseButton, OrthographicProjection, Query, Rectangle, Res, ResMut, Time, Transform, Window, With, Without};
use bevy::reflect::Array;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use bevy::utils::{HashMap, HashSet};
use bevy::window::{PresentMode, PrimaryWindow};
use crate::{CAMERA_SPEED, INVISIBLE, UNSTBLE_CHANGER_LIMIT, UPDATE_COUNT_LIMIT, WHITE, ZOOM_MULTIPLIER};
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
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut tilemap_query: Query<&mut TileMap>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
    mut tile_query: Query<(Entity, &Tile, &mut Transform), With<InTileMap>>,
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
                if  !tileMap.current_state.contains_key(&uVec3::new(world_position.x as i32, world_position.y as i32, 0)){
                    tileMap.current_state.insert(uVec3::new(world_position.x as i32, world_position.y as i32, 0), (None, UPDATE_COUNT_LIMIT));
                    let mut switching: Vec<(uVec3, Entity)> = Vec::new();
                    for i in -UNSTBLE_CHANGER_LIMIT..UNSTBLE_CHANGER_LIMIT+1{
                        for j in -UNSTBLE_CHANGER_LIMIT..UNSTBLE_CHANGER_LIMIT+1{
                            if tileMap.stable_current_state.contains_key(&uVec3::new(world_position.x as i32 + i, world_position.y as i32 + j, 0)){
                                let (entity, updateCounter) = tileMap.stable_current_state.get(&uVec3::new(world_position.x as i32 + i, world_position.y as i32 + j, 0)).unwrap();
                                switching.push((uVec3::new(world_position.x as i32 + i, world_position.y as i32 + j, 0), entity.unwrap()));
                            }
                        }
                    }

                    for (spos, sent) in switching{
                        tileMap.current_state.insert(spos, (Option::from(sent), UPDATE_COUNT_LIMIT));
                        tileMap.stable_current_state.remove(&spos);
                    }
                }
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
                let mut switching: Vec<(uVec3, Entity)> = Vec::new();
                for i in -1..2{
                    for j in -1..2{
                        if tileMap.stable_current_state.contains_key(&uVec3::new(world_position.x as i32 + i, world_position.y as i32 + j, 0)){
                            let (entity, updateCounter) = tileMap.stable_current_state.get(&uVec3::new(world_position.x as i32 + i, world_position.y as i32 + j, 0)).unwrap();
                            switching.push((uVec3::new(world_position.x as i32 + i, world_position.y as i32 + j, 0), entity.unwrap()));
                        }
                    }
                }
                for (spos, sent) in switching{
                    tileMap.current_state.insert(spos, (Option::from(sent), UPDATE_COUNT_LIMIT));
                    tileMap.stable_current_state.remove(&spos);
                }

                if let Some((entity, updateCounter)) = tileMap.stable_current_state.get(&uVec3::new(world_position.x as i32, world_position.y as i32, 0)){
                    if !entity.is_none(){
                        commands.entity(entity.unwrap()).remove::<InTileMap>();
                        tileMap.stable_current_state.remove(&uVec3::new(world_position.x as i32, world_position.y as i32, 0));
                    }
                }
                if let Some((entity, updateCounter)) = tileMap.current_state.get(&uVec3::new(world_position.x as i32, world_position.y as i32, 0)){
                    if !entity.is_none(){
                        commands.entity(entity.unwrap()).remove::<InTileMap>();
                        tileMap.current_state.remove(&uVec3::new(world_position.x as i32, world_position.y as i32, 0));
                    }
                }
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
            current_state: HashMap::new(),
            stable_current_state: HashMap::new(),
        }
    ));
}


pub fn hide_off_tiles(
    mut not_in_map_tile_query: Query<(Entity, &Tile, &mut Transform), Without<InTileMap>>,
){
    not_in_map_tile_query.par_iter_mut().for_each(|(entity, tile, mut transform)|{
        transform.translation = Vec3::new(transform.translation.x, transform.translation.y, -7.);
    });
}

//TODO multithread this shit and make it despawn tiles instead of lifting
pub fn display_tilemap(
    mut refresh_timer_query: Query<&mut RefreshTimer>,
    mut tilemap_query: Query<&mut TileMap>,
    mut not_in_map_tile_query: Query<(Entity, &Tile, &mut Transform), Without<InTileMap>>,
    mut tile_query: Query<(Entity, &Tile, &mut Transform), With<InTileMap>>,
    mut reference_tile_query: Query<(&ReferenceTile, &Mesh2dHandle, &Handle<ColorMaterial>)>,
    mut commands: Commands,
    time: Res<Time>
){

    let mut refresh_timer = refresh_timer_query.single_mut();
    // println!("{} {} {}", refresh_timer.lastRefresh, refresh_timer.timeBetweenRefresh, time.elapsed().as_millis());
    if(refresh_timer.timeBetweenRefresh != 0 && refresh_timer.lastRefresh + refresh_timer.timeBetweenRefresh >= time.elapsed().as_millis()){
        return;
    }
    refresh_timer.lastRefresh = time.elapsed().as_millis();

    let mut tileMap = tilemap_query.single_mut();
    let sizea = tileMap.current_state.len();
    let sizeb = tileMap.stable_current_state.len();

    let tileMapSize = sizea + sizeb;

    let mut tiles = not_in_map_tile_query.iter_mut();
    let tilesSize = tiles.len();

    let mut index = 0;

    let mut currTile = tiles.next();



    let (reft, refmesh, refmat) = reference_tile_query.single();
    
    for (pos, (entity, updateState)) in tileMap.current_state.iter_mut(){
        if !entity.is_none(){
            if let Ok((tileEntity, tile, mut transform)) = tile_query.get_mut(entity.unwrap()){
                // println!("im in there");
                transform.translation = pos.toVec3();
            } else if(!currTile.is_none()){
                // println!("im right here");
                let (tileEntity, tile, mut tileTransform) = currTile.unwrap();
                *entity = Option::from(tileEntity);
                tileTransform.translation = pos.toVec3();
                currTile = tiles.next();
                commands.entity(tileEntity).insert(InTileMap);
            } else if(index < tileMapSize){
                // println!("over there");
                let ent = commands.spawn((
                    MaterialMesh2dBundle {
                        mesh: refmesh.clone(),
                        transform: Transform::from_translation(pos.toVec3()).with_scale(Vec3::splat(1.)),
                        material: refmat.clone(),
                        ..default()
                    },
                    Tile,
                    InTileMap
                ));
                *entity = Option::from(ent.id());
            }
        } else {
            if(!currTile.is_none()){
                let (tileEntity, tile, mut tileTransform) = currTile.unwrap();
                *entity = Option::from(tileEntity);
                tileTransform.translation = pos.toVec3();
                currTile = tiles.next();
                commands.entity(tileEntity).insert(InTileMap);
            } else if(index < tileMapSize){
                let ent = commands.spawn((
                    MaterialMesh2dBundle {
                        mesh: refmesh.clone(),
                        transform: Transform::from_translation(pos.toVec3()).with_scale(Vec3::splat(1.)),
                        material: refmat.clone(),
                        ..default()
                    },
                    Tile,
                    InTileMap
                ));
                *entity = Option::from(ent.id());
            }
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
                }
                currTile = tiles.next();
            }
        }
    }

    println!("{} + {} = {} tiles {}s", sizea, sizeb, tileMapSize, time.delta().as_secs_f64());
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
    mut commands: Commands,
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
        let mut newTileMap: HashMap<uVec3, (Option<Entity>, i32)> = tileMap.current_state.clone();
        let mut newStableTileMap: HashMap<uVec3, (Option<Entity>, i32)> = tileMap.stable_current_state.clone();
        let mut inTileMapRemove: Vec<Entity> = Vec::new();
        //TODO multithread the inside of this loop
        for tile in tileMap.current_state.iter(){
            checkArround(tile.0, &tileMap.current_state, &tileMap.stable_current_state, &mut newTileMap, &mut newStableTileMap, &mut inTileMapRemove);
        }
        //
        for t in inTileMapRemove{
            commands.entity(t).remove::<InTileMap>();
        }
        tileMap.current_state = newTileMap;
        tileMap.stable_current_state = newStableTileMap;
    }
}



pub fn checkArround(pos: &uVec3, tileMap: &HashMap<uVec3, (Option<Entity>, i32)>, stableTileMap: &HashMap<uVec3, (Option<Entity>, i32)>, newTileMap: &mut HashMap<uVec3, (Option<Entity>, i32)>, newStableTileMap: &mut HashMap<uVec3, (Option<Entity>, i32)>, inTileMapRemove: &mut Vec<Entity>){
    let mut count = 0;
    for i in -1..2{
        for j in -1..2{
            if(!tileMap.contains_key(&uVec3::new(pos.x + i, pos.y + j, 0)) && !stableTileMap.contains_key(&uVec3::new(pos.x + i, pos.y + j, 0))){
                let mut countArround = 0;
                let mut tilesAvailable: Vec<uVec3> = Vec::new();
                for k in -1..2{
                    for l in -1..2{
                        if(!(k==0 && l==0) && (tileMap.contains_key(&uVec3::new(pos.x + i + k, pos.y + j + l, 0)) || stableTileMap.contains_key(&uVec3::new(pos.x + i + k, pos.y + j + l, 0)))){
                            countArround += 1;
                            tilesAvailable.push(*pos);
                        }
                    }
                }
                if(countArround == 3){
                    newTileMap.insert(uVec3::new(pos.x + i, pos.y + j, 0), (None, UPDATE_COUNT_LIMIT));
                    for p in tilesAvailable{
                        if stableTileMap.contains_key(&p){
                            for k in -UNSTBLE_CHANGER_LIMIT..UNSTBLE_CHANGER_LIMIT+1{
                                for l in -UNSTBLE_CHANGER_LIMIT..UNSTBLE_CHANGER_LIMIT+1{
                                    if stableTileMap.contains_key(&uVec3::new(pos.x+k, pos.y+l, pos.z)) || tileMap.contains_key(&uVec3::new(pos.x+k, pos.y+l, pos.z)){
                                        let (entity, updateCount) = stableTileMap.get(&uVec3::new(pos.x, pos.y, pos.z)).unwrap();
                                        newStableTileMap.remove(&uVec3::new(p.x + k, p.y + l, 0));
                                        newTileMap.insert(uVec3::new(p.x + k, p.y + l, 0), (*entity, UPDATE_COUNT_LIMIT));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if(!(i==0 && j==0) && (tileMap.contains_key(&uVec3::new(pos.x + i, pos.y + j, 0)) || stableTileMap.contains_key(&uVec3::new(pos.x + i, pos.y + j, 0)))){
                count += 1;
            }
        }
    }
    if(count < 2 || count > 3){
        let (ent, upd) = tileMap.get(&uVec3::new(pos.x, pos.y, 0)).unwrap();
        if(!ent.is_none()){
            inTileMapRemove.push(ent.unwrap());
        }
        newTileMap.remove(&uVec3::new(pos.x, pos.y, 0));
        newStableTileMap.remove(&uVec3::new(pos.x, pos.y, 0));
        for i in -UNSTBLE_CHANGER_LIMIT..UNSTBLE_CHANGER_LIMIT+1{
            for j in -UNSTBLE_CHANGER_LIMIT..UNSTBLE_CHANGER_LIMIT+1{
                if !(i==0 && j==0) && stableTileMap.contains_key(&uVec3::new(pos.x + i, pos.y + j, 0)){
                    let (entity, updateStep) = stableTileMap.get(&uVec3::new(pos.x + i, pos.y + j, 0)).unwrap();
                    newStableTileMap.remove(&uVec3::new(pos.x + i, pos.y + j, 0));
                    newTileMap.insert(uVec3::new(pos.x + i, pos.y + j, 0), (*entity, UPDATE_COUNT_LIMIT));
                }
            }
        }
    } else {
        let (entity, updateStep) = tileMap.get(&uVec3::new(pos.x, pos.y, 0)).unwrap();
        if(*updateStep <= 1){
            // println!("moved one key !!!!");
            newTileMap.remove(&uVec3::new(pos.x, pos.y, 0));
            newStableTileMap.insert(uVec3::new(pos.x, pos.y, 0), (*entity, updateStep-1));
        } else {
            newTileMap.insert(uVec3::new(pos.x, pos.y, 0), (*entity, updateStep-1));
        }
    }
}


pub fn place_patterns(
    mut tilemap_query: Query<&mut TileMap>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut tile_query: Query<(Entity, &Tile, &mut Transform), With<InTileMap>>,
    mut commands: Commands,
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
                    tileMap.current_state.insert(uVec3::new(i as i32, j as i32, 0), (None, UPDATE_COUNT_LIMIT));
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
                    tileMap.current_state.insert(uVec3::new(i as i32, j as i32, 0), (None, UPDATE_COUNT_LIMIT));
                }
            }
        }
    }
    if(keyboard_input.just_pressed(KeyCode::Backspace)){
        let mut tileMap = tilemap_query.single_mut();
        tileMap.current_state.clear();
        tileMap.stable_current_state.clear();
        for (entity, tile, mut transform) in tile_query.iter_mut(){
            commands.entity(entity).remove::<InTileMap>();
            transform.translation = Vec3::new(transform.translation.x, transform.translation.y, -7.);
        }
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