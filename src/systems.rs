#[path="./components.rs"]
mod components;

use bevy::asset::Assets;
use bevy::input::ButtonInput;
use bevy::input::mouse::MouseWheel;
use bevy::math::Vec3;
use bevy::prelude::{default, Camera, Camera2dBundle, ColorMaterial, Commands, EventReader, GlobalTransform, KeyCode, Mesh, MouseButton, OrthographicProjection, Query, Rectangle, Res, ResMut, Time, Transform, Window, With};
use bevy::sprite::MaterialMesh2dBundle;
use bevy::window::PrimaryWindow;
use crate::{CAMERA_SPEED, WHITE, ZOOM_MULTIPLIER};

pub fn setup_camera(mut commands: Commands){
    let mut proj = OrthographicProjection::default();
    proj.near = -1000.;
    proj.scale = 1.;
    commands.spawn((
        Camera2dBundle{
            transform: Transform::from_xyz(0.0,0.0,0.0),
            projection: proj,
            ..default()
        },
        components::MainCamera,
    ));
}



pub fn setup_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(Rectangle::default()).into(),
            transform: Transform::from_xyz(0., 0., 0.).with_scale(Vec3::splat(1.)),
            material: materials.add(WHITE),
            ..default()
        },
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

        if(keyboard_input.pressed(KeyCode::KeyW)){
            direction += Vec3::new(0.,1.,0.);
        }
        if(keyboard_input.pressed(KeyCode::KeyS)){
            direction += Vec3::new(0.,-1.,0.);
        }
        if(keyboard_input.pressed(KeyCode::KeyD)){
            direction += Vec3::new(1.,0.,0.);
        }
        if(keyboard_input.pressed(KeyCode::KeyA)){
            direction += Vec3::new(-1.,0.,0.);
        }

        if(direction.length() > 0.0){
            direction = direction.normalize();
        }

        transform.translation += direction * CAMERA_SPEED * time.delta_seconds();

        let mut wheel: f32 = 0.0;

        for i in mouse_wheel_input.read(){
            wheel += i.y;
        }

        let mut zoom = projection.scale;
        if(wheel != 0.0){
            zoom -= (wheel * 0.01) * zoom * ZOOM_MULTIPLIER;

            if(zoom < 0.01){
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
    mouse_input: Res<ButtonInput<MouseButton>>
){
    let mut key: MouseButton = MouseButton::Forward;
    if(mouse_input.just_pressed(MouseButton::Left) && !mouse_input.just_pressed(MouseButton::Right)){
        key = MouseButton::Left;
    } else if(mouse_input.just_pressed(MouseButton::Right) && !mouse_input.just_pressed(MouseButton::Left)){
        key = MouseButton::Right;
    }
    if(key != MouseButton::Forward){
        let (camera, global_transform) = camera_query.single();
        let window = window_query.single();
        if let Some(mut world_position) = window.cursor_position()
            .and_then(|cursor| camera.viewport_to_world(global_transform, cursor))
            .map(|ray| ray.origin.truncate()){
            world_position.x = world_position.x.round();
            world_position.y = world_position.y.round();
            eprintln!("World coords: {}/{}", world_position.x, world_position.y);
        }
    }
}